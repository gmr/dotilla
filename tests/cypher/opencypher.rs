use std::{
    collections::HashMap,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};

use gherkin::{Feature, GherkinEnv, ParseError, Scenario, Step};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("could not parse feature file {path}: {source}")]
    Gherkin { path: PathBuf, source: ParseError },
}

#[derive(Debug)]
pub struct FeatureQuery {
    pub path: PathBuf,
    pub scenario: String,
    pub query: String,
}

pub struct FeatureQueries;

impl FeatureQueries {
    pub fn iter() -> Result<impl Iterator<Item = FeatureQuery>, Error> {
        let mut queries = Vec::new();

        for path in Self::gather()? {
            let content = fs::read_to_string(&path)?;
            let content = Self::normalize_table_escapes(&content);
            let feature = Feature::parse(content, GherkinEnv::default()).map_err(|source| {
                Error::Gherkin {
                    path: path.clone(),
                    source,
                }
            })?;
            Self::collect_feature_queries(&mut queries, &path, &feature);
        }

        Ok(queries.into_iter())
    }

    fn gather() -> Result<Vec<PathBuf>, Error> {
        let mut files: Vec<PathBuf> = Vec::new();
        let extension: Option<&OsStr> = Some(OsStr::new("feature"));
        let mut stack = vec![PathBuf::from("openCypher/tck/features/")];

        while let Some(current_dir) = stack.pop() {
            for entry in fs::read_dir(current_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension() == extension {
                    files.push(path);
                }
            }
        }
        files.sort();
        Ok(files)
    }

    fn normalize_table_escapes(input: &str) -> String {
        input
            .lines()
            .map(|line| {
                if !line.trim_start().starts_with('|') {
                    return line.to_owned();
                }

                let mut normalized = String::with_capacity(line.len());
                let mut chars = line.chars().peekable();
                while let Some(character) = chars.next() {
                    normalized.push(character);
                    if character != '\\' {
                        continue;
                    }

                    match chars.peek() {
                        Some('\\') => normalized.push(chars.next().unwrap()),
                        Some('n' | '|') | None => {}
                        Some(_) => normalized.push('\\'),
                    }
                }
                normalized
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn collect_feature_queries(queries: &mut Vec<FeatureQuery>, path: &Path, feature: &Feature) {
        if let Some(background) = &feature.background {
            Self::collect_steps(
                queries,
                path,
                &format!("{} (Background)", feature.name),
                &background.steps,
                false,
                &HashMap::new(),
            );
        }

        for scenario in &feature.scenarios {
            Self::collect_scenario(queries, path, scenario);
        }

        for rule in &feature.rules {
            if let Some(background) = &rule.background {
                Self::collect_steps(
                    queries,
                    path,
                    &format!("{} (Background)", rule.name),
                    &background.steps,
                    false,
                    &HashMap::new(),
                );
            }

            for scenario in &rule.scenarios {
                Self::collect_scenario(queries, path, scenario);
            }
        }
    }

    fn collect_scenario(queries: &mut Vec<FeatureQuery>, path: &Path, scenario: &Scenario) {
        let expects_error = scenario
            .steps
            .iter()
            .any(|step| step.value.contains("Error should be raised"));
        let examples = Self::example_rows(scenario);

        if examples.is_empty() {
            Self::collect_steps(
                queries,
                path,
                &scenario.name,
                &scenario.steps,
                expects_error,
                &HashMap::new(),
            );
        } else {
            for substitutions in examples {
                Self::collect_steps(
                    queries,
                    path,
                    &scenario.name,
                    &scenario.steps,
                    expects_error,
                    &substitutions,
                );
            }
        }
    }

    fn collect_steps(
        queries: &mut Vec<FeatureQuery>,
        path: &Path,
        scenario: &str,
        steps: &[Step],
        expects_error: bool,
        substitutions: &HashMap<String, String>,
    ) {
        for step in steps {
            let Some(docstring) = &step.docstring else {
                continue;
            };

            // Keep setup and control queries, but omit the query whose failure is
            // the expected outcome of a negative scenario.
            if expects_error && step.value == "executing query:" {
                continue;
            }

            let query = substitutions
                .iter()
                .fold(docstring.clone(), |query, (key, value)| {
                    query.replace(&format!("<{key}>"), value)
                });

            queries.push(FeatureQuery {
                path: path.to_owned(),
                scenario: scenario.to_owned(),
                query,
            });
        }
    }

    fn example_rows(scenario: &Scenario) -> Vec<HashMap<String, String>> {
        scenario
            .examples
            .iter()
            .filter_map(|examples| examples.table.as_ref())
            .flat_map(|table| {
                let Some((headers, rows)) = table.rows.split_first() else {
                    return Vec::new();
                };

                rows.iter()
                    .map(|row| headers.iter().cloned().zip(row.iter().cloned()).collect())
                    .collect()
            })
            .collect()
    }
}
