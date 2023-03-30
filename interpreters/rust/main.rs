use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
struct ParseError {
    msg: String,
}

#[derive(Debug, PartialEq, Eq)]
struct ParseResult {
    exps: Vec<Rc<Value>>,
    rest: String,
}

#[derive(Debug, PartialEq, Eq)]
struct EvalError {
    msg: String,
}

#[derive(Debug, PartialEq, Eq, Default)]
struct Env {
    map: HashMap<String, Rc<Value>>,
    parent: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub fn get(&self, key : &str) -> Option<Rc<Value>> {
        if let Some(v) = self.map.get(key) {
            println!("lookup succeed {} {:p} ", key, self);
            Some(v.clone())
        } else {
            if let Some (v) = self.parent.as_ref().and_then(|e| e.borrow().get(key)) {
                Some(v.clone())
            } else {
                println!("lookup failed {} {:p} ", key, self);
                None
            }
        }
    }

    pub fn set(&mut self, key : String, value: Rc<Value>) {
        println!("set {} {:p} ", key, self);
        self.map.insert(key, value);
    }

    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_with_parent(parent: Rc<RefCell<Env>>) -> Self {
        Env{map: HashMap::new(), parent: Some(parent)}
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Value {
    Void,
    Int(i64),
    BuiltInFun(BuiltInFunType),
    Closure {
        env: Rc<RefCell<Env>>,
        body: Rc<Value>,
        params: Vec<String>
    },
    Bool(bool),
    Pair(Rc<Value>, Rc<Value>),
    Null,
    Symbol(String),
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
        Value::Void => "#void".into(),
        Value::Int(v) => v.to_string(),
        Value::BuiltInFun(f) => format!("{:?}",f),
        Value::Closure{env:_, body:_, params:_} => "#procedure".into(),
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

trait Interpreter {
    fn eval(&self, env : Rc<RefCell<Env>>) -> Result<Rc<Value>, EvalError>;
}


#[derive(Debug, PartialEq, Eq, Clone)]
enum BuiltInFunType {
    Add,
    Eq,
    Sub,
    Mul,
    Cons,
    Car,
    Cdr,
    IsNull,
}

impl FromStr for BuiltInFunType {

    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err>  {
        match s {
            "+" => Ok(BuiltInFunType::Add),
            "-" => Ok(BuiltInFunType::Sub),
            "*" => Ok(BuiltInFunType::Mul),
            "eq?" => Ok(BuiltInFunType::Eq),
            "cons" => Ok(BuiltInFunType::Cons),
            "car" => Ok(BuiltInFunType::Car),
            "cdr" => Ok(BuiltInFunType::Cdr),
            "null?" => Ok(BuiltInFunType::IsNull),
            _ => Err(()),
        }
    }
}

fn apply_built_in_function(f : BuiltInFunType, params : Vec<Rc<Value>>) -> Result<Rc<Value>, EvalError> {
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
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval() {
        let env = Rc::new(RefCell::new(Env::new()));
        let v = "123".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(123));
        let v = "(define a 123) a".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(123));
        let v = "(+ 1 2)".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(3));
        let v = "(define three (+ 1 2)) three".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(3));
        let v = "(define three (+ 1 2)) (+ three (+ 3 4 5))".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(15));
    }

    #[test]
    fn test_lambda() {
        let env = Rc::new(RefCell::new(Env::new()));
        let v = "((lambda (a b) (+ a b)) 1 2)".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(3));
        let v = "(define sum (lambda (a b) (+ a b))) (sum (sum 1 2) 3)".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(6));
    }

    #[test]
    fn test_if() {
        let env = Rc::new(RefCell::new(Env::new()));
        let v = "(if #t 1 2)".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(1));
        let v = "(if #f 1 2)".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(2));
        let v = "(if (eq? 1 (+ 1 1)) 1 2)".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(2));
        let v = "(define fact (lambda (n) (if (eq? n 0) 1 (* n (fact (- n 1)))))) (fact 5)".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(120));
    }

    #[test]
    fn test_list() {
        let env = Rc::new(RefCell::new(Env::new()));
        let v = "null".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Null);
        let v = "(car (cons 1 2))".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(1));
        let v = "(cdr (cons 1 2))".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(2));
        let v = "(car (cdr (cons 1 (cons 2 null))))".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(2));
    }

    #[test]
    fn test_quote() {
        let env = Rc::new(RefCell::new(Env::new()));
        let v = "'()".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Null);
        let v = "'123".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(123));
        let v = "'abc".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Symbol("abc".into()));
        let v = "(car '(1 2))".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(1));
        let v = "(car (cdr '(1 2)))".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(2));
        let v = "(car (car '('1)))".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Symbol("quote".into()));
        let v = "(car (cdr (car '('1))))".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(1));
    }

    #[test]
    fn test_list_op() {
        let env = Rc::new(RefCell::new(Env::new()));
        let v = "'()".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Null);
        let v = "'123".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(123));
        let v = "'abc".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Symbol("abc".into()));
        let v = "(car '(1 2))".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(1));
        let v = "(car (cdr '(1 2)))".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(2));
        let v = "(car (car '('1)))".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Symbol("quote".into()));
        let v = "(car (cdr (car '('1))))".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(1));
    }

    #[test]
    fn test_value_to_stirng() {
        let env = Rc::new(RefCell::new(Env::new()));
        let v = "'()".eval(env.clone()).unwrap().to_string();
        assert_eq!(v, "'()".to_string());
        let v = "123".eval(env.clone()).unwrap().to_string();
        assert_eq!(v, "123".to_string());
        let v = "'(1 2 (3 4) 5)".eval(env.clone()).unwrap().to_string();
        assert_eq!(v, "'(1 2 (3 4) 5)".to_string());
        let v = "(cons 1 2)".eval(env.clone()).unwrap().to_string();
        assert_eq!(v, "'(1 . 2)".to_string());
        let v = "(cons '(1 2) (cons '(3 4) 5))".eval(env.clone()).unwrap().to_string();
        assert_eq!(v, "'((1 2) (3 4) . 5)".to_string());
    }


    #[test]
    fn test_perm() {
        let env = Rc::new(RefCell::new(Env::new()));
        let v = r###"
            (define map (lambda (f l)
                (if (null? l)
                    '()
                    (cons (f (car l)) (map f (cdr l))))))
            (define filter (lambda (f l)
                (if (null? l)
                    '()
                    (if (f (car l))
                        (cons (car l) (filter f (cdr l)))
                        (filter f (cdr l))))))
            (define append (lambda (a b)
                (if (null? a)
                    b
                    (cons (car a) (append (cdr a) b)))))
            (define flat (lambda (l)
                (if (null? l)
                    '()
                    (append (car l) (flat (cdr l))))))
            (define not (lambda (x) (if x #f #t)))
            (define perm (lambda (l)
                (if (null? l)
                    '(())
                    (flat (map (lambda (x)
                                (map (lambda (y) (cons x y)) (perm (filter (lambda (y) (not (eq? y x))) l))))
                            l)))))
            (perm '(1 2 3))
        "###.eval(env.clone()).unwrap().to_string();
        assert_eq!(v, "'((1 2 3) (1 3 2) (2 1 3) (2 3 1) (3 1 2) (3 2 1))".to_string());
    }
}
