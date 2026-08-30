use dotilla::cypher::{lexer, token};

use crate::cypher::opencypher;

#[test]
fn opencypher_tck_files() {
    for feature_query in opencypher::FeatureQueries::iter().unwrap() {
        match lexer::lex(&feature_query.query) {
            Ok(lexed) => {
                assert!(
                    !lexed.is_empty(),
                    "{}: {}",
                    feature_query.path.display(),
                    feature_query.scenario
                );
                assert_eq!(
                    lexed.last().map(|token| &token.kind),
                    Some(&token::TokenKind::Eof),
                    "{}: {}",
                    feature_query.path.display(),
                    feature_query.scenario,
                );
            }
            Err(err) => {
                panic!(
                    "{}: {}\nquery:\n{}\nerror: {}",
                    feature_query.path.display(),
                    feature_query.scenario,
                    feature_query.query,
                    err,
                );
            }
        }
    }
}
