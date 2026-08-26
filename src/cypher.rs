use std::sync::Mutex;

pub struct Parser(pub Mutex<tree_sitter::Parser>);

impl Parser {
    pub fn new() -> Result<Self, tree_sitter::LanguageError> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&tree_sitter_cypher::LANGUAGE.into())?;
        Ok(Self(Mutex::new(parser)))
    }

    pub fn parse(&self, source: &str) -> tree_sitter::Tree {
        let mut parser = self.0.lock().unwrap();
        parser.parse(source, None).unwrap()
    }
}
