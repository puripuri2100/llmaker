use super::types;

pub fn make_parse_token_fn_str(setting: types::Setting) -> String {
  let (main_type_str, token_and_str_vec) = setting;
  token_and_str_vec_to_str(main_type_str, token_and_str_vec)
}

fn token_and_str_to_str(
  main_type_str: String,
  token_and_str: (types::Range, String, types::TypeStr),
) -> String {
  let (_, token_name, type_str) = token_and_str;
  format!(
    "
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(unused_parens)]
fn _parse_token_{}(
  tokens: &Vec<{}>,
  pos: usize,
) -> Result<({}, usize), ParseError>
{{
  let token1 = tokens.get(pos);
  token1
  .ok_or(ParseError::Eof)
  .and_then(|tok| match tok {{
    {} => Ok((tok.clone(), pos + 1)),
    _ => Err(ParseError::UnexpectedToken(tok.clone())),
  }})
}}
",
    token_name, main_type_str, main_type_str, type_str
  )
}

fn token_and_str_vec_to_str(
  main_type_str: String,
  token_and_str_vec: Vec<(types::Range, String, types::TypeStr)>,
) -> String {
  let mut main_s = String::new();
  for v in token_and_str_vec.iter() {
    let s = token_and_str_to_str(main_type_str.clone(), v.clone());
    main_s.push_str(&s)
  }
  main_s
}
