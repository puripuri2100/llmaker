use std::fs::File;
use std::io::prelude::*;
use std::process;
use std::str::from_utf8;

use super::frontend::lexer;
use super::frontend::parse;
use super::types;

#[derive(Debug, Clone)]
pub enum OptionError {
  NoInputFileName,
  NotFoundInputFileName(String),
  BrokenInputFile(String),
  BrokenInputFilePath(String),
}

#[derive(Debug, Clone)]
pub enum ConfigError {
  NotFoundPubFunctin,
  NotFoundTokenTypeStr(String),
  NotFoundFunctionName(String),
}

#[derive(Debug, Clone)]
pub enum Error {
  OptionError(OptionError),
  LexerError(lexer::LexError),
  ParserError(parse::ParseError),
  ConfigError(ConfigError),
}

pub fn print_error_msg(err: Error, input_file_name_opt: Option<&str>) -> () {
  match err {
    Error::OptionError(e) => match e {
      OptionError::NoInputFileName => {
        eprintln! {"![option error]\n  no input file name"}
      }
      OptionError::NotFoundInputFileName(s) => {
        eprintln!("![opiton error]\n  not found input file: {}", s)
      }
      OptionError::BrokenInputFile(s) => eprintln!("![opiton error]\n  broken input file: {}", s),
      OptionError::BrokenInputFilePath(s) => {
        eprintln!("![opiton error]\n  broken input file path: {}", s)
      }
    },
    Error::LexerError(e) => {
      // OptionErrorではないので、ファイルを読みこむことができることは保障されている。
      let mut f = File::open(&mut input_file_name_opt.unwrap()).unwrap();
      let mut contents = String::new();
      f.read_to_string(&mut contents).unwrap();
      let input_bytes = contents.as_bytes();
      let (e, rng) = e;
      match e {
        lexer::LexErrorKind::InvalidChar(c) => {
          let (err_point_s, start_pos, _end_pos) = get_error_point(rng, input_bytes);
          let input_file_path = input_file_name_opt.unwrap();
          let (start_row, start_column) = start_pos;
          let start_pos_s = format!("{}:{}", start_row, start_column);
          eprintln!(
            "![lexer error]\n  illegal token '{}' at {}:{}\n{}",
            c, input_file_path, start_pos_s, err_point_s
          )
        }
        lexer::LexErrorKind::UnDefinedToken(s) => {
          let (err_point_s, start_pos, _end_pos) = get_error_point(rng, input_bytes);
          let input_file_path = input_file_name_opt.unwrap();
          let (start_row, start_column) = start_pos;
          let start_pos_s = format!("{}:{}", start_row, start_column);
          eprintln!(
            "![lexer error]\n  undefinde token \"{}\" at {}:{}\n{}",
            s, input_file_path, start_pos_s, err_point_s
          )
        }
        lexer::LexErrorKind::Eof => {
          let (_, start_pos, _end_pos) = get_error_point(rng, input_bytes);
          let input_file_path = input_file_name_opt.unwrap();
          let (start_row, start_column) = start_pos;
          let start_pos_s = format!("{}:{}", start_row, start_column);
          eprintln!(
            "![lexer error]\n unexpected end of file at {}:{}",
            input_file_path, start_pos_s
          )
        }
      }
    }
    Error::ParserError(e) => {
      // OptionErrorではないので、ファイルを読みこむことができることは保障されている。
      let mut f = File::open(&mut input_file_name_opt.unwrap()).unwrap();
      let mut contents = String::new();
      f.read_to_string(&mut contents).unwrap();
      let input_bytes = contents.as_bytes();
      match e {
        parse::ParseError::UnexpectedToken(tok) => {
          let (_errkind, rng) = tok;
          let (err_point_s, start_pos, _end_pos) = get_error_point(rng, input_bytes);
          let input_file_path = input_file_name_opt.unwrap();
          let (start_row, start_column) = start_pos;
          let start_pos_s = format!("{}:{}", start_row, start_column);
          eprintln!(
            "![parse error]\n  unexpected token '{}' at {}:{}",
            err_point_s, input_file_path, start_pos_s
          )
        }
        parse::ParseError::RedundantExpression(tok) => {
          let (_errkind, rng) = tok;
          let (err_point_s, start_pos, _end_pos) = get_error_point(rng, input_bytes);
          let input_file_path = input_file_name_opt.unwrap();
          let (start_row, start_column) = start_pos;
          let start_pos_s = format!("{}:{}", start_row, start_column);
          eprintln!(
            "![parse error]\n  redundant expression '{}' at {}:{}",
            err_point_s, input_file_path, start_pos_s
          )
        }
        parse::ParseError::Eof => {
          let input_file_path = input_file_name_opt.unwrap();
          eprintln!(
            "![parse error]\n  unexpected end of file at {}",
            input_file_path
          )
        }
      }
    }
    Error::ConfigError(e) => match e {
      ConfigError::NotFoundPubFunctin => {
        let input_file_path = input_file_name_opt.unwrap();
        eprintln!(
          "![config file error]\n not found pub function at {}",
          input_file_path
        )
      }
      ConfigError::NotFoundTokenTypeStr(s) => {
        let input_file_path = input_file_name_opt.unwrap();
        eprintln!(
          "![config file error]\n not found \"{}\"'s type at {}",
          s, input_file_path
        )
      }
      ConfigError::NotFoundFunctionName(s) => {
        let input_file_path = input_file_name_opt.unwrap();
        eprintln!(
          "![config file error]\n not found \"{}\"'s name at {}",
          s, input_file_path
        )
      }
    },
  };
  process::exit(1);
}

fn get_error_point(
  range: types::Range,
  input_bytes: &[u8],
) -> (String, (usize, usize), (usize, usize)) {
  let (start, end) = range.to_tuple();
  let error_point_string = from_utf8(&input_bytes[start..end]).unwrap().to_string();
  let mut start_pos = (1, 0);
  let mut end_pos = (1, 0);
  let mut pos = 0;
  let mut line = 1;
  let mut is_start_pos_update = false;
  let split_input_bytes = input_bytes.split(|b| b == &b'\n');
  for b_vec in split_input_bytes {
    let new_pos = pos + b_vec.len() + 1;// splitするときに\nが削除されるので
    if new_pos > start {
      if is_start_pos_update {
        // 既に更新されているのでstart_posに関してはスルー
        // endより大きいか見る
        if new_pos > end {
          // end_posを更新して終了 見かけ上、'\n'分だけ文字が増えているので補正
          end_pos = (line, end - pos);
          break;
        } else {
          // 行を1つ増やして、posも更新する
          line += 1;
          pos = new_pos;
        }
      } else {
        // startはまだ更新されていないので更新する
        is_start_pos_update = true;
        start_pos = (line, start - pos + 1);
        // endより大きいか見る
        if new_pos > end {
          // end_posを更新して終了
          end_pos = (line, end - pos);
          break;
        } else {
          // 行を1つ増やして、posも更新する
          line += 1;
          pos = new_pos;
        }
      }
    } else {
      line += 1;
      pos = new_pos;
    }
  }
  (error_point_string, start_pos, end_pos)
}

#[test]
fn check_get_error_point() {
  assert_eq!(
    get_error_point(types::Range::make(1, 2), b"abcd\nhoge\n"),
    ("bc".to_string(), (1, 2), (1, 3))
  );
  assert_eq!(
    get_error_point(types::Range::make(1, 5), b"abcd\nhoge\n"),
    ("bcd\nh".to_string(), (1, 2), (2, 1))
  );
}
