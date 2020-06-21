use super::error;
use super::types;

pub mod lexer;
pub mod parse;

pub fn get_ast(input: &str) -> Result<types::Term, error::Error> {
  let tokens = match lexer::lex(input) {
    Ok(t) => Ok(t),
    Err(e) => Err(error::Error::LexerError(e)),
  }?;
  let ast = match parse::parse(tokens) {
    Ok(t) => Ok(t),
    Err(e) => Err(error::Error::ParserError(e)),
  }?;
  Ok(ast)
}
