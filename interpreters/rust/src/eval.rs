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

fn bind(env : Rc<RefCell<Env>>, params: Rc<Value>, exps: &[Rc<Value>], mut new_env:  Env) -> Result<Rc<RefCell<Env>>, EvalError> {
    let mut value = Rc::new(Value::Null);
    for exp in exps.iter().rev() {
        let v = eval_helper(exp.clone(), env.clone())?;
        value = Rc::new(Value::Pair(v, value));
    }
    fn helper(params: Rc<Value>, values: Rc<Value>, env : &mut Env) -> Result<(), EvalError> {
        match params.as_ref() {
            Value::Null => {
                match values.as_ref() {
                    Value::Null => Ok(()),
                    _ => Err(EvalError{msg: "binding failed".into()}),
                } 
            },
            Value::Pair(fst, rst) => {
                match values.as_ref() {
                    Value::Pair(fst_value, rst_value) => {
                        helper(fst.clone(), fst_value.clone(), env)?;
                        helper(rst.clone(), rst_value.clone(), env)?;
                        Ok(())
                    }
                    _ => Err(EvalError{msg: "binding failed".into()}),
                } 
            },
            Value::Symbol(s) => {
                env.set(s.into(), values.clone());
                Ok(())
            },
            _ => Err(EvalError{msg: "binding failed".into()}),
        }
    }
    helper(params.clone(), value.clone(), &mut new_env)?;
    Ok(Rc::new(RefCell::new(new_env)))
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
                        return Ok(Rc::new(Value::Closure{env: env.clone(), params: lst[1].clone(), body: lst[2].clone()}));
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
                                env = bind(env.clone(), params.clone(), &lst[1..], Env::new_with_parent(e.clone()))?;
                                cur = body.clone();
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