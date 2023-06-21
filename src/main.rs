use std::fs;
use std::error::Error;
use std::thread::current;

#[derive(Debug)]
enum Token {
    LParen {
        line: usize,
        column: usize,
    },
    RParen {
        line: usize,
        column: usize,
    },
    Symbol {
        content: String,
        line: usize,
        column: usize,
    },
    StrLit {
        content: String,
        line: usize,
        column: usize,
    },
}

enum Quote {
    Single,
    Double
}

enum Mode {
    Unsure,
    Symbol,
    String
}

fn main() -> Result<(), Box<dyn Error>> {
    let source = fs::read_to_string("test.qtz")?;
    println!("{}", source);

    // Eat up those characters
    let mut mode = Mode::Unsure;
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
                            line: this_line,
                            column: this_column
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
                            line: this_line,
                            column: this_column
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
                        line,
                        column
                    });
                    mark_pos = true;
                }
                ')' => {
                    if in_symbol {
                        tokens.push(Token::Symbol {
                            content: current_symbol.clone(),
                            line: this_line,
                            column: this_column
                        });
                        current_symbol = String::new();
                        in_symbol = false;
                    }
                    tokens.push(Token::RParen {
                        line,
                        column
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
                                line: this_line,
                                column: this_column
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

    println!("{:#?}", tokens);

    Ok(())
}