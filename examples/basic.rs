use std::fs;
use std::error::Error;
use quartz::parse::{
    Token, Exp, tokenize, parse_expression
};

fn main() -> Result<(), Box<dyn Error>> {
    let source = fs::read_to_string("test.qz")?;
    let tokens = tokenize(source);

    let mut expressions: Vec<Exp> = Vec::new();
    let mut nesting: usize = 0;
    let mut start: usize = 0;
    let mut i: usize = 0;
    for t in &tokens {
        match t {
            Token::LParen {..} | Token::LBracket {..} => {
                if nesting == 0 {
                    start = i;
                }
                nesting += 1;
            }
            Token::RParen {..} | Token::RBracket {..} => {
                if nesting > 0 {
                    nesting -= 1;
                } else {
                    todo!() // Oh, what horror!
                }
                
                if nesting == 0 {
                    expressions.push(parse_expression(&tokens, start, i)?);
                }
            }
            _ => {
                if nesting == 0 {
                    expressions.push(Exp::Token { contents: t.clone() });
                }
            }
        }
        i += 1;
    }

    for x in expressions {
        println!("{x}");
    }

    Ok(())
}
