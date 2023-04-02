use std::{rc::Rc, cell::RefCell};

use crate::{value::{Value, BuiltInFunType, apply_built_in_function}, env::Env};
use crate::parser::parse;

#[derive(Debug, PartialEq, Eq)]
pub struct EvalError {
    pub msg: String,
}


pub trait Interpreter {
    fn eval(&self, env : Rc<RefCell<Env>>) -> Result<Rc<Value>, EvalError>;
}

fn eval_helper(cur: Rc<Value>, env : Rc<RefCell<Env>>)  -> Result<Rc<Value>, EvalError> {
    let mut cur = cur.clone();
    let mut env = env.clone();
    loop {
        match cur.as_ref() {
            Value::Symbol(s) if s == "null" => {
                return Ok(Rc::new(Value::Null));
            },
            Value::Int(_)
            | Value::Bool(_)
            | Value::Null => {
                return Ok(cur.clone());
            },
            Value::Symbol(s) => {
                if let Some(v) = env.borrow().get(s.as_str()) {
                    return Ok(v);
                } else if let Ok(f) = s.parse::<BuiltInFunType>() {
                    return Ok(Rc::new(Value::BuiltInFun(f)));
                } else {
                    return Ok(Rc::new(Value::Void));
                }
            },
            Value::Pair(_, _) => {
                fn to_vec(lst : &Value) -> Result<Vec<Rc<Value>>, EvalError> {
                    match lst {
                        Value::Null => Ok(vec![]),
                        Value::Pair(fst, rst) => {
                            let mut res = vec![fst.clone()];
                            res.extend(to_vec(rst.as_ref())?);
                            return Ok(res)
                        }
                        _ => {
                            return Err(EvalError{msg:"failed to convert pair to list".into()});
                        }
                    }
                }
                let lst = to_vec(cur.as_ref())?;
                match lst[0].as_ref() {
                    Value::Symbol(s) if s == "define" => {
                        assert_eq!(lst.len(), 3);
                        match lst[1].as_ref() {
                            Value::Symbol(name) => {
                                let value = eval_helper(lst[2].clone(), env.clone())?;
                                env.as_ref().borrow_mut().set(name.into(), value);
                               return Ok(Rc::new(Value::Void));
                            },
                            _ => {
                                return Err(EvalError{msg:"evaluate define failed".into()});
                            }
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
                        let cond = eval_helper(lst[1].clone(), env.clone())?;
                        match cond.as_ref() {
                            Value::Bool(true) => {
                                cur = lst[2].clone();
                                env = env.clone();
                            }
                            Value::Bool(false) => {
                                cur = lst[3].clone();
                                env = env.clone();
                            }
                            _ => {
                                return Err(EvalError{msg: "Condition is not boolean".into()});
                            },
                        };
                    },
                    Value::Symbol(s) if s == "quote" => {
                        assert_eq!(lst.len(), 2);
                        return Ok(lst[1].clone());
                    },
                    _ => {
                        let f = eval_helper(lst[0].clone(), env.clone())?;
                        match f.as_ref() {
                            Value::BuiltInFun(f) => {
                                let mut params = vec![];
                                for i in 1..lst.len() {
                                    params.push(eval_helper(lst[i].clone(), env.clone())?);
                                }
                                return apply_built_in_function(f.clone(), params);
                            },
                            Value::Closure{env: e, body, params} => {
                                if lst.len() != params.len() + 1 {
                                    return Err(EvalError{msg: "param number mismatch".into()});
                                }
                                let mut new_env = Env::new_with_parent(e.clone());
                                for i in 1..lst.len() {
                                    let v = eval_helper(lst[i].clone(), env.clone())?;
                                    new_env.set(params[i - 1].clone(), v);
                                }
                                cur = body.clone();
                                env = Rc::new(RefCell::new(new_env));
                                continue;
                            },
                            _ => {
                                return Err(EvalError{msg: format!("unknown function type: {:?}", f)})
                            },
                        }
                    },
                }
            },
            _ => {
                return Err(EvalError{msg: "invalid ast".into()});
            }
        }
    }
}

impl Interpreter for str {
    fn eval(&self, env : Rc<RefCell<Env>>) -> Result<Rc<Value>, EvalError> {
        let res = parse(self)?;
        assert_eq!(res.rest.len(), 0);
        let mut result = Rc::new(Value::Void);
        for exp in res.exps {
            result = eval_helper(exp.clone(), env.clone())?;
        }
        return Ok(result);
    }
}