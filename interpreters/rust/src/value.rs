
use std::{rc::Rc, cell::RefCell};

use crate::eval::EvalError;
use crate::env::Env;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value {
    Void,
    Int(i64),
    BuiltInFun(BuiltInFunType),
    Closure {
        env: Rc<RefCell<Env>>,
        body: Vec<Rc<Value>>,
        params: Rc<Value>,
        is_macro: bool,
    },
    Bool(bool),
    Pair(Rc<Value>, Rc<Value>),
    Null,
    Symbol(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BuiltInFunType {
    Add,
    Eq,
    Sub,
    Mul,
    Cons,
    Car,
    Cdr,
    IsNull,
    Display
}

pub fn apply_built_in_function(f : BuiltInFunType, params : Vec<Rc<Value>>) -> Result<Rc<Value>, EvalError> {
    match f {
        BuiltInFunType::Add => {
            let mut res : i64 = 0;
            for param in params {
                if let Value::Int(v) = param.as_ref() {
                    res += v;
                } else {
                    return Err(EvalError{msg: format!("cannot add non int value: {:?}", param)});
                }
            }
            return Ok(Rc::new(Value::Int(res)));
        },
        BuiltInFunType::Mul => {
            let mut res : i64 = 1;
            for param in params {
                if let Value::Int(v) = param.as_ref() {
                    res *= v;
                } else {
                    return Err(EvalError{msg: format!("cannot add non int value: {:?}", param)});
                }
            }
            return Ok(Rc::new(Value::Int(res)));
        },
        BuiltInFunType::Eq => {
            assert_eq!(params.len(), 2);
            Ok(Rc::new(Value::Bool(params[0].as_ref() == params[1].as_ref())))
        },
        BuiltInFunType::Sub => {
            assert_eq!(params.len(), 2);
            if let Value::Int(a) = params[0].as_ref() {
                if let Value::Int(b) = params[1].as_ref() {
                    return Ok(Rc::new(Value::Int(a - b)));
                }
            }
            return Err(EvalError{msg: format!("- only applies to integers, got {:#?}", params)});
        },
        BuiltInFunType::Cons => {
            assert_eq!(params.len(), 2);
            Ok(Rc::new(Value::Pair(
                params[0].clone(), params[1].clone())))
        },
        BuiltInFunType::Car => {
            assert_eq!(params.len(), 1);
            if let Value::Pair(v, _) = params[0].as_ref() {
                Ok(v.clone())
            } else {
                Err(EvalError{msg: "car only apply to pairs".into()})
            }
        },
        BuiltInFunType::Cdr => {
            assert_eq!(params.len(), 1);
            if let Value::Pair(_, v) = params[0].as_ref(){
                Ok(v.clone())
            } else {
                Err(EvalError{msg: "car only apply to pairs".into()})
            }
        },
        BuiltInFunType::IsNull => {
            assert_eq!(params.len(), 1);
            if let Value::Null = params[0].as_ref() {
                Ok(Rc::new(Value::Bool(true)))
            } else {
                Ok(Rc::new(Value::Bool(false)))
            }
        },
        BuiltInFunType::Display => {
            assert_eq!(params.len(), 1);
            print!("{}", params[0].to_string());
            Ok(Rc::new(Value::Void))
        }
    }
}

// without outer '()
fn list_to_string_inner(value : &Value) -> String {
    match value {
        Value::Null => "".into(),
        Value::Pair(fst, rest) => {
            let t = rest.as_ref();
            match t {
                Value::Null => to_string_helper(fst.as_ref()),
                Value::Pair(_, _) => format!("{} {}", &to_string_helper(fst.as_ref()), &list_to_string_inner(t)),
                _ => format!("{} . {}", &to_string_helper(fst.as_ref()), &to_string_helper(t)),
            }
        },
        _ => panic!("should not reach here"),
    }
}

// list without prefix '
fn to_string_helper(value : &Value) -> String {
    match value {
        Value::Void => "".into(),
        Value::Int(v) => v.to_string(),
        Value::BuiltInFun(f) => format!("{:?}",f),
        Value::Closure{env:_, body:_, params:_, is_macro: _} => "#procedure".into(),
        Value::Bool(v) => if *v {
            "#t".into()
        } else {
            "#f".into()
        },
        Value::Null
        | Value::Pair(_, _) => format!("({})", list_to_string_inner(value)),
        Value::Symbol(s) => s.clone(),
    }
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::Null
            | Value::Pair(_, _) => format!("'{}", to_string_helper(self)),
            _ => to_string_helper(self),
        }
    }
}