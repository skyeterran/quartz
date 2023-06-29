use std::error::Error;
use std::fmt;
use std::fs;

/// The location of a token/expression in the source code
#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

impl Location {
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            line,
            column,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Token {
    LParen {
        location: Location,
    },
    RParen {
        location: Location,
    },
    LBracket {
        location: Location,
    },
    RBracket {
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
    Comment {
        content: String,
        location: Location,
    },
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::LParen {..} => { write!(f, "(") }
            Token::RParen {..} => { write!(f, ")") }
            Token::LBracket {..} => { write!(f, "[") }
            Token::RBracket {..} => { write!(f, "]") }
            Token::Symbol { content, .. } => { write!(f, "{content}") }
            Token::StrLit { content, .. } => { write!(f, "\"{content}\"") }
            Token::Comment { content, .. } => { write!(f, "{content}") }
        }
    }
}

impl Token {
    fn get_location(&self) -> Location {
        match self {
            Self::LParen { location } => { *location }
            Self::RParen { location } => { *location }
            Self::LBracket { location } => { *location }
            Self::RBracket { location } => { *location }
            Self::Symbol { location, .. } => { *location }
            Self::StrLit { location, .. } => { *location }
            Self::Comment { location, .. } => { *location }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Exp {
    Nil,
    SExp {
        car: Box<Exp>,
        cdr: Vec<Exp>,
    },
    List {
        contents: Vec<Exp>,
    },
    Symbol {
        contents: Token,
    },
}

impl Exp {
    fn new_sexp(contents: Vec<Exp>) -> Self {
        let mut car = Exp::Nil;
        let mut cdr: Vec<Exp> = Vec::new();
        let mut i: usize = 0;
        for c in contents {
            if i == 0 {
                car = c;
            } else {
                cdr.push(c);
            }
            i += 1;
        }
        Self::SExp { car: Box::new(car), cdr }
    }
}

impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Exp::Nil => {
                write!(f, "nil")
            }
            Exp::SExp { car, cdr } => {
                let content = if cdr.is_empty() {
                    format!("({car})")
                } else {
                    let cdr_list = cdr.iter()
                                      .map(|x| {format!("{x}")})
                                      .collect::<Vec<String>>()
                                      .join(" ");
                    format!("({car} {cdr_list})")
                };
                write!(f, "{content}")
            }
            Exp::List { contents } => {
                let inner = contents.iter()
                                    .map(|x| {format!("{x}")})
                                    .collect::<Vec<String>>()
                                    .join(" ");
                write!(f, "[{inner}]")
            }
            Exp::Symbol { contents } => {
                write!(f, "{contents}")
            }
        }
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub location: Location,
    pub cause: Option<Box<dyn Error>>,
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

pub enum ParseMode {
    String,
    Normal,
    Comment,
}

pub fn tokenize(source: String) -> Vec<Token> {
    // Eat up those characters
    let mut mode = ParseMode::Normal;
    let mut in_symbol = false;
    let mut current_symbol = String::new();
    let mut current_string = String::new();
    let mut current_comment = String::new();
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
        match mode {
            ParseMode::String => {
                match c {
                    '\"' => {
                        mode = ParseMode::Normal;
                        tokens.push(Token::StrLit {
                            content: current_string.clone(),
                            location: Location::new(this_line, this_column),
                        });
                        current_string = String::new();
                        mark_pos = false;
                        continue;
                    }
                    _ => {
                        mark_pos = false;
                    }
                }
                current_string.push(c);
            }
            ParseMode::Normal => {
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
                    '[' => {
                        tokens.push(Token::LBracket {
                            location: Location::new(line, column),
                        });
                        mark_pos = true;
                    }
                    ']' => {
                        if in_symbol {
                            tokens.push(Token::Symbol {
                                content: current_symbol.clone(),
                                location: Location::new(this_line, this_column),
                            });
                            current_symbol = String::new();
                            in_symbol = false;
                        }
                        tokens.push(Token::RBracket {
                            location: Location::new(line, column),
                        });
                        mark_pos = true;
                    }
                    '\"' => {
                        mode = ParseMode::String;
                        mark_pos = false;
                    }
                    ' ' | '\n' => {
                        if in_symbol {
                            if !current_symbol.is_empty() {
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
                    ';' => {
                        mode = ParseMode::Comment;
                    }
                    _ => {
                        in_symbol = true;
                        current_symbol.push(c);
                        mark_pos = current_symbol == "";
                    }
                }
            }
            ParseMode::Comment => {
                match c {
                    '\n' => {
                        // This comment is done, save it if needed
                        if !current_comment.is_empty() {
                            tokens.push(Token::Comment {
                                content: current_comment.clone(),
                                location: Location::new(this_line, this_column),
                            });
                            current_comment = String::new();
                        }
                        mode = ParseMode::Normal;
                        mark_pos = true;
                    }
                    _ => {
                        current_comment.push(c);
                        mark_pos = current_comment == "";
                    }
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

    tokens
}

// Reads a list of expressions from a source string
pub fn read_expressions(source: String) -> Result<Vec<Exp>, Box<dyn Error>> {
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
                    expressions.push(process_token(t));
                }
            }
        }
        i += 1;
    }

    Ok(expressions)
}

pub fn process_token(token: &Token) -> Exp {
    match token {
        Token::Symbol { content, .. }=> {
            if content == "nil" {
                Exp::Nil
            } else {
                Exp::Symbol { contents: token.clone() }
            }
        }
        _ => Exp::Symbol { contents: token.clone() }
    }
}

// start: the start of the range of tokens to parse
// end: the end of the range of tokens to parse
pub fn parse_expression(
    tokens: &Vec<Token>,
    start: usize,
    end: usize
) -> Result<Exp, Box<dyn Error>> {
    let mut contents: Vec<Exp> = Vec::new();
    let mut nested = false;
    let mut in_list = false;
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
                    let inner_end = find_exp_end(tokens, i, false)?;
                    contents.push(parse_expression(tokens, i, inner_end)?);
                    i = inner_end;
                } else {
                    nested = true;
                }
            }
            Token::RParen {..} => {
                if nested {
                    nested = false;
                } else {
                    // Syntax error: Unexpected RParen
                    return Err(ParseError::new(
                        format!("Unexpected closing paren"),
                        location,
                    ));
                }
            }
            Token::LBracket {..} => {
                if nested {
                    // Find the matching RParen
                    let inner_end = find_exp_end(tokens, i, true)?;
                    contents.push(parse_expression(tokens, i, inner_end)?);
                    i = inner_end;
                } else {
                    nested = true;
                    in_list = true;
                }
            }
            Token::RBracket { location } => {
                if !in_list {
                    return Err(ParseError::new(
                        format!("Unexpected closing bracket"),
                        *location,
                    ));
                }
                if nested {
                    nested = false;
                } else {
                    return Err(ParseError::new(
                        format!("Unexpected closing bracket"),
                        *location,
                    ));
                }
            }
            Token::Symbol {..} | Token::StrLit {..} => {
                if nested {
                    contents.push(process_token(t));
                } else {
                    if end - start > 1 {
                        // Multiple atoms outside of a list (syntax error)
                        return Err(ParseError::new(
                            format!("Expression missing opening paren/bracket"),
                            location,
                        ));
                    } else {
                        // Single atom
                        return Ok(process_token(t));
                    }
                }
            }
            _ => {}
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
        if in_list {
            return Ok(Exp::List { contents });
        } else {
            if contents.is_empty() {
                return Ok(Exp::Nil);
            } else {
                return Ok(Exp::new_sexp(contents));
            }
        }
    }
}

// Given a starting LParen index, return the index of the closing RParen
fn find_exp_end(
    tokens: &Vec<Token>,
    start: usize,
    in_list: bool
) -> Result<usize, Box<dyn Error>> {
    let mut nesting: usize = 0;
    let mut start_location = Location::new(0, 0);
    for i in start..tokens.len() {
        let t = tokens.get(i).unwrap();
        if i == start {
            start_location = t.get_location();
        }
        match t {
            Token::LParen {..} => {
                nesting += 1;
            }
            Token::RParen {..} => {
                match nesting {
                    0 => {
                        return Err(ParseError::new(
                            format!("List is missing closing bracket"),
                            start_location,
                        ));
                    }
                    1 => {
                        if in_list {
                            return Err(ParseError::new(
                                format!("List is missing closing bracket"),
                                start_location,
                            ));
                        } else {
                            return Ok(i);
                        }
                    }
                    _ => {}
                }
                nesting -= 1;
            }
            Token::LBracket {..} => {
                nesting += 1;
            }
            Token::RBracket { location } => {
                match nesting {
                    0 => {
                        return Err(ParseError::new(
                            format!("Unexpected closing bracket"),
                            *location,
                        ));
                    }
                    1 => {
                        if !in_list {
                            return Err(ParseError::new(
                                format!("Unexpected closing bracket"),
                                *location,
                            ));
                        } else {
                            return Ok(i);
                        }
                    }
                    _ => {}
                }
                nesting -= 1;
            }
            _ => {}
        }
    }
    Err(ParseError::new(
        format!("Inner expression is never closed"),
        start_location,
    ))
}
