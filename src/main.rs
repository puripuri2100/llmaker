use clap::{App, Arg};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub mod backend;
pub mod error;
pub mod frontend;
pub mod types;

fn print_line() {
  println!(" --- --- ---");
}

fn print_msg(s: &str) {
  println!("  {}", s);
}

fn write_file(file_name: String, text: String) {
  let mut file = File::create(file_name).unwrap();
  file.write_all(text.as_bytes()).unwrap();
}

fn sub(
  input_file_name_opt: Option<&str>,
  output_file_name_opt: Option<&str>,
) -> Result<(), error::Error> {
  let mut input_file_name = match input_file_name_opt {
    Some(s) => Ok(s),
    None => Err(error::Error::OptionError(
      error::OptionError::NoInputFileName,
    )),
  }?;
  let output_file_name = match output_file_name_opt {
    Some(path) => path.to_string(),
    None => {
      let path = Path::new(input_file_name);
      let parent_opt = path.parent();
      let file_steam_opt = path.file_stem();
      let new_path = match (parent_opt, file_steam_opt) {
        (Some(parent), Some(file_steam)) => Ok(parent.join(file_steam)),
        (None, Some(file_steam)) => Ok((Path::new("")).join(file_steam)),
        _ => Err(error::Error::OptionError(
          error::OptionError::BrokenInputFilePath(input_file_name.to_string()),
        )),
      }?;
      format!("{}.rs", new_path.to_str().unwrap().to_string())
    }
  };
  print_line();
  print_msg(&format!("target file: '{}'", output_file_name));
  print_line();
  print_msg(&format!("parsing '{}' ...", input_file_name));
  let mut f = match File::open(&mut input_file_name) {
    Ok(v) => Ok(v),
    Err(_) => Err(error::Error::OptionError(
      error::OptionError::NotFoundInputFileName(input_file_name.to_string()),
    )),
  }?;
  let mut contents = String::new();
  match f.read_to_string(&mut contents) {
    Ok(_) => Ok(()),
    Err(_) => Err(error::Error::OptionError(
      error::OptionError::BrokenInputFile(input_file_name.to_string()),
    )),
  }?;
  let ast = frontend::get_ast(&contents)?;
  print_msg("dune.");
  print_line();
  print_msg("making texts ...");
  let output_str = backend::to_string(ast)?;
  print_msg("dune.");
  print_line();
  write_file(output_file_name.clone(), output_str);
  print_msg(&format!("output written on '{}'", output_file_name));
  Ok(())
}

fn main() {
  let app = App::new("llmaker")
    .version("0.0.1")
    .arg(
      Arg::with_name("input")
        .help("Specify input file")
        .value_name("FILE")
        .takes_value(true),
    )
    .arg(
      Arg::with_name("output")
        .help("Specify output file")
        .value_name("FILE")
        .short("o")
        .long("output")
        .takes_value(true),
    );
  let matches = app.get_matches();
  let input_file_name_opt = matches.value_of("input");
  let output_file_name_opt = matches.value_of("output");
  match sub(input_file_name_opt, output_file_name_opt) {
    Ok(()) => (),
    Err(e) => error::print_error_msg(e, input_file_name_opt),
  }
}
