#[derive(Debug, PartialEq)]
pub enum Token {
    LeftParen,
    RightParen,
    Quote,
    Symbol(String),
}

#[derive(Default)]
pub struct Tokenizer {
    cur : String,
}

impl Tokenizer {
    fn new() -> Tokenizer {
        Default::default()
    }
    
    fn push_token(&mut self, tokens : &mut Vec<Token>) {
       if self.cur.len() > 0 {
            tokens.push(Token::Symbol(self.cur.clone()));
            self.cur = "".into();
        }
    }

}

pub fn tokenize(s: &str) -> Vec<Token> {
    let mut t = Tokenizer::new();
    let mut tokens = vec![];
    for c in s.chars() {
        match c {
            '(' => {
                t.push_token(&mut tokens);
                tokens.push(Token::LeftParen);
            },
            ')' => {
                t.push_token(&mut tokens);
                tokens.push(Token::RightParen);
            },
            '\'' => {
                t.push_token(&mut tokens);
                tokens.push(Token::Quote);
            },
            c if c.is_whitespace() =>  t.push_token(&mut tokens),
            c => t.cur.push(c),
        }
    }
    t.push_token(&mut tokens);
    tokens
}