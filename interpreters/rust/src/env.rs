use std::{collections::HashMap, rc::Rc, cell::RefCell};

use crate::{value::{Value, BuiltInFunType}, eval::Interpreter};

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Env {
    map: HashMap<String, Rc<Value>>,
    parent: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub fn get(&self, key : &str) -> Option<Rc<Value>> {
        if let Some(v) = self.map.get(key) {
            Some(v.clone())
        } else {
            if let Some (v) = self.parent.as_ref().and_then(|e| e.borrow().get(key)) {
                Some(v.clone())
            } else {
                None
            }
        }
    }

    pub fn set(&mut self, key : String, value: Rc<Value>) {
        self.map.insert(key, value);
    }

    pub fn new() -> Rc<RefCell<Self>> {
        let res = Rc::new(RefCell::new(Env { 
            map: HashMap::from([
                ("+".to_string(), Rc::new(Value::BuiltInFun(BuiltInFunType::Add))),
                ("-".to_string(), Rc::new(Value::BuiltInFun(BuiltInFunType::Sub))),
                ("*".to_string(), Rc::new(Value::BuiltInFun(BuiltInFunType::Mul))),
                ("eq?".to_string(), Rc::new(Value::BuiltInFun(BuiltInFunType::Eq))),
                ("cons".to_string(), Rc::new(Value::BuiltInFun(BuiltInFunType::Cons))),
                ("car".to_string(), Rc::new(Value::BuiltInFun(BuiltInFunType::Car))),
                ("cdr".to_string(), Rc::new(Value::BuiltInFun(BuiltInFunType::Cdr))),
                ("null?".to_string(), Rc::new(Value::BuiltInFun(BuiltInFunType::IsNull))),
                ("display".to_string(), Rc::new(Value::BuiltInFun(BuiltInFunType::Display))),
            ]),
            parent: None
        }));
        static PRELUDE : &str = r###"
            (define (map f l)
                (if (null? l)
                    '()
                    (cons (f (car l)) (map f (cdr l)))))
            (define (filter f l)
                (if (null? l)
                    '()
                    (if (f (car l))
                        (cons (car l) (filter f (cdr l)))
                        (filter f (cdr l)))))
            (define (append a b)
                (if (null? a)
                    b
                    (cons (car a) (append (cdr a) b))))
            (define (not x) (if x #f #t))
            (define (flat l)
                (if (null? l)
                    '()
                    (append (car l) (flat (cdr l)))))
            (define (cadr x) (car (cdr x)))
        "###;
        if let Err(e) = PRELUDE.eval(res.clone()) {
            println!("{:?}", e);
            panic!("failed to evaluate prelude");
        }
        res
    }

    pub fn new_with_parent(parent: Rc<RefCell<Env>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Env{map: HashMap::new(), parent: Some(parent)}))
    }
}