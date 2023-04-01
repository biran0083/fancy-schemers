use std::{rc::Rc, str::FromStr};

use crate::value::Value;
use crate::eval::EvalError;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    pub msg: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseResult {
    pub exps: Vec<Rc<Value>>,
    pub rest: String,
}

impl From<ParseError> for EvalError {
    fn from(err: ParseError) -> Self {
        EvalError{msg: format!("ParseError: {}", err.msg)}
    }
}

fn parse_one(s: &str) -> Result<ParseResult, ParseError> {
    assert!(s.len() > 0);
    let ss : Vec<char> = s.chars().collect();
    if ss[0].is_ascii_whitespace() {
        return s.trim_start().parse();
    }
    assert_ne!(ss[0], ')');
    fn to_pairs(lst : &[Rc<Value>]) -> Rc<Value> {
        if lst.len() == 0 {
            Rc::new(Value::Null)
        } else {
            Rc::new(Value::Pair(
                lst[0].clone(),
                to_pairs(&lst[1..])))
        }
    }
    if ss[0] == '\'' {
        let res = parse_one(&s[1..])?;
        return Ok(ParseResult{
            exps: vec![
                to_pairs(&vec![
                    Rc::new(Value::Symbol("quote".into())),
                    res.exps[0].clone()])
            ],
            rest: res.rest,
        });
    } else if ss[0] == '(' {
        let mut cur: String = s[1..].trim_start().into();
        let mut exps = vec![];
        while cur.len() > 0 && cur.chars().next() != Some(')') {
            let res = parse_one(&cur)?;
            exps.extend(res.exps);
            cur = res.rest;
        }
        if cur.chars().next() != Some(')') {
            return Err(ParseError{msg: format!("expect ), got: {}", cur)});
        }
        return Ok(ParseResult{
            exps: vec![to_pairs(&exps)],
            rest: cur[1..].trim_start().into(),
        });
    } else if ss[0].is_ascii_digit() {
        let mut j = 1;
        while j < ss.len() && ss[j].is_ascii_digit() {
            j += 1;
        }
        if j < ss.len() && !ss[j].is_ascii_whitespace() && ss[j] != ')' {
            return Err(ParseError{msg: format!("fail to parse int: {}", s)})
        }
        if let Ok(v) = s[..j].parse() {
            return Ok(ParseResult{
                exps: vec![Rc::new(Value::Int(v))],
                rest: s[j..].trim_start().into(),
            });
        } else {
            return Err(ParseError{msg: format!("fail to parse int: {} ", &s[..j])})
        }
    } else {
        let mut j = 1;
        while j < ss.len() && !ss[j].is_ascii_whitespace() && ss[j] != ')' {
            j += 1;
        }
        let mut res = ParseResult{
            exps: vec![],
            rest: s[j..].trim_start().into(),
        };
        let atom_value = match &s[..j] {
            "#t" => Value::Bool(true),
            "#f" => Value::Bool(false),
            s => Value::Symbol(s.into()),
        };
        res.exps.push(Rc::new(atom_value));
        return Ok(res);
    }
}

impl FromStr for ParseResult {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err>  {
        if s.len() == 0 {
            return Ok(ParseResult{exps: vec![], rest: "".into()});
        }
        let mut res = parse_one(s)?;
        if res.rest.len() > 0 {
            let rem = res.rest.parse::<ParseResult>()?;
            res.exps.extend(rem.exps);
            res.rest = rem.rest;
        }
        return Ok(res);
    }
}