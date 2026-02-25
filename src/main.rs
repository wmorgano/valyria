use std::env;
use std::io::{self, Write};

fn display_prompt(prompt: &str, writer: &mut impl Write) {
    write!(writer, "{prompt}").expect("Failed to write prompt.");
}

fn read_expression(writer: &mut impl Write, reader: &mut impl io::BufRead) -> String {
    let prompt = ">> ";
    display_prompt(prompt, writer);
    writer.flush().expect("Failed to flush output stream.");

    let mut input = String::new();

    reader
        .read_line(&mut input)
        .expect("Failed to parse input.");

    input.trim().to_owned()
}

fn is_terminator(expression: &str) -> bool {
    expression == "quit"
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenType {
    Operator,
    Operand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ObjectType {
    Int,
    Float,
    String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    literal: String,
    token_type: TokenType,
    object_type: Option<ObjectType>,
}

impl Token {
    fn new(literal: &str) -> Token {
        let mut token_type = TokenType::Operand;
        let mut object_type = None;
        if matches!(literal, "+") {
            token_type = TokenType::Operator;
        } else {
            if literal.parse::<i64>().is_ok() {
                object_type = Some(ObjectType::Int);
            } else if literal.parse::<f64>().is_ok() {
                object_type = Some(ObjectType::Float);
            } else if literal.starts_with('"') && literal.ends_with('"') {
                object_type = Some(ObjectType::String);
            }
        }

        Token {
            literal: String::from(literal),
            token_type,
            object_type,
        }
    }

    fn unwrap_string(&self) -> Option<String> {
        if self.token_type == TokenType::Operand
            && self.literal.starts_with('"')
            && self.literal.ends_with('"')
        {
            Some(self.literal[1..self.literal.len() - 1].to_string())
        } else {
            None
        }
    }
}

fn parse(expression: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut is_in_quotes = false;

    for ch in expression.chars() {
        match ch {
            '"' => {
                current_token.push(ch);
                is_in_quotes = !is_in_quotes;
            }
            ' ' if !is_in_quotes => {
                if !current_token.is_empty() {
                    tokens.push(Token::new(&current_token));
                    current_token.clear();
                }
            }
            _ => {
                current_token.push(ch);
            }
        }
    }

    if !current_token.is_empty() {
        tokens.push(Token::new(&current_token));
    }

    tokens
}

fn evaluate(expression: &str) -> Option<String> {
    let tokens = parse(expression);

    if let Some(operator) = tokens
        .iter()
        .find(|token| token.token_type == TokenType::Operator)
    {
        return match operator.literal.as_str() {
            "+" => {
                if tokens
                    .iter()
                    .filter(|token| token.token_type == TokenType::Operand)
                    .all(|token| token.object_type == Some(ObjectType::Int))
                {
                    let sum: i64 = tokens
                        .iter()
                        .filter(|token| token.token_type == TokenType::Operand)
                        .filter_map(|token| token.literal.parse::<i64>().ok())
                        .sum();

                    Some(sum.to_string())
                } else if tokens
                    .iter()
                    .filter(|token| token.token_type == TokenType::Operand)
                    .all(|token| token.object_type == Some(ObjectType::Float))
                {
                    let sum: f64 = tokens
                        .iter()
                        .filter(|token| token.token_type == TokenType::Operand)
                        .filter_map(|token| token.literal.parse::<f64>().ok())
                        .sum();

                    Some(sum.to_string())
                } else if tokens
                    .iter()
                    .filter(|token| token.token_type == TokenType::Operand)
                    .all(|token| token.object_type == Some(ObjectType::String))
                {
                    let mut new_string = String::new();
                    for string in tokens
                        .into_iter()
                        .filter(|token| token.token_type == TokenType::Operand)
                        .filter_map(|token| token.unwrap_string())
                    {
                        new_string.push_str(&string);
                    }
                    Some(new_string)
                } else {
                    None
                }
            }
            _ => None,
        };
    }

    // For now, 'evaluation' will consist of just returning the first token.
    tokens.first().map(|result| result.literal.clone())
}

fn repl(mut writer: impl Write, mut reader: impl io::BufRead) {
    loop {
        let expression = read_expression(&mut writer, &mut reader);
        if is_terminator(&expression) {
            break;
        }

        if let Some(result) = evaluate(&expression) {
            println!("{result}");
        }
    }
}

fn read_file(path: &str) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(String::from)
        .collect()
}

fn process_file(lines: &[String], writer: &mut impl Write) {
    for expression in lines {
        if let Some(result) = evaluate(&expression) {
            writeln!(writer, "{result}").expect("Failed to write result.");
        }
    }
}

fn main() {
    match env::args().nth(1) {
        Some(path) => process_file(&read_file(&path), &mut io::stdout()),
        None => repl(io::stdout(), io::stdin().lock()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_floats_eq(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-10);
    }

    #[test]
    fn test_display_prompt() {
        let mut output: Vec<u8> = Vec::new();
        display_prompt(">> ", &mut output);
        assert_eq!(output, b">> ");
    }

    #[test]
    fn test_read_expression() {
        // simple expression
        let mut input = "123".as_bytes();
        let mut output: Vec<u8> = Vec::new();
        let expression = read_expression(&mut output, &mut input);
        assert_eq!(expression, "123");

        // with whitespace to split
        let mut input = "123 456".as_bytes();
        let mut output: Vec<u8> = Vec::new();
        let expression = read_expression(&mut output, &mut input);
        assert_eq!(expression, "123 456");
    }

    #[test]
    fn test_is_terminator() {
        assert!(is_terminator("quit"));
        assert!(!is_terminator("exit"));
    }

    #[test]
    fn test_token() {
        let token = Token::new("123");
        assert_eq!(token.literal, "123");
    }

    #[test]
    fn test_parse() {
        let input_expressions = "123 456";

        let tokens = vec![Token::new("123"), Token::new("456")];

        assert_eq!(parse(&input_expressions), tokens);
    }

    #[test]
    fn test_evaluate() {
        let input_expressions = "123 456";
        assert_eq!(evaluate(&input_expressions), Some(String::from("123")));
    }

    #[test]
    fn test_add_integers() {
        assert_eq!(evaluate("1 + 2"), Some(String::from("3")));
    }

    #[test]
    fn test_add_floats() {
        let result: f64 = evaluate("1.4 + 2.3").unwrap().parse().unwrap();
        let expected: f64 = 3.7;

        assert_floats_eq(result, expected);
    }
    #[test]
    fn test_file() {
        let lines = read_file("test_input.txt");
        let mut output: Vec<u8> = Vec::new();
        process_file(&lines, &mut output);

        let results = vec!["3", "5", "9", "7.7", "hello world"];

        let expected = results.join("\n") + "\n";
        assert_eq!(output, expected.as_bytes());
    }

    #[test]
    fn test_unwrap_string() {
        assert_eq!(
            Token::new("\"hello\"").unwrap_string(),
            Some(String::from("hello"))
        )
    }

    #[test]
    fn test_add_strings() {
        assert_eq!(
            evaluate("\"hello\" + \" world\""),
            Some(String::from("hello world"))
        );
    }
}
