use super::error;
use super::types;
pub mod headstr;
pub mod parse_fn;
pub mod parse_token;

pub fn to_string(term: types::Term) -> Result<String, error::Error> {
  let (head, setting, bnfs) = term;
  let head_str = headstr::head_to_str(head, setting.clone());
  let parse_token_fn_str = parse_token::make_parse_token_fn_str(setting.clone());
  let parse_fn_fn_str = parse_fn::make_parse_fn_fn_str(setting, bnfs)?;
  Ok(format!(
    "{}\n{}\n{}\n",
    head_str, parse_fn_fn_str, parse_token_fn_str
  ))
}
