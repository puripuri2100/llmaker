use super::types;

// トークン
#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum TokenKind {
  EOF,
  GRAMMAR,
  EXTERN,
  ENUM,
  PUB,
  VAR(String),
  CONSTRUCTOR(String),
  LCURLYBRACES,
  RCURLYBRACES,
  EQ,
  COMMA,
  SEMICOLON,
  COLON,
  LBRACES,
  RBRACES,
  ARROW,
  STR(String),
}

// 位置情報とのタプルで表す
pub type Token = (TokenKind, types::Range);

pub fn get_string(tok: TokenKind) -> Option<String> {
  match tok {
    TokenKind::STR(s) => Some(s),
    TokenKind::VAR(s) => Some(s),
    TokenKind::CONSTRUCTOR(s) => Some(s),
    _ => None,
  }
}

// エラー情報の実装
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LexErrorKind {
  InvalidChar(char),
  UnDefinedToken(String),
  Eof,
}

pub type LexError = (LexErrorKind, types::Range);

fn error_invalid_char(c: char, r: types::Range) -> LexError {
  (LexErrorKind::InvalidChar(c), r)
}

fn error_undefined_token(s: String, r: types::Range) -> LexError {
  (LexErrorKind::UnDefinedToken(s), r)
}

fn error_eof(r: types::Range) -> LexError {
  (LexErrorKind::Eof, r)
}

const DIGIT: &[u8] = b"1234567890";
const CAPITAL: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const SMALL: &[u8] = b"abcdefghijklmnopqrstuvwxyz";

// lexer関数
#[allow(unused_assignments)]
pub fn lex(input: &str) -> Result<Vec<Token>, LexError> {
  let mut tokens = Vec::new();
  let input = input.as_bytes();

  // 位置情報
  let mut pos = 0;
  // posを更新するマクロ
  macro_rules! lex_a_token {
    ($lexer:expr) => {{
      let (tok, p) = $lexer;
      tokens.push(tok);
      pos = p;
    }};
  }
  // tokenとposのリストを更新するマクロ
  macro_rules! lex_token_list {
    ($tokenlst:expr) => {{
      let (lst, p) = $tokenlst?;
      let mut mut_lst = lst;
      tokens.append(&mut mut_lst);
      pos = p;
    }};
  }

  lex_token_list!(lex_program(input, pos));
  lex_a_token!(lex_eof(pos));
  Ok(tokens)
}
#[test]
fn check_lex() {
  assert_eq!(
    lex("\"hoge\" grammar; hoge:fuga"),
    Ok(vec![
      (TokenKind::STR("hoge".to_string()), types::Range::make(0, 6)),
      (TokenKind::GRAMMAR, types::Range::make(7, 7)),
      (TokenKind::SEMICOLON, types::Range::make(14, 1)),
      (
        TokenKind::VAR("hoge".to_string()),
        types::Range::make(16, 4)
      ),
      (TokenKind::COLON, types::Range::make(20, 1)),
      (
        TokenKind::VAR("fuga".to_string()),
        types::Range::make(21, 4)
      ),
      (TokenKind::EOF, types::Range::make(25, 1)),
    ])
  );
}

// デフォルト
fn lex_program(input: &[u8], pos: usize) -> Result<(Vec<Token>, usize), LexError> {
  let mut tokens = Vec::new();
  let mut pos = pos;

  // posを更新するマクロ
  macro_rules! lex_a_token {
    ($lexer:expr) => {{
      let (tok, p) = $lexer;
      tokens.push(tok);
      pos = p;
    }};
  }

  macro_rules! lex_a_token_result {
    ($lexer:expr) => {{
      let (tok, p) = $lexer?;
      tokens.push(tok);
      pos = p;
    }};
  }

  while pos < input.len() {
    match input[pos] {
      // 次の文字も'/'ならコメント
      // そうでないならerror_invalid_char
      b'/' => match input.get(pos + 1) {
        None => return Err(error_eof(types::Range::make(pos, 1))),
        Some(b) => {
          if b == &b'/' {
            let ((), p) = lex_comment(input, pos + 1);
            pos = p;
          } else {
            return Err(error_undefined_token(
              "/".to_string(),
              types::Range::make(pos, 1),
            ));
          }
        }
      },
      b'{' => {
        lex_a_token!(lex_lcurlybraces(pos));
      }
      b'}' => {
        lex_a_token!(lex_rcurlybraces(pos));
      }
      b'<' => {
        lex_a_token!(lex_lbraces(pos));
      }
      b'>' => {
        lex_a_token!(lex_rbraces(pos));
      }
      b';' => {
        lex_a_token!(lex_semicolon(pos));
      }
      b'"' => lex_a_token_result!(lex_literal(input, pos)),

      // アルファベットと数字と'_'が続く限りvarへ
      b'_' => lex_a_token!(lex_identifier(input, pos)),

      b':' => {
        lex_a_token!(lex_colon(pos));
      }
      b',' => {
        lex_a_token!(lex_comma(pos));
      }

      // 次の文字が'>'ならarrow
      // そうでないならeq
      b'=' => match input.get(pos + 1) {
        Some(v) => {
          if v == &b'>' {
            lex_a_token!(lex_arrow(pos))
          } else {
            lex_a_token!(lex_eq(pos))
          }
        }
        None => lex_a_token!(lex_eq(pos)),
      },

      // identifier = (small (digit | latin | "_")*)
      // 登録してあった文字列以外はvarに
      b'a'..=b'z' => lex_a_token!(lex_identifier(input, pos)),

      // constructor = (capital (digit | latin | "_")*)
      b'A'..=b'Z' => lex_a_token!(lex_constructor(input, pos)),

      b' ' | b'\n' | b'\t' => {
        let ((), p) = skip_spaces(input, pos);
        pos = p;
      }
      b => return Err(error_invalid_char(b as char, types::Range::make(pos, 1))),
    }
  }
  Ok((tokens, pos))
}
#[test]
fn check_lex_program() {
  assert_eq!(
    lex_program(b"\"hoge\" grammar;", 0),
    Ok((
      vec![
        (TokenKind::STR("hoge".to_string()), types::Range::make(0, 6)),
        (TokenKind::GRAMMAR, types::Range::make(7, 7)),
        (TokenKind::SEMICOLON, types::Range::make(14, 1)),
      ],
      15
    ))
  );
}

// 次の文字もそのトークンだったらposを1つ移動させる関数
fn recognize_many(input: &[u8], mut pos: usize, mut f: impl FnMut(u8) -> bool) -> usize {
  while pos < input.len() && f(input[pos]) {
    pos += 1;
  }
  pos
}

// 空白等を無視する
fn skip_spaces(input: &[u8], pos: usize) -> ((), usize) {
  let pos = recognize_many(input, pos, |b| b" \n\t".contains(&b));
  ((), pos)
}

// 改行文字が来るまで読み飛ばす
fn lex_comment(input: &[u8], mut pos: usize) -> ((), usize) {
  while pos < input.len() && (!(b"\n".contains(&input[pos]))) {
    pos += 1;
  }
  ((), pos)
}

//identifier = (small (digit | latin | "_")*)
//続くところまで取得
fn lex_identifier(input: &[u8], pos: usize) -> (Token, usize) {
  use std::str::from_utf8;
  let start = pos;
  let end_pos = recognize_many(input, pos + 1, |b| {
    SMALL.contains(&b) || CAPITAL.contains(&b) || DIGIT.contains(&b) || b == b'_'
  });
  let v_str = from_utf8(&input[start..end_pos]).unwrap();
  let v_string = v_str.to_string();
  match v_str {
    "grammar" => (
      (
        TokenKind::GRAMMAR,
        types::Range::make_start_end(start, end_pos),
      ),
      end_pos,
    ),
    "extern" => (
      (
        TokenKind::EXTERN,
        types::Range::make_start_end(start, end_pos),
      ),
      end_pos,
    ),
    "enum" => (
      (
        TokenKind::ENUM,
        types::Range::make_start_end(start, end_pos),
      ),
      end_pos,
    ),
    "pub" => (
      (TokenKind::PUB, types::Range::make_start_end(start, end_pos)),
      end_pos,
    ),
    _ => (
      (
        TokenKind::VAR(v_string),
        types::Range::make_start_end(start, end_pos),
      ),
      end_pos,
    ),
  }
}

// constructor = (capital (digit | latin | "_")*)
// constructorが続くところまで取得
fn lex_constructor(input: &[u8], pos: usize) -> (Token, usize) {
  use std::str::from_utf8;
  let start = pos;
  let end_pos = recognize_many(input, pos + 1, |b| {
    SMALL.contains(&b) || CAPITAL.contains(&b) || DIGIT.contains(&b) || b == b'_'
  });
  let v_string = from_utf8(&input[start..end_pos]).unwrap().to_string();
  (
    (
      TokenKind::CONSTRUCTOR(v_string),
      types::Range::make_start_end(start, end_pos),
    ),
    end_pos,
  )
}

// 文字列取得
// '"'で終了
// '\'が来たら次の文字を読み、'"'だったら'"'を追加
// そうでなかったら'\'を追加
fn lex_literal(input: &[u8], pos: usize) -> Result<(Token, usize), LexError> {
  let mut v = Vec::new();
  let start = pos + 1;
  let mut s_pos = start;
  while s_pos < input.len() {
    match input.get(s_pos) {
      None => return Err(error_eof(types::Range::make(s_pos, 1))),
      Some(b) => match b {
        b'\\' => match input.get(s_pos + 1) {
          None => {
            s_pos += 1;
            v.push(*b)
          }
          Some(b_next) => {
            if b_next == &b'"' {
              s_pos += 2;
              v.push(b'"')
            } else {
              s_pos += 1;
              v.push(*b)
            }
          }
        },
        b'"' => {
          break;
        }
        _ => {
          s_pos += 1;
          v.push(*b)
        }
      },
    }
  }
  let end = s_pos;
  let v_string = String::from_utf8(v).unwrap();
  Ok((
    (
      TokenKind::STR(v_string),
      types::Range::make_start_end(start - 1, end + 1),
    ),
    end + 1,
  ))
}

#[test]
fn check_lex_literal() {
  assert_eq!(
    lex_literal(b"\"hoge\\\"fuga\"", 0),
    Ok((
      (
        TokenKind::STR("hoge\"fuga".to_string()),
        types::Range::make(0, 12)
      ),
      12
    ))
  )
}

//EOF,
//LCURLYBRACES,
//RCURLYBRACES,
//EQ,
//COMMA,
//SEMICOLON,
//COLON,
//LBRACES,
//RBRACES,
//ARROW,
//STR(String),

fn lex_eof(pos: usize) -> (Token, usize) {
  ((TokenKind::EOF, types::Range::make(pos, 1)), pos + 1)
}

fn lex_lcurlybraces(pos: usize) -> (Token, usize) {
  (
    (TokenKind::LCURLYBRACES, types::Range::make(pos, 1)),
    pos + 1,
  )
}

fn lex_rcurlybraces(pos: usize) -> (Token, usize) {
  (
    (TokenKind::RCURLYBRACES, types::Range::make(pos, 1)),
    pos + 1,
  )
}

fn lex_eq(pos: usize) -> (Token, usize) {
  ((TokenKind::EQ, types::Range::make(pos, 2)), pos + 2)
}

fn lex_comma(pos: usize) -> (Token, usize) {
  ((TokenKind::COMMA, types::Range::make(pos, 1)), pos + 1)
}

fn lex_semicolon(pos: usize) -> (Token, usize) {
  ((TokenKind::SEMICOLON, types::Range::make(pos, 1)), pos + 1)
}

fn lex_colon(pos: usize) -> (Token, usize) {
  ((TokenKind::COLON, types::Range::make(pos, 1)), pos + 1)
}

fn lex_lbraces(pos: usize) -> (Token, usize) {
  ((TokenKind::LBRACES, types::Range::make(pos, 1)), pos + 1)
}

fn lex_rbraces(pos: usize) -> (Token, usize) {
  ((TokenKind::RBRACES, types::Range::make(pos, 1)), pos + 1)
}

fn lex_arrow(pos: usize) -> (Token, usize) {
  ((TokenKind::ARROW, types::Range::make(pos, 2)), pos + 2)
}
