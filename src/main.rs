use std::io::{self, Write};

fn display_prompt(prompt: &str, writer: &mut impl io::Write) {
    write!(writer, "{prompt}").expect("Failed to write prompt.");
}

fn read_expression(writer: &mut impl io::Write, reader: &mut impl io::BufRead) -> String {
    let prompt = ">> ";
    display_prompt(prompt, writer);
    writer
        .flush()
        .expect("Failed to flush output stream.");

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
    Operand
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    literal: String,
    token_type: TokenType
}

impl Token {
    fn new(literal: &str) -> Token {
        let mut token_type = TokenType::Operand;
        if matches!(literal, "+") {
            token_type = TokenType::Operator;
        }

        Token {
            literal: String::from(literal),
            token_type
        }
    }
}

fn parse(expression: &str) -> Vec<Token> {
    expression.split_whitespace().map(Token::new).collect()
}

fn evaluate(expression: &str) -> Option<String> {
    let tokens = parse(expression);

    if let Some(operator) = tokens.iter().find(|token| token.token_type == TokenType::Operator) {
        return match operator.literal.as_str() {
            "+" => {
                let sum: i64 = tokens.iter()
                    .filter(|token| token.token_type == TokenType::Operand)
                    .filter_map(|token| token.literal.parse::<i64>().ok())
                    .sum();
                Some(sum.to_string())
            }
            _ => None,
        }
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

fn main() {
    repl(io::stdout(), io::stdin().lock())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
