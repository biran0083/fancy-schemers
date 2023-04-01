use std::{rc::Rc, cell::RefCell};

use crate::{value::{Value, BuiltInFunType, apply_built_in_function}, parser::ParseResult, env::Env};


#[derive(Debug, PartialEq, Eq)]
pub struct EvalError {
    pub msg: String,
}


pub trait Interpreter {
    fn eval(&self, env : Rc<RefCell<Env>>) -> Result<Rc<Value>, EvalError>;
}

impl Interpreter for Value {
    fn eval(&self, env : Rc<RefCell<Env>>) -> Result<Rc<Value>, EvalError> {
        match self {
            Value::Symbol(s) if s == "null" => Ok(Rc::new(Value::Null)),
            Value::Int(_)
            | Value::Bool(_)
            | Value::Null => Ok(Rc::new(self.clone())),
            Value::Symbol(s) => {
                if let Some(v) = env.borrow().get(s.as_str()) {
                    Ok(v)
                } else if let Ok(f) = s.parse::<BuiltInFunType>() {
                    Ok(Rc::new(Value::BuiltInFun(f)))
                } else {
                    Ok(Rc::new(Value::Void))
                }
            },
            Value::Pair(_, _) => {
                fn to_vec(lst : &Value) -> Result<Vec<Rc<Value>>, EvalError> {
                    match lst {
                        Value::Null => Ok(vec![]),
                        Value::Pair(fst, rst) => {
                            let mut res = vec![fst.clone()];
                            res.extend(to_vec(rst.as_ref())?);
                            Ok(res)
                        }
                        _ => Err(EvalError{msg:"failed to convert pair to list".into()})
                    }
                }
                let lst = to_vec(self)?;
                match lst[0].as_ref() {
                    Value::Symbol(s) if s == "define" => {
                        assert_eq!(lst.len(), 3);
                        match lst[1].as_ref() {
                            Value::Symbol(name) => {
                                let value = lst[2].as_ref().eval(env.clone())?;
                                env.as_ref().borrow_mut().set(name.into(), value);
                                Ok(Rc::new(Value::Void))
                            },
                            _ => Err(EvalError{msg:"evaluate define failed".into()})
                        }
                    },
                    Value::Symbol(s) if s == "lambda" => {
                        assert_eq!(lst.len(), 3);
                        let mut params = vec![];
                        for p in to_vec(lst[1].as_ref())? {
                            if let Value::Symbol(name) = p.as_ref() {
                                params.push(name.into());
                            } else {
                                return Err(EvalError{msg: "param is not symbol".into()})
                            }
                        }
                        return Ok(Rc::new(Value::Closure{env: env.clone(), params: params, body: lst[2].clone()}));
                    },
                    Value::Symbol(s) if s == "if" => {
                        assert_eq!(lst.len(), 4);
                        let cond = lst[1].as_ref().eval(env.clone())?;
                        match cond.as_ref() {
                            Value::Bool(true) => lst[2].eval(env.clone()),
                            Value::Bool(false) => lst[3].eval(env.clone()),
                            _ => Err(EvalError{msg: "Condition is not boolean".into()}),
                        }
                    },
                    Value::Symbol(s) if s == "quote" => {
                        assert_eq!(lst.len(), 2);
                        Ok(lst[1].clone())
                    },
                    _ => {
                        let f = lst[0].eval(env.clone())?;
                        match f.as_ref() {
                            Value::BuiltInFun(f) => {
                                let mut params = vec![];
                                for i in 1..lst.len() {
                                    params.push(lst[i].eval(env.clone())?);
                                }
                                return apply_built_in_function(f.clone(), params);
                            },
                            Value::Closure{env: e, body, params} => {
                                if lst.len() != params.len() + 1 {
                                    return Err(EvalError{msg: "param number mismatch".into()});
                                }
                                let mut new_env = Env::new_with_parent(e.clone());
                                for i in 1..lst.len() {
                                    let v = lst[i].eval(env.clone())?;
                                    new_env.set(params[i - 1].clone(), v);
                                }
                                return body.as_ref().eval(Rc::new(RefCell::new(new_env)));
                            },
                            _ => {
                                return Err(EvalError{msg: format!("unknown function type: {:?}", f)})
                            },
                        }
                    },
                }
            },
            _ => Err(EvalError{msg: "invalid ast".into()})
        }
    }
}

impl Interpreter for str {
    fn eval(&self, env : Rc<RefCell<Env>>) -> Result<Rc<Value>, EvalError> {
        let res = self.parse::<ParseResult>()?;
        assert!(res.rest == "");
        let mut result = Rc::new(Value::Void);
        for exp in res.exps {
            result = exp.eval(env.clone())?;
        }
        return Ok(result);
    }
}