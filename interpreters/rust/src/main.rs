mod parser;
mod value;
mod env;
mod eval;
mod lex;
use env::Env;
use eval::{Interpreter, EvalError};
use linefeed::{Interface, ReadResult};
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let env = Env::new();
    if args.len() == 2 {
        match fs::read_to_string(&args[1]) {
            Ok(script) => {
                match script.eval(env.clone()) {
                    Ok(_) => (),
                    Err(e) => println!("execution failed: {:?}", e),
                }
            },
            Err(e) => {
                println!("{:?}", e);
            }
        }
    } else {
        let reader = Interface::new("fancy-schemers").unwrap();
        reader.set_prompt("scheme> ").unwrap();
        while let ReadResult::Input(input) = reader.read_line().unwrap() {
            match input.eval(env.clone()) {
                Ok(value) => println!("{}", value.to_string()),
                Err(EvalError{msg}) => println!("Err: {}", msg),
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use super::value::Value;

    #[test]
    fn test_eval() {
        let env = Env::new();
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
        let env = Env::new();
        let v = "((lambda (a b) (+ a b)) 1 2)".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(3));
        let v = "(define sum (lambda (a b) (+ a b))) (sum (sum 1 2) 3)".eval(env.clone()).unwrap();
        assert_eq!(v.as_ref(), &Value::Int(6));
    }

    #[test]
    fn test_if() {
        let env = Env::new();
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
        let env = Env::new();
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
        let env = Env::new();
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
        let env = Env::new();
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
        let env = Env::new();
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
        let env = Env::new();
        let v = r###"
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
    
    #[test]
    fn test_tail_recursion() {
        let env = Env::new();
        let v = r###"
        (define f (lambda (n res) (if (eq? n 0) res (f (- n 1) (+ 1 res)))))
        (f 10000 0)
        "###.eval(env.clone()).unwrap().to_string();
        assert_eq!(v, "10000".to_string());

    }

    #[test]
    fn test_multi_arg_lambda() {
        let env = Env::new();
        let v = r###"
        (define f (lambda x (car x)))
        (f 1 2)
        "###.eval(env.clone()).unwrap().to_string();
        assert_eq!(v, "1".to_string());
        let v = r###"
        (define f (lambda x (cdr x)))
        (f 1 2)
        "###.eval(env.clone()).unwrap().to_string();
        assert_eq!(v, "'(2)".to_string());
    }

    #[test]
    fn test_dot() {
        let env = Env::new();
        let v = r###"
        (define f (lambda (x . y) (car y)))
        (f 1 2 3)
        "###.eval(env.clone()).unwrap().to_string();
        assert_eq!(v, "2".to_string());
    }

    #[test]
    fn test_define_function() {
        let env = Env::new();
        let v = r###"
        (define (f x y) (+ x y))
        (f 1 2)
        "###.eval(env.clone()).unwrap().to_string();
        assert_eq!(v, "3".to_string());
        let v = r###"
        (define (f . x) (cdr x))
        (f 1 2)
        "###.eval(env.clone()).unwrap().to_string();
        assert_eq!(v, "'(2)".to_string());
    }

    #[test]
    fn test_define_macro() {
        let env = Env::new();
        let v = r###"
        (defmacro (and a b) (cons 'if (cons a (cons b (cons '#f null)))))
        (and (eq? 1 1) (eq? 1 2))
        "###.eval(env.clone()).unwrap().to_string();
        assert_eq!(v, "#f".to_string());

        let v = r###"
        (defmacro (and a b) `(if ,a ,b #f))
        (and (eq? 1 1) (eq? 1 2))
        "###.eval(env.clone()).unwrap().to_string();
        assert_eq!(v, "#f".to_string());
    
        let v = r###"
        (defmacro (let bindings . body) 
            `((lambda ,(map car bindings) . ,body) 
                . ,(map (lambda (x) (cadr x)) bindings)))
        (let ((x 1)
              (y 2)) (+ x y))
        "###.eval(env.clone()).unwrap().to_string();
        assert_eq!(v, "3".to_string());
    }

    #[test]
    fn test_multi_exp_function() {
        let env = Env::new();
        let v = r###"
        (define (f) 1 2)
        (f)
        "###.eval(env.clone()).unwrap().to_string();
        assert_eq!(v, "2".to_string());

        let v = r###"
        (define f (lambda () 1 2))
        (f)
        "###.eval(env.clone()).unwrap().to_string();
        assert_eq!(v, "2".to_string());
    }
}
