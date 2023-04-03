use std::{rc::Rc, cell::RefCell};
use crate::{value::{Value, apply_built_in_function}, env::Env};
use crate::parser::parse;

#[derive(Debug, PartialEq, Eq)]
pub struct EvalError {
    pub msg: String,
}


pub trait Interpreter {
    fn eval(&self, env : Rc<RefCell<Env>>) -> Result<Rc<Value>, EvalError>;
}

fn bind_helper(params: Rc<Value>, values: Rc<Value>, env : &Rc<RefCell<Env>>) -> Result<(), EvalError> {
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
                    bind_helper(fst.clone(), fst_value.clone(), env)?;
                    bind_helper(rst.clone(), rst_value.clone(), env)?;
                    Ok(())
                }
                _ => Err(EvalError{msg: "binding failed".into()}),
            } 
        },
        Value::Symbol(s) => {
            env.as_ref().borrow_mut().set(s.into(), values.clone());
            Ok(())
        },
        _ => Err(EvalError{msg: "binding failed".into()}),
    }
}

fn bind_for_macro(params: Rc<Value>, exps: &[Rc<Value>], new_env: Rc<RefCell<Env>>) -> Result<Rc<RefCell<Env>>, EvalError> {
    let mut value = Rc::new(Value::Null);
    for exp in exps.iter().rev() {
        value = Rc::new(Value::Pair(exp.clone(), value));
    }
    bind_helper(params.clone(), value.clone(), &new_env)?;
    Ok(new_env)
}

// env: environment to evaluate exps in
// new_envs: environment to store bindings
fn bind_for_function(env : Rc<RefCell<Env>>, params: Rc<Value>, exps: &[Rc<Value>], new_env: Rc<RefCell<Env>>) -> Result<Rc<RefCell<Env>>, EvalError> {
    let mut value = Rc::new(Value::Null);
    for exp in exps.iter().rev() {
        let v = eval_helper(exp.clone(), env.clone())?;
        value = Rc::new(Value::Pair(v, value));
    }
    bind_helper(params.clone(), value.clone(), &new_env)?;
    Ok(new_env)
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
                    Value::Symbol(s) if s == "define" || s == "defmacro"=> {
                        match lst[1].as_ref() {
                            Value::Symbol(name) => {
                                let value = eval_helper(lst[2].clone(), env.clone())?;
                                env.as_ref().borrow_mut().set(name.into(), value);
                               return Ok(Rc::new(Value::Void));
                            },
                            Value::Pair(fst, rst) => {
                                match fst.as_ref() {
                                    Value::Symbol(name) => {
                                        let value = Rc::new(Value::Closure{
                                            env: env.clone(),
                                            params: rst.clone(), 
                                            body: lst[2..].iter().map(Rc::clone).collect(),
                                            is_macro: s == "defmacro",
                                        });
                                        env.as_ref().borrow_mut().set(name.into(), value);
                                        return Ok(Rc::new(Value::Void));
                                    },
                                    _ => {
                                        return Err(EvalError{msg:"illegal define syntax".into()});
                                    }
                                }
                            },
                            _ => {
                                return Err(EvalError{msg:"evaluate define failed".into()});
                            }
                        }
                    },
                    Value::Symbol(s) if s == "lambda" => {
                        return Ok(Rc::new(Value::Closure{
                            env: env.clone(), 
                            params: lst[1].clone(), 
                            body: lst[2..].iter().map(Rc::clone).collect(),
                            is_macro: false,
                        }));
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
                    Value::Symbol(s) if s == "quasiquote" => {
                        assert_eq!(lst.len(), 2);
                        fn quasiquote_helper(exp : &Rc<Value>, env : &Rc<RefCell<Env>>) -> Result<Rc<Value>, EvalError> {
                            match exp.as_ref() {
                                Value:: Pair(fst, rst) => 
                                    match (fst.as_ref(), rst.as_ref()) {
                                        (Value::Symbol(s), Value::Pair(body, _)) 
                                        if s == "unquote" => 
                                            eval_helper(body.clone(), env.clone()),
                                        _ => Ok(Rc::new(Value::Pair(quasiquote_helper(fst, env)?, 
                                            quasiquote_helper(rst, env)?))),
                                    },
                                _ => Ok(exp.clone()),
                            }
                        }
                        return Ok(quasiquote_helper(&lst[1], &env)?);
                    },
                    Value::Symbol(s) if s == "unquote" => {
                        return Err(EvalError{msg: "unquote should only happen inside quasiquote".into()});
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
                            Value::Closure{
                                env: e, 
                                body, 
                                params, 
                                is_macro,
                            } => {
                                assert!(body.len() > 0);
                                if *is_macro {
                                    let new_env = bind_for_macro(params.clone(), &lst[1..], Env::new_with_parent(e.clone()))?;
                                    for i in 0..body.len() - 1 {
                                        eval_helper(body[i].clone(), new_env.clone())?;
                                    }
                                    cur = eval_helper(body.last().unwrap().clone(), new_env.clone())?;
                                } else {
                                    env = bind_for_function(env.clone(), params.clone(), &lst[1..], Env::new_with_parent(e.clone()))?;
                                    for i in 0..body.len() - 1 {
                                        eval_helper(body[i].clone(), env.clone())?;
                                    }
                                    cur = body.last().unwrap().clone();
                                }
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
                return Err(EvalError{msg: format!("invalid ast: {}", cur.as_ref().to_string())});
            }
        }
    }
}

impl Interpreter for str {
    fn eval(&self, env : Rc<RefCell<Env>>) -> Result<Rc<Value>, EvalError> {
        let res = parse(self)?;
        assert_eq!(res.rest.len(), 0);
        let mut result = Rc::new(Value::Void);
        for exp in &res.exps {
            result = eval_helper(exp.clone(), env.clone())?;
        }
        return Ok(result);
    }
}