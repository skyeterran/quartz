use std::fs;
use std::error::Error;
use std::fmt;

/// The location of a token/expression in the source code
#[derive(Debug, Clone, Copy)]
struct Location {
    line: usize,
    column: usize,
}

impl Location {
    fn new(line: usize, column: usize) -> Self {
        Self {
            line,
            column,
        }
    }
}

#[derive(Debug, Clone)]
enum Token {
    LParen {
        location: Location,
    },
    RParen {
        location: Location,
    },
    Symbol {
        content: String,
        location: Location,
    },
    StrLit {
        content: String,
        location: Location,
    },
}

impl Token {
    fn get_location(&self) -> Location {
        match self {
            Self::LParen { location } => { *location }
            Self::RParen { location } => { *location }
            Self::Symbol { location, .. } => { *location }
            Self::StrLit { location, .. } => { *location }
        }
    }
}

enum Quote {
    Single,
    Double
}

#[derive(Debug, Clone)]
enum Exp {
    Nil,
    List {
        contents: Vec<Exp>,
    },
    Token {
        contents: Token,
    },
}

#[derive(Debug)]
struct ParseError {
    message: String,
    location: Location,
    cause: Option<Box<dyn Error>>,
}

impl ParseError {
    fn new(message: String, location: Location) -> Box<dyn Error> {
        Box::new(Self { message, location, cause: None })
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        &self.message
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.cause.as_ref().map(|e| &**e)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let source = fs::read_to_string("test.qtz")?;

    // Eat up those characters
    let mut in_symbol = false;
    let mut current_symbol = String::new();
    let mut in_string = false;
    let mut current_string = String::new();
    let mut string_delim = Quote::Single;
    let mut tokens: Vec<Token> = Vec::new();
    let mut line: usize = 1;
    let mut this_line: usize = 1;
    let mut column: usize = 0;
    let mut this_column: usize = 0;
    let mut mark_pos = true;

    for c in source.chars() {
        if let '\n' = c {
            line += 1;
            column = 0;
        } else {
            column += 1;
        }
        if mark_pos {
            this_line = line;
            this_column = column;
        }
        //println!("[{line}:{column}] ({this_line}:{this_column}) '{c}'");
        if in_string {
            match c {
                '\'' => {
                    if let Quote::Single = string_delim {
                        in_string = false;
                        tokens.push(Token::StrLit {
                            content: current_string.clone(),
                            location: Location::new(this_line, this_column),
                        });
                        current_string = String::new();
                        mark_pos = false;
                        continue;
                    }
                }
                '\"' => {
                    if let Quote::Double = string_delim {
                        in_string = false;
                        tokens.push(Token::StrLit {
                            content: current_string.clone(),
                            location: Location::new(this_line, this_column),
                        });
                        current_string = String::new();
                        mark_pos = false;
                        continue;
                    }
                }
                _ => {
                    mark_pos = false;
                }
            }
            current_string.push(c);
        } else {
            match c {
                '(' => {
                    tokens.push(Token::LParen {
                        location: Location::new(line, column),
                    });
                    mark_pos = true;
                }
                ')' => {
                    if in_symbol {
                        tokens.push(Token::Symbol {
                            content: current_symbol.clone(),
                            location: Location::new(this_line, this_column),
                        });
                        current_symbol = String::new();
                        in_symbol = false;
                    }
                    tokens.push(Token::RParen {
                        location: Location::new(line, column),
                    });
                    mark_pos = true;
                }
                '\'' => {
                    in_string = true;
                    string_delim = Quote::Single;
                    mark_pos = false;
                }
                '\"' => {
                    in_string = true;
                    string_delim = Quote::Double;
                    mark_pos = false;
                }
                ' ' | '\n' => {
                    if in_symbol {
                        if current_symbol != "" {
                            tokens.push(Token::Symbol {
                                content: current_symbol.clone(),
                                location: Location::new(this_line, this_column),
                            });
                            current_symbol = String::new();
                            in_symbol = false;
                        }
                    }
                    mark_pos = true;
                }
                _ => {
                    in_symbol = true;
                    current_symbol.push(c);
                    mark_pos = current_symbol == "";
                }
            }
        }
    }
    if in_symbol {
        tokens.push(Token::Symbol {
            content: current_symbol.clone(),
            location: Location::new(this_line, this_column),
        });
    }

    //let mut expressions: Vec<Exp> = Vec::new();
    let expressions = parse_expression(&tokens, 0, tokens.len())?;

    println!("{:#?}", expressions);

    Ok(())
}

// start: the start of the range of tokens to parse
// end: the end of the range of tokens to parse
fn parse_expression(
    tokens: &Vec<Token>,
    start: usize,
    end: usize
) -> Result<Exp, Box<dyn Error>> {
    let mut contents: Vec<Exp> = Vec::new();
    let mut nested = false;
    let mut i: usize = start;
    let mut location = Location::new(0, 0);
    loop {
        if i > end || i >= tokens.len() { break; }
        let t = tokens.get(i).unwrap();
        location = t.get_location();
        match t {
            Token::LParen {..} => {
                if nested {
                    // Find the matching RParen
                    let inner_end = find_exp_end(tokens, i);
                    contents.push(parse_expression(tokens, i, inner_end)?);
                    i = inner_end;
                } else {
                    nested = true;
                }
            }
            Token::RParen {..} => {
                if nested {
                    /*
                    if contents.is_empty() {
                        return Ok(Exp::Nil);
                    } else {
                        return Ok(Exp::List { contents });
                    }
                    */
                    nested = false;
                } else {
                    // Syntax error: Unexpected RParen
                    return Err(ParseError::new(
                        format!("Unexpected closing parentheses"),
                        location,
                    ));
                }
            }
            Token::Symbol {..} | Token::StrLit {..} => {
                if nested {
                    contents.push(Exp::Token { contents: t.clone() });
                } else {
                    if end - start > 1 {
                        // Multiple atoms outside of a list (syntax error)
                        return Err(ParseError::new(
                            format!("Invalid expression"),
                            location,
                        ));
                    } else {
                        // Single atom
                        return Ok(Exp::Token { contents: t.clone() });
                    }
                }
            }
        }
        i += 1;
    }
    if nested {
        // Syntax error: missing RParen
        Err(ParseError::new(
            format!("Missing closing parentheses"),
            location,
        ))
    } else {
        if contents.is_empty() {
            return Ok(Exp::Nil);
        } else {
            return Ok(Exp::List { contents });
        }
    }
}

// Given a starting LParen index, return the index of the closing RParen
fn find_exp_end(tokens: &Vec<Token>, start: usize) -> usize {
    let mut nesting: usize = 0;
    for i in start..tokens.len() {
        let t = tokens.get(i).unwrap();
        match t {
            Token::LParen {..} => {
                nesting += 1;
                // Find the matching RParen
            }
            Token::RParen {..} => {
                match nesting {
                    0 => { // ERROR: Unexpected RParen
                        todo!();
                    }
                    1 => { // Reached end of current outer exp
                        return i;
                    }
                    _ => {}
                }
                nesting -= 1;
            }
            _ => {}
        }
    }
    todo!()
}