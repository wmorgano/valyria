use std::io::{self, Write};

fn display_prompt(prompt: String, writer: &mut impl io::Write) {
    write!(writer, "{}", prompt).expect("Failed to write prompt.");
}

fn read_expression(writer: &mut impl io::Write, reader: &mut impl io::BufRead) -> Vec<String> {
    let prompt = String::from(">> ");
    display_prompt(prompt, writer);
    io::stdout()
        .flush()
        .expect("Failed to flush stdout stream.");

    let mut input = String::new();

    reader
        .read_line(&mut input)
        .expect("Failed to parse input.");

    input.split_whitespace().map(String::from).collect()
}

fn is_terminator(expression: &Vec<String>) -> bool {
    match expression.first() {
        None => false,
        Some(command) => command == "quit",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    literal: String,
}

impl Token {
    fn new(literal: String) -> Token {
        Token { literal }
    }
}

fn tokenize(expression: &Vec<String>) -> Vec<Token> {
    expression
        .iter()
        .map(|token| Token::new(token.clone()))
        .collect()
}

fn evaluate(expression: &Vec<String>) -> Result<String, &'static str> {
    let tokens = tokenize(expression);

    // For now, 'evaluation' will consist of just returning the first token.
    match tokens.first() {
        None => Err("Tokenized expression is empty."),
        Some(result) => Ok(result.literal.clone()),
    }
}

fn repl(mut writer: impl io::Write, mut reader: impl io::BufRead) {
    loop {
        let expression = read_expression(&mut writer, &mut reader);
        if is_terminator(&expression) {
            break;
        }

        match evaluate(&expression) {
            Ok(result) => println!("{result}"),
            Err(e) => println!("{}", e),
        }
    }
}

fn main() {
    repl(io::stdout(), io::stdin().lock())
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_display_prompt() {
        let mut output: Vec<u8> = Vec::new();
        display_prompt(String::from(">> "), &mut output);
        assert_eq!(output, b">> ");
    }

    #[test]
    fn test_read_expression() {
        // simple expression
        let mut input = "123".as_bytes();
        let mut output: Vec<u8> = Vec::new();
        let expression = read_expression(&mut output, &mut input);
        assert_eq!(expression, vec!["123"]);

        // with whitespace to split
        let mut input = "123 456".as_bytes();
        let mut output: Vec<u8> = Vec::new();
        let expression = read_expression(&mut output, &mut input);
        assert_eq!(expression, vec!["123", "456"]);
    }

    #[test]
    fn test_is_terminator() {
        assert!(is_terminator(&vec![String::from("quit")]));
        assert!(!is_terminator(&vec![String::from("exit")]));
    }

    #[test]
    fn test_token() {
        let token = Token::new(String::from("123"));
        assert_eq!(token.literal, "123");
    }

    #[test]
    fn test_tokenize() {
        let input_expressions = vec![String::from("123"), String::from("456")];

        let tokens = vec![
            Token::new(String::from("123")),
            Token::new(String::from("456")),
        ];

        assert_eq!(tokenize(&input_expressions), tokens);
    }

    #[test]
    fn test_evaluate() {
        let input_expressions = vec![String::from("123"), String::from("456")];
        assert_eq!(evaluate(&input_expressions), Ok(String::from("123")));
   } 
}
