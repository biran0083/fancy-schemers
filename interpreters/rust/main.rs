use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum AtomValue {
    Int(i64),
    Bool(bool),
    Symbol(String),
}


#[derive(Debug, PartialEq, Clone, Eq)]
pub enum SExp {
    Atom(AtomValue),
    List(Vec<SExp>),
}

impl SExp {
    fn get_symbol(&self) -> Option<&str> {
        if let SExp::Atom(AtomValue::Symbol(s)) = self {
            Some(s)
        } else {
            None
        }
    }
}

use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
struct ParseError {
    msg: String,
}

#[derive(Debug, PartialEq, Eq)]
struct ParseResult {
    exps: Vec<SExp>,
    rest: String,
}

#[derive(Debug, PartialEq, Eq)]
struct EvalError {
    msg: String,
}

#[derive(Debug, PartialEq, Eq)]
struct Env {
    map: HashMap<String, SchemeValue>,
    parent: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub fn get(&self, key : &str) -> Option<SchemeValue> {
        if let Some(v) = self.map.get(key) {
            Some(v.clone())
        } else if let Some(e) = &self.parent {
            e.borrow().get(key)
        } else {
            None
        }
    }

    pub fn set(&mut self, key : String, value: SchemeValue) {
        self.map.insert(key, value);
        ()
    }

    pub fn new() -> Self {
        let mut res = Env{map: HashMap::new(), parent: None};
        res.set("null".into(), SchemeValue::ListValue(SchemeListValue::Null));
        res
    }

    pub fn new_with_parent(parent: Rc<RefCell<Env>>) -> Self{
        Env{map: HashMap::new(), parent: Some(parent)}
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Closure {
    env: Rc<RefCell<Env>>,
    body: SExp,
    params: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum SchemeListValue {
    Null,
    Pair(Rc<RefCell<SchemeValue>>, Rc<RefCell<SchemeValue>>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum SchemeValue {
    Void,
    Int(i64),
    BuiltInFunctionValue(BuiltInFunction),
    ClosureValue(Closure),
    Bool(bool),
    ListValue(SchemeListValue),
    SymbolValue(String),
}

// without outer '()
fn list_to_string_inner(value : &SchemeValue) -> String {
    match value {
        SchemeValue::ListValue(SchemeListValue::Null) => "".into(),
        SchemeValue::ListValue(SchemeListValue::Pair(fst, rest)) => {
            let t = (*rest).borrow();
            match *t {
                SchemeValue::ListValue(SchemeListValue::Null) => to_string_helper(&(*fst).borrow()),
                SchemeValue::ListValue(_) => format!("{} {}", &to_string_helper(&(*fst).borrow()), &list_to_string_inner(&t)),
                _ => format!("{} . {}", &to_string_helper(&(*fst).borrow()), &to_string_helper(&t)),
            }
        },
        _ => panic!("should not reach here"),
    }
}

// list without prefix '
fn to_string_helper(value : &SchemeValue) -> String {
    match value {
        SchemeValue::Void => "#void".into(),
        SchemeValue::Int(v) => v.to_string(),
        SchemeValue::BuiltInFunctionValue(f) => format!("{:?}",f),
        SchemeValue::ClosureValue(_) => "#procedure".into(),
        SchemeValue::Bool(v) => if *v {
            "#t".into()
        } else {
            "#f".into()
        },
        SchemeValue::ListValue(_) => format!("({})", list_to_string_inner(value)),
        SchemeValue::SymbolValue(s) => s.clone(),
    }
}

impl ToString for SchemeValue {
    fn to_string(&self) -> String {
        match self {
            SchemeValue::ListValue(_) => format!("'{}", to_string_helper(self)),
            _ => to_string_helper(self),
        }
    }
}

trait Interpreter {
    fn eval(&self, env : Rc<RefCell<Env>>) -> Result<SchemeValue, EvalError>;
}

impl From<SExp> for SchemeValue {
    fn from(exp: SExp) -> SchemeValue {
        match exp {
            SExp::Atom(AtomValue::Int(v)) => SchemeValue::Int(v),
            SExp::Atom(AtomValue::Bool(v)) => SchemeValue::Bool(v),
            SExp::Atom(AtomValue::Symbol(v)) => SchemeValue::SymbolValue(v),
            SExp::List(exps) if exps.len() == 0 =>  SchemeValue::ListValue(SchemeListValue::Null),
            SExp::List(mut exps) => {
                let first : SchemeValue = exps[0].clone().into();
                exps.remove(0);
                let rest : SchemeValue = SExp::List(exps).into();
                SchemeValue::ListValue(SchemeListValue::Pair(Rc::new(RefCell::new(first)), Rc::new(RefCell::new(rest))))
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum BuiltInFunction {
    Add,
    Eq,
    Sub,
    Mul,
    Cons,
    Car,
    Cdr,
    IsNull,
}

impl FromStr for BuiltInFunction {

    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err>  {
        match s {
            "+" => Ok(BuiltInFunction::Add),
            "-" => Ok(BuiltInFunction::Sub),
            "*" => Ok(BuiltInFunction::Mul),
            "eq?" => Ok(BuiltInFunction::Eq),
            "cons" => Ok(BuiltInFunction::Cons),
            "car" => Ok(BuiltInFunction::Car),
            "cdr" => Ok(BuiltInFunction::Cdr),
            "null?" => Ok(BuiltInFunction::IsNull),
            _ => Err(()),
        }
    }
}

fn apply_built_in_function(f : BuiltInFunction, params : Vec<SchemeValue>) -> Result<SchemeValue, EvalError> {
    match f {
        BuiltInFunction::Add => {
            let mut res : i64 = 0;
            for param in params {
                if let SchemeValue::Int(v) = param {
                    res += v;
                } else {
                    return Err(EvalError{msg: format!("cannot add non int value: {:?}", param)});
                }
            }
            return Ok(SchemeValue::Int(res));
        },
        BuiltInFunction::Mul => {
            let mut res : i64 = 1;
            for param in params {
                if let SchemeValue::Int(v) = param {
                    res *= v;
                } else {
                    return Err(EvalError{msg: format!("cannot add non int value: {:?}", param)});
                }
            }
            return Ok(SchemeValue::Int(res));
        },
        BuiltInFunction::Eq => {
            assert_eq!(params.len(), 2);
            Ok(SchemeValue::Bool(params[0] == params[1]))
        },
        BuiltInFunction::Sub => {
            assert_eq!(params.len(), 2);
            if let SchemeValue::Int(a) = params[0] {
                if let SchemeValue::Int(b) = params[1] {
                    return Ok(SchemeValue::Int(a - b));
                }
            }
            return Err(EvalError{msg: "- only applies to integers".into()});
        },
        BuiltInFunction::Cons => {
            assert_eq!(params.len(), 2);
            Ok(SchemeValue::ListValue(SchemeListValue::Pair(
                Rc::new(RefCell::new(params[0].clone())), Rc::new(RefCell::new(params[1].clone())))))
        },
        BuiltInFunction::Car => {
            assert_eq!(params.len(), 1);
            if let SchemeValue::ListValue(SchemeListValue::Pair(v, _)) = params.into_iter().next().unwrap() {
                Ok((*v).borrow().clone())
            } else {
                Err(EvalError{msg: "car only apply to pairs".into()})
            }
        },
        BuiltInFunction::Cdr => {
            assert_eq!(params.len(), 1);
            if let SchemeValue::ListValue(SchemeListValue::Pair(_, v)) = params.into_iter().next().unwrap() {
                Ok((*v).borrow().clone())
            } else {
                Err(EvalError{msg: "car only apply to pairs".into()})
            }
        },
        BuiltInFunction::IsNull => {
            assert_eq!(params.len(), 1);
            if let SchemeValue::ListValue(SchemeListValue::Null) = params[0] {
                Ok(SchemeValue::Bool(true))
            } else {
                Ok(SchemeValue::Bool(false))
            }
        }
    }
}

impl Interpreter for SExp {
    fn eval(&self, env : Rc<RefCell<Env>>) -> Result<SchemeValue, EvalError> {
        match self {
            SExp::Atom(AtomValue::Int(v)) => Ok(SchemeValue::Int(*v)),
            SExp::Atom(AtomValue::Bool(v)) => Ok(SchemeValue::Bool(*v)),
            SExp::Atom(AtomValue::Symbol(s)) => Ok(env.borrow().get(s.as_str()).unwrap_or_else(|| s.parse::<BuiltInFunction>().map(SchemeValue::BuiltInFunctionValue).unwrap_or(SchemeValue::Void))),
            SExp::List(lst) => {
                assert!(lst.len() > 0);
                match lst[0].get_symbol() {
                    Some("define") => {
                        assert_eq!(lst.len(), 3);
                        match lst[1].get_symbol() {
                            Some(name) =>  {
                                let value = lst[2].eval(env.clone())?;
                                (*env).borrow_mut().set(name.into(), value);
                                Ok(SchemeValue::Void)
                            }
                            _ => Err(EvalError{msg:"TODO: handle define".into()})
                        }
                    },
                    Some("lambda") => {
                        assert_eq!(lst.len(), 3);
                        let mut closure = Closure{env: env.clone(), params: vec![], body: lst[2].clone()};
                        if let SExp::List(params) = &lst[1] {
                            for p in params {
                                if let Some(name) = p.get_symbol() {
                                    closure.params.push(name.into());
                                } else {
                                    return Err(EvalError{msg: "param is not symbol".into()})
                                }
                            }
                            return Ok(SchemeValue::ClosureValue(closure));
                        } else {
                            return Err(EvalError{msg: "TODO: not support lambda with list params".into()});
                        }
                    },
                    Some("if") => {
                        assert_eq!(lst.len(), 4);
                        let cond = lst[1].eval(env.clone())?;
                        match cond {
                            SchemeValue::Bool(v) => {
                                if v {
                                    return lst[2].eval(env.clone());
                                } else {
                                    return lst[3].eval(env.clone());
                                }
                            },
                            _ => {
                                return Err(EvalError{msg: "Condition is not boolean".into()});
                            },
                        }
                    },
                    Some("quote") => {
                        assert_eq!(lst.len(), 2);
                        Ok(lst[1].clone().into())
                    },
                    _ => {
                        let f = lst[0].eval(env.clone())?;
                        match f {
                            SchemeValue::BuiltInFunctionValue(f) => {
                                let mut params = vec![];
                                for i in 1..lst.len() {
                                    params.push(lst[i].eval(env.clone())?);
                                }
                                return apply_built_in_function(f, params);
                            },
                            SchemeValue::ClosureValue(c) => {
                                if lst.len() != c.params.len() + 1 {
                                    return Err(EvalError{msg: "param number mismatch".into()});
                                }
                                let mut e = Env::new_with_parent(c.env);
                                for i in 1..lst.len() {
                                    let v = lst[i].eval(env.clone())?;
                                    e.set(c.params[i - 1].clone(), v);
                                }
                                return c.body.eval(Rc::new(RefCell::new(e)));
                            },
                            _ => {
                                return Err(EvalError{msg: format!("unknown function type: {:?}", f)})
                            },
                        }
                    },
                }
            },
        }
    }
}

impl Interpreter for str {
    fn eval(&self, env : Rc<RefCell<Env>>) -> Result<SchemeValue, EvalError> {
        let res = self.parse::<ParseResult>()?;
        assert!(res.rest == "");
        let mut result = SchemeValue::Void;
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
    if ss[0] == '\'' {
        let res = parse_one(&s[1..])?;
        return Ok(ParseResult{
            exps: vec![SExp::List(vec![
                SExp::Atom(AtomValue::Symbol("quote".into())),
                res.exps[0].clone(),
            ])],
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
            exps: vec![SExp::List(exps)],
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
                exps: vec![SExp::Atom(AtomValue::Int(v))],
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
            "#t" => AtomValue::Bool(true),
            "#f" => AtomValue::Bool(false),
            s => AtomValue::Symbol(s.into()),
        };
        res.exps.push(SExp::Atom(atom_value));
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
    fn test_sexp_to_string() {
        let a = SExp::Atom(AtomValue::Int(10));
        assert_eq!("Atom(Int(10))", format!("{:?}", a));
        let b = SExp::Atom(AtomValue::Symbol("abc".into()));
        assert_eq!("Atom(Symbol(\"abc\"))", format!("{:?}", b));
        let c = SExp::List(vec![a, b]);
        assert_eq!("List([Atom(Int(10)), Atom(Symbol(\"abc\"))])", format!("{:?}", c));
    }

    #[test]
    fn test_parse() {
        assert_eq!(Ok(ParseResult{
            exps: vec![],
            rest: "".into(),
        }), "".parse::<ParseResult>());
        assert_eq!(Ok(ParseResult{
            exps: vec![SExp::Atom(AtomValue::Int(123))],
            rest: "".into(),
        }), "123".parse::<ParseResult>());
        assert_eq!(Ok(ParseResult{
            exps: vec![SExp::Atom(AtomValue::Int(456))],
            rest: "".into(),
        }), " 456".parse::<ParseResult>());
        assert_eq!(Err(ParseError{
            msg: "fail to parse int: 1a".into(),
        }), "1a".parse::<ParseResult>());
        assert_eq!(Ok(ParseResult{
            exps: vec![SExp::Atom(AtomValue::Symbol("a1".into()))],
            rest: "".into(),
        }), "a1".parse::<ParseResult>());
        assert_eq!(Ok(ParseResult{
            exps: vec![SExp::Atom(AtomValue::Symbol("b2".into()))],
            rest: "".into(),
        }), " b2 ".parse::<ParseResult>());
        assert_eq!(Ok(ParseResult{
            exps: vec![
                SExp::List(vec![SExp::Atom(AtomValue::Symbol("f".into())), SExp::Atom(AtomValue::Int(1))])
            ],
            rest: "".into(),
        }), "(f 1)".parse::<ParseResult>());
        assert_eq!(Ok(ParseResult{
            exps: vec![
                SExp::List(vec![
                    SExp::Atom(AtomValue::Symbol("f".into())),
                    SExp::List(vec![
                        SExp::Atom(AtomValue::Symbol("g".into())),
                        SExp::Atom(AtomValue::Int(1))
                    ]),
                    SExp::Atom(AtomValue::Symbol("abc".into()))
                ])
            ],
            rest: "".into(),
        }), "(f (g 1) abc)".parse::<ParseResult>());
    }

    #[test]
    fn test_eval() {
        let env = Rc::new(RefCell::new(Env::new()));
        let v = "123".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(123));
        let v = "(define a 123) a".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(123));
        let v = "(+ 1 2)".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(3));
        let v = "(define three (+ 1 2)) three".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(3));
        let v = "(define three (+ 1 2)) (+ three (+ 3 4 5))".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(15));
    }

    #[test]
    fn test_lambda() {
        let env = Rc::new(RefCell::new(Env::new()));
        let v = "((lambda (a b) (+ a b)) 1 2)".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(3));
        let v = "(define sum (lambda (a b) (+ a b))) (sum (sum 1 2) 3)".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(6));
    }

    #[test]
    fn test_if() {
        let env = Rc::new(RefCell::new(Env::new()));
        let v = "(if #t 1 2)".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(1));
        let v = "(if #f 1 2)".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(2));
        let v = "(if (eq? 1 (+ 1 1)) 1 2)".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(2));
        let v = "(define fact (lambda (n) (if (eq? n 0) 1 (* n (fact (- n 1)))))) (fact 5)".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(120));
    }

    #[test]
    fn test_list() {
        let env = Rc::new(RefCell::new(Env::new()));
        let v = "null".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::ListValue(SchemeListValue::Null));
        let v = "(car (cons 1 2))".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(1));
        let v = "(cdr (cons 1 2))".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(2));
        let v = "(car (cdr (cons 1 (cons 2 null))))".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(2));
    }

    #[test]
    fn test_quote() {
        let env = Rc::new(RefCell::new(Env::new()));
        let v = "'()".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::ListValue(SchemeListValue::Null));
        let v = "'123".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(123));
        let v = "'abc".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::SymbolValue("abc".into()));
        let v = "(car '(1 2))".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(1));
        let v = "(car (cdr '(1 2)))".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(2));
        let v = "(car (car '('1)))".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::SymbolValue("quote".into()));
        let v = "(car (cdr (car '('1))))".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(1));
    }

    #[test]
    fn test_list_op() {
        let env = Rc::new(RefCell::new(Env::new()));
        let v = "'()".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::ListValue(SchemeListValue::Null));
        let v = "'123".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(123));
        let v = "'abc".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::SymbolValue("abc".into()));
        let v = "(car '(1 2))".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(1));
        let v = "(car (cdr '(1 2)))".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(2));
        let v = "(car (car '('1)))".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::SymbolValue("quote".into()));
        let v = "(car (cdr (car '('1))))".eval(env.clone()).unwrap();
        assert_eq!(v, SchemeValue::Int(1));
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
