use super::error;
use super::types;
use std::collections::HashMap;

pub fn make_parse_fn_fn_str(
  setting: types::Setting,
  bnfs: &Vec<types::Bnf>,
) -> Result<String, error::Error> {
  let (main_type_str, token_tbl) = setting;
  let mut token_map = HashMap::new();
  for (_, tokenname, typestr) in token_tbl.iter() {
    token_map.insert(tokenname, typestr);
  }
  let mut fn_name_map = HashMap::new();
  for bnf in bnfs.iter() {
    let (range, name, typestr, code_vec) = match bnf {
      types::Bnf::Pub(range, name, typestr, code_vec) => (range, name, typestr, code_vec),
      types::Bnf::NonPub(range, name, typestr, code_vec) => (range, name, typestr, code_vec),
    };
    fn_name_map.insert(name, (range, typestr, code_vec));
  }
  let main_parse_fn_str = make_main_parse_fn_str(main_type_str.clone(), bnfs.clone())?;
  let parse_fn_str = make_parse_fn_str(
    main_type_str.clone(),
    &fn_name_map,
    &token_map,
    bnfs,
  )?;
  Ok(format!("{}\n{}\n", main_parse_fn_str, parse_fn_str))
}

fn make_main_parse_fn_str(
  main_type_str: String,
  bnfs: Vec<types::Bnf>,
) -> Result<String, error::Error> {
  let main_fn_name_opt = bnfs.iter().find(|bnf| match bnf {
    types::Bnf::Pub(_, _, _, _) => true,
    _ => false,
  });
  let (main_fn_name, target_type) = match main_fn_name_opt {
    Some(types::Bnf::Pub(_, s, ty, _)) => Ok((s, ty)),
    _ => Err(error::Error::ConfigError(
      error::ConfigError::NotFoundPubFunctin,
    )),
  }?;
  Ok(format!(
    "#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(unused_parens)]
pub fn parse(tokens: Vec<{}>) -> Result<{}, ParseError> {{
  let (ret, pos) = _parse_fn_{}(&tokens, 0)?;
  if pos == tokens.len() {{
    Ok(ret)
  }} else if pos > tokens.len() {{
    Err(ParseError::Eof)
  }} else {{
    Err(ParseError::RedundantExpression(tokens[pos].clone()))
  }}
}}
",
    main_type_str, target_type, main_fn_name
  ))
}

fn make_parse_fn_str(
  main_type_str: String,
  fn_name_map: &HashMap<&String, (&types::Range, &String, &Vec<types::Code>)>,
  token_map: &HashMap<&String, &String>,
  bnfs: &Vec<types::Bnf>,
) -> Result<String, error::Error> {
  let mut main_s = String::new();
  for v in bnfs {
    let s = match v {
      types::Bnf::Pub(_, name, _, _) => {
        make_parse_fn(main_type_str.clone(), name.to_string(), fn_name_map, token_map)?
      }
      types::Bnf::NonPub(_, name, _, _) => {
        make_parse_fn(main_type_str.clone(), name.to_string(), fn_name_map, token_map)?
      }
    };
    main_s.push_str(&s)
  }
  Ok(main_s)
}

fn make_parse_fn(
  main_type_str: String,
  name: String,
  fn_name_map: &HashMap<&String, (&types::Range, &String, &Vec<types::Code>)>,
  token_map: &HashMap<&String, &String>,
) -> Result<String, error::Error> {
  let (_rng, type_str, code_lst) = match fn_name_map.get(&name) {
    Some((_rng, type_str, code_lst)) => Ok((_rng, type_str, code_lst)),
    None => Err(error::Error::ConfigError(
      error::ConfigError::NotFoundFunctionName(name.clone()),
    )),
  }?;
  let code_type = make_code_type_str(code_lst);
  let nexttoken_to_code_type = make_nexttoken_to_code_type(code_lst, fn_name_map, token_map)?;
  let main_code_str_result = make_main_code_str(code_lst);
  let (main_code_str, err_or_null_code) = match main_code_str_result {
    Ok(code) => (
      code,
      "return Err(ParseError::UnexpectedToken(tokens.iter().next().unwrap().clone()))".to_string(),
    ),
    Err((main_code, null_code)) => (main_code, null_code),
  };
  Ok(format!(
    "
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(unused_parens)]
fn _parse_fn_{}(
  tokens: &Vec<{}>,
  pos: usize,
) -> Result<({}, usize), ParseError>
{{
  let mut _token_pos = pos;
  let token1 = tokens.get(pos);
  {}
  let code_type =
    token1
      .ok_or(ParseError::Eof)
      .and_then(|tok| match tok {{
    {}
      _ => {{Ok(CodeType::Other)}}
      }});
  let main =
  match code_type? {{
    {}
    _ => {{ {} }}
  }};
  Ok((main, _token_pos))
}}
",
    name,
    main_type_str,
    type_str,
    code_type,
    nexttoken_to_code_type,
    main_code_str,
    err_or_null_code
  ))
}

fn make_code_type_str(code_lst: &Vec<types::Code>) -> String {
  let mut toknum_str = String::new();
  let mut toknum = 0;
  for (v, _) in code_lst.iter() {
    if v.len() == 0 {
    } else {
      toknum_str.push_str(&format!("Code{},", toknum));
      toknum = toknum + 1;
    }
  }
  format! {
  "enum CodeType {{
    {}
    Other
  }}", toknum_str}
}

fn make_nexttoken_to_code_type(
  code_lst: &Vec<types::Code>,
  fn_name_map: &HashMap<&String, (&types::Range, &String, &Vec<types::Code>)>,
  token_map: &HashMap<&String, &String>,
) -> Result<String, error::Error> {
  let mut tok_vec = Vec::new();
  for i in 0..code_lst.len() {
    let (v, _) = &code_lst[i];
    let next_tokens_lst = make_next_tokens_lst(&v, fn_name_map, i)?;
    if v.len() == 0 {
    } else {
      tok_vec.push(next_tokens_lst)
    }
  }
  let mut tok_vec_old = tok_vec.concat();
  tok_vec_old.sort();
  let mut tok_vec = Vec::new();
  let mut stack: Vec<Vec<usize>> = Vec::new();
  for i in 0..tok_vec_old.len() {
    let (fn_or_tok, v, i_vec) = &tok_vec_old[i];
    let next_opt = &tok_vec_old.get(i + 1);
    match next_opt {
      None => {
        // 最後なのでリストを回収・結合してまとめてpush
        stack.push(i_vec.clone());
        let i_vec_new = stack.concat();
        tok_vec.push((fn_or_tok, v, i_vec_new));
        stack = Vec::new();
      }
      Some((fn_or_tok_next, _, _)) => {
        if fn_or_tok_next == fn_or_tok {
          // 次のトークンは同じものなのでリストを結合して更新
          stack.push(i_vec.clone())
        } else {
          // 次のトークンは違うものなのでリストを回収・結合してまとめてpush
          stack.push(i_vec.clone());
          let i_vec_new = stack.concat();
          tok_vec.push((fn_or_tok, v, i_vec_new));
          stack = Vec::new();
        }
      }
    }
  }
  let mut toknum_str = String::new();
  for (fn_or_token, tree, i_vec) in tok_vec.iter() {
    match fn_or_token {
      types::FnOrToken::Token(tokname) => {
        let s = match token_map.get(tokname) {
          Some(s) => Ok(s.as_str()),
          None => Err(error::Error::ConfigError(
            error::ConfigError::NotFoundTokenTypeStr(tokname.clone()),
          )),
        }?;
        let string = format!("{} => Ok(CodeType::Code{}),\n", s, i_vec[0]);
        println!("{:?}: {:?}", fn_or_token, tree);
        toknum_str.push_str(&string)
      }
      types::FnOrToken::Function(_) => (),
    }
  }
  Ok(toknum_str)
}

fn make_next_tokens_lst(
  tokens_lst: &Vec<(String, types::FnOrToken)>,
  fn_name_map: &HashMap<&String, (&types::Range, &String, &Vec<types::Code>)>,
  i: usize,
) -> Result<Vec<(types::FnOrToken, Vec<Vec<usize>>, Vec<usize>)>, error::Error> {
  match tokens_lst.iter().next() {
    None => Ok(Vec::new()),
    Some((_, fn_or_token)) => {
      let lst = serch_next_token(
        &vec![(fn_or_token.clone(), vec![vec![i]])],
        fn_name_map,
        vec![vec![i]],
      )?;
      let mut v = Vec::new();
      for (ft, tree) in lst.iter() {
        v.push((ft.clone(), tree.clone(), vec![i]))
      }
      Ok(v)
    }
  }
}

fn serch_next_token(
  fn_or_token_lst: &Vec<(types::FnOrToken, Vec<Vec<usize>>)>,
  fn_name_map: &HashMap<&String, (&types::Range, &String, &Vec<types::Code>)>,
  i_vec: Vec<Vec<usize>>,
) -> Result<Vec<(types::FnOrToken, Vec<Vec<usize>>)>, error::Error> {
  // fn_or_token_lstにserchをmapしてリストを作る
  // それらをconcatして重複を取り除く
  // 長さが元のfn_or_token_lstと変わらなかったらループに入っているので処理を終了し、
  // Tokenだけを残す処理をして、処理後のリストを返す
  // 長さが変化していたら処理が継続中なのでserch_next_tokenに値を入れて再帰化
  let new_fn_or_token_lst_lst_res: Vec<
    Result<Vec<(types::FnOrToken, Vec<Vec<usize>>)>, error::Error>,
  > = fn_or_token_lst
    .iter()
    .map(|(fn_or_token, tree)| serch(fn_or_token, tree, fn_name_map))
    .collect();
  let mut old_fn_or_token_lst = fn_or_token_lst.clone();
  let mut new_fn_or_token_lst_lst = Vec::new();
  // Errが無い限り突っ込んでいく
  // Errがあったらそこで終了
  for v in new_fn_or_token_lst_lst_res.iter() {
    match v {
      Ok(fn_or_token_lst) => new_fn_or_token_lst_lst.push(fn_or_token_lst.clone()),
      Err(e) => return Err(e.clone()),
    }
  }
  let mut new_fn_or_token_lst = new_fn_or_token_lst_lst.concat();
  // 元のリストと結合
  new_fn_or_token_lst.append(&mut old_fn_or_token_lst);
  // sortして揃える
  new_fn_or_token_lst.sort();
  // 次の中身が同じならi_vecをstack
  // 次の中身が違うならstackの中身を取り出す
  new_fn_or_token_lst.dedup();
  // 長さを見る
  if fn_or_token_lst.len() == new_fn_or_token_lst.len() {
    Ok(new_fn_or_token_lst)
  } else {
    serch_next_token(&new_fn_or_token_lst, fn_name_map, i_vec)
  }
}

fn serch(
  fn_or_token: &types::FnOrToken,
  tree: &Vec<Vec<usize>>,
  fn_name_map: &HashMap<&String, (&types::Range, &String, &Vec<types::Code>)>,
) -> Result<Vec<(types::FnOrToken, Vec<Vec<usize>>)>, error::Error> {
  fn get_head(
    lst: &Vec<types::Code>,
    tree: &Vec<Vec<usize>>,
  ) -> Vec<(types::FnOrToken, Vec<Vec<usize>>)> {
    let mut main_vec = Vec::new();
    for (new_fn_or_token_lst, _) in lst.iter() {
      match new_fn_or_token_lst.iter().next() {
        None => (),
        Some((_, fn_or_token)) => main_vec.push((fn_or_token.clone(), tree.to_vec())),
      }
    }
    main_vec
  };
  match fn_or_token {
    types::FnOrToken::Token(_) => Ok(vec![(fn_or_token.clone(), tree.to_vec())]),
    types::FnOrToken::Function(s) => {
      let code_lst = match fn_name_map.get(s) {
        Some((_, _, code_lst)) => Ok(code_lst),
        None => Err(error::Error::ConfigError(
          error::ConfigError::NotFoundFunctionName(s.clone()),
        )),
      }?;
      Ok(get_head(code_lst, tree))
    }
  }
}

// nullが無ければコードを全部結合した文字列を
// nullがあったらnull以外のコードを結合した文字列とnullの場合のコードを返す。
fn make_main_code_str(code_lst: &[types::Code]) -> Result<String, (String, String)> {
  let mut null_code_opt = None;
  let mut code_str = String::new();
  let mut toknum = 0;
  for (fn_or_token_lst, code) in code_lst.iter() {
    if fn_or_token_lst.len() == 0 {
      null_code_opt = Some(code.to_string())
    } else {
      let let_code = make_let_code(fn_or_token_lst);
      code_str.push_str(&format!(
        "CodeType::Code{} => {{
{}
      _token_pos = pos;
{}
        }}",
        toknum, let_code, code
      ));
      toknum = toknum + 1;
    }
  }
  match null_code_opt {
    None => Ok(code_str),
    Some(null_code) => Err((code_str, null_code)),
  }
}

fn make_let_code(fn_or_token_lst: &[(String, types::FnOrToken)]) -> String {
  let mut main_s = String::new();
  for (name, fn_or_token) in fn_or_token_lst.iter() {
    let s = match fn_or_token {
      types::FnOrToken::Function(fn_name) => format!(
        "      let ({}, pos) = _parse_fn_{}(tokens, pos)?;\n",
        name, fn_name
      ),
      types::FnOrToken::Token(tok_name) => format!(
        "      let ({}, pos) = _parse_token_{}(tokens, pos)?;\n",
        name, tok_name
      ),
    };
    main_s.push_str(&s)
  }
  main_s
}
