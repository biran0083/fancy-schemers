use std::rc::Rc;

use crate::value::Value;
use crate::eval::EvalError;
use crate::lex::{Token, tokenize};

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    pub msg: String,
}

#[derive(Debug, PartialEq)]
pub struct ParseResult<'a> {
    pub exps: Vec<Rc<Value>>,
    pub rest: &'a[Token],
}

impl From<ParseError> for EvalError {
    fn from(err: ParseError) -> Self {
        EvalError{msg: format!("ParseError: {}", err.msg)}
    }
}


fn parse_one(tokens: &[Token]) -> Result<ParseResult, ParseError> {
    assert!(tokens.len() > 0);
    fn to_pairs(lst : &[Rc<Value>], end: &Rc<Value>) -> Rc<Value> {
        if lst.len() == 0 {
            end.clone()
        } else {
            Rc::new(Value::Pair(
                lst[0].clone(),
                to_pairs(&lst[1..], end)))
        }
    }
    match &tokens[0] {
        Token::LeftParen => {
            let mut rem = &tokens[1..];
            let mut exps = vec![];
            let mut end = Rc::new(Value::Null);
            while rem.len() > 0 && rem[0] != Token::RightParen {
                match &rem[0] {
                    Token::Dot => {
                        let res = parse_one(&rem[1..])?;
                        end = res.exps[0].clone();
                        rem = res.rest;
                        assert_eq!(rem[0], Token::RightParen);
                    },
                    _ => {
                        let res = parse_one(rem)?;
                        exps.extend(res.exps);
                        rem = res.rest;
                    },
                }
            }
            if rem.get(0) != Some(&Token::RightParen) {
                Err(ParseError{msg: format!("expect ), got: {:?}", rem.get(0))})
            } else {
                Ok(ParseResult{
                    exps: vec![to_pairs(&exps, &end)],
                    rest: &rem[1..],
                })
            }
        },
        Token::RightParen => Err(ParseError{msg: "unexpected token )".into()}),
        Token::Dot => Err(ParseError{msg: "unexpected token .".into()}),
        Token::Quote => {
            let mut res = parse_one(&tokens[1..])?;
            assert_eq!(res.exps.len(), 1);
            Ok(ParseResult{
                exps: vec![
                    to_pairs(&vec![
                                Rc::new(Value::Symbol("quote".into())),
                                res.exps.pop().unwrap()
                            ],
                            &Rc::new(Value::Null))
                ],
                rest: res.rest,
            })
        },
        Token::Symbol(ref s) if s == "#t" => Ok(ParseResult{
            exps: vec![Rc::new(Value::Bool(true))],
            rest: &tokens[1..],
        }),
        Token::Symbol(ref s) if s == "#f" => Ok(ParseResult{
            exps: vec![Rc::new(Value::Bool(false))],
            rest: &tokens[1..],
        }),
        Token::Symbol(s) => {
            assert!(s != "");
            if let Ok(v) = s.parse() {
                Ok(ParseResult{
                    exps: vec![Rc::new(Value::Int(v))],
                    rest: &tokens[1..],
                })
            } else {
                Ok(ParseResult{
                    exps: vec![Rc::new(Value::Symbol(s.into()))],
                    rest: &tokens[1..],
                })
            }
        },
    }
}

pub fn parse(s: &str) -> Result<ParseResult, ParseError>  {
    if s.len() == 0 {
        return Ok(ParseResult{exps: vec![], rest: &[]});
    }
    let mut tokens : &[Token] = &tokenize(s);
    let mut res = vec![];
    while tokens.len() > 0 {
        let t = parse_one(tokens)?;
        res.extend(t.exps);
        tokens = t.rest;
    }
    return Ok(ParseResult{
        exps: res,
         rest: &[],
    });
}