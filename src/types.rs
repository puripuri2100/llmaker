use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct Range(usize, usize);

impl Range {
  pub fn merge(&self, other: &Range) -> Range {
    use std::cmp::{max, min};
    Range(min(self.0, other.0), max(self.1, other.1))
  }
  pub fn unite(v1: Range, v2: Range) -> Range {
    v1.merge(&v2)
  }
  pub fn dummy() -> Range {
    Range(0, 0)
  }
  pub fn make(start: usize, size: usize) -> Range {
    Range(start, start + size)
  }
  pub fn make_start_end(start: usize, end: usize) -> Range {
    Range(start, end)
  }
  pub fn to_tuple(&self) -> (usize, usize) {
    let Range(start, end) = self;
    if start > end {
      (*end, *start)
    } else {
      (*start, *end)
    }
  }
}

pub type Term = (Head, Setting, Bnfs);

pub type Head = Vec<(Range, String)>;

pub type Setting = (String, Vec<(Range, String, TypeStr)>);

pub type TypeStr = String;

pub type Bnfs = Vec<Bnf>;

#[derive(Debug, Clone)]
pub enum Bnf {
  Pub(Range, String, TypeStr, Vec<Code>),
  NonPub(Range, String, TypeStr, Vec<Code>),
}

pub type Code = (Vec<(String, FnOrToken)>, String);

#[derive(Debug, Clone)]
pub enum FnOrToken {
  Function(String),
  Token(String),
}
impl Ord for FnOrToken {
  fn cmp(&self, other: &Self) -> Ordering {
    match (self, other) {
      (&FnOrToken::Function(ref s1), &FnOrToken::Function(ref s2)) => s1.cmp(&s2),
      (&FnOrToken::Token(ref s1), &FnOrToken::Token(ref s2)) => s1.cmp(&s2),
      (&FnOrToken::Function(_), &FnOrToken::Token(_)) => Ordering::Less,
      (&FnOrToken::Token(_), &FnOrToken::Function(_)) => Ordering::Greater,
    }
  }
}

impl PartialOrd for FnOrToken {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Eq for FnOrToken {}

impl PartialEq for FnOrToken {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (&FnOrToken::Function(ref s1), &FnOrToken::Function(ref s2)) => s1 == s2,
      (&FnOrToken::Token(ref s1), &FnOrToken::Token(ref s2)) => s1 == s2,
      _ => false,
    }
  }
}
