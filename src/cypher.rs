use tree_sitter::{LanguageError, Parser};

pub fn build_cypher_parser() -> Result<Parser, LanguageError> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_cypher::LANGUAGE.into())?;
    Ok(parser)
}
