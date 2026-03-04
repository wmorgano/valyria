use std::env;
use std::fmt::Display;
use std::io::{self, Write};
use std::ops::Add;

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

#[derive(Debug, Clone, PartialEq)]
struct Token {
    literal: String,
    token_type: TokenType,
    object: Option<Object>,
}

impl Token {
    fn new(literal: &str) -> Token {
        let mut token_type = TokenType::Operand;
        let mut object = None;
        if matches!(literal, "+") || matches!(literal, "=") {
            token_type = TokenType::Operator;
        } else {
            if let Ok(val) = literal.parse::<i64>() {
                object = Some(Object::new(Box::new(IntObject::new(val))));
            } else if let Ok(val) = literal.parse::<f64>() {
                object = Some(Object::new(Box::new(FloatObject::new(val))));
            } else if literal.starts_with('"') && literal.ends_with('"') {
                object = Some(Object::new(Box::new(StringObject::new(literal[1..literal.len() - 1]
                    .to_string()))));
            }
        }

        Token {
            literal: String::from(literal),
            token_type,
            object,
        }
    }
}

trait ObjectTrait: std::fmt::Debug {
    fn to_string(&self) -> String;
    fn add(&self, other: &dyn ObjectTrait) -> Box<dyn ObjectTrait>;
    fn as_any(&self) -> &dyn std::any::Any;
    fn clone_obj(&self) -> Box<dyn ObjectTrait>;
    fn equals(&self, other: &dyn ObjectTrait) -> bool;
}

// Generic Object Structure

struct Object {
    obj: Box<dyn ObjectTrait>,
}

impl Object {
    fn new(obj: Box<dyn ObjectTrait>) -> Self {
        Object { obj }
    }
}

impl Clone for Object {
    fn clone(&self) -> Self {
        Object {
            obj: self.obj.clone_obj(),
        }
    }
}

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.obj.fmt(f)
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.obj.equals(other.obj.as_ref())
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.obj.to_string())
    }
}

impl Add for Object {
    type Output = Object;

    fn add(self, rhs: Object) -> Object {
        Object::new(self.obj.add(rhs.obj.as_ref()))
    }
}

// Int Object

#[derive(Debug, Clone, PartialEq)]
struct IntObject {
    value: i64,
}

impl IntObject {
    fn new(value: i64) -> IntObject {
        IntObject { value }
    }
}


impl ObjectTrait for IntObject {
    fn to_string(&self) -> String {
        self.value.to_string()
    }

    fn add(&self, other: &dyn ObjectTrait) -> Box<dyn ObjectTrait> {
        if let Some(other_int) = other.as_any().downcast_ref::<IntObject>() {
            Box::new(IntObject::new(self.value + other_int.value))
        } else if let Some(other_float) = other.as_any().downcast_ref::<FloatObject>() {
            Box::new(FloatObject::new(self.value as f64 + other_float.value))
        } else {
            panic!("Cannot add {:?} to Int", other)
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn clone_obj(&self) -> Box<dyn ObjectTrait> {
        Box::new(self.clone())
    }

    fn equals(&self, other: &dyn ObjectTrait) -> bool {
        if let Some(other_int) = other.as_any().downcast_ref::<IntObject>() {
            self.value == other_int.value
        } else {
            false
        }
    }
}

// Float Object

#[derive(Debug, Clone, PartialEq)]
struct FloatObject {
    value: f64,
}

impl FloatObject {
    fn new(value: f64) -> FloatObject {
        FloatObject { value }
    }
}

impl ObjectTrait for FloatObject {
    fn to_string(&self) -> String {
        self.value.to_string()
    }

    fn add(&self, other: &dyn ObjectTrait) -> Box<dyn ObjectTrait> {
        if let Some(other_float) = other.as_any().downcast_ref::<FloatObject>() {
            Box::new(FloatObject::new(self.value + other_float.value))
        } else if let Some(other_int) = other.as_any().downcast_ref::<IntObject>() {
            Box::new(FloatObject::new(self.value + other_int.value as f64))
        } else {
            panic!("Cannot add {:?} to Float", other)
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn clone_obj(&self) -> Box<dyn ObjectTrait> {
        Box::new(self.clone())
    }

    fn equals(&self, other: &dyn ObjectTrait) -> bool {
        if let Some(other_float) = other.as_any().downcast_ref::<FloatObject>() {
            self.value == other_float.value
        } else {
            false
        }
    }
}

// String Object

#[derive(Debug, Clone, PartialEq)]
struct StringObject {
    value: String,
}

impl StringObject {
    fn new(value: String) -> StringObject {
        StringObject { value }
    }
}

impl ObjectTrait for StringObject {
    fn to_string(&self) -> String {
        self.value.clone()
    }

    fn add(&self, other: &dyn ObjectTrait) -> Box<dyn ObjectTrait> {
        if let Some(other_string) = other.as_any().downcast_ref::<StringObject>() {
            Box::new(StringObject::new(self.value.clone() + &other_string.value))
        } else {
            panic!("Cannot add {:?} to String", other)
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn clone_obj(&self) -> Box<dyn ObjectTrait> {
        Box::new(self.clone())
    }

    fn equals(&self, other: &dyn ObjectTrait) -> bool {
        if let Some(other_string) = other.as_any().downcast_ref::<StringObject>() {
            self.value == other_string.value
        } else {
            false
        }
    }
}

// Variables

struct Variable {
    name: String,
    object: Object,
}

impl Variable {
    fn new(name: String, object: Object) -> Variable {
        Variable { name, object }
    }

    fn to_string(&self) -> String {
        format!("{} == {}", self.name, self.object)
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

    let operator = tokens
        .iter()
        .find(|t| t.token_type == TokenType::Operator);

    match operator {
        Some(op) => match op.literal.as_str() {
            "+" => {
                let mut operands = tokens
                    .into_iter()
                    .filter(|t| t.token_type == TokenType::Operand);

                let lhs = operands.next()
                    .expect("Error getting first operand from Add expression.")
                    .object
                    .expect("Error unwrapping object from first operand in Add expression.");
                let rhs = operands.next()
                    .expect("Error getting second operand from Add expression.")
                    .object
                    .expect("Error unwrapping object from second operand in Add expression.");

                let result = lhs + rhs;

                Some(result.to_string())
            },
            "=" => {
                let mut operands = tokens
                    .into_iter()
                    .filter(|t| t.token_type == TokenType::Operand);

                let lhs_token = operands.next()
                    .expect("Error getting variable name from assignment expression.");
                let rhs_token = operands.next()
                    .expect("Error getting variable value from assignment expression.");

                let var_name = lhs_token.literal.clone();
                let var_value = rhs_token.object
                    .expect("Error unwrapping object for value from assignment expression.");

                let var = Variable::new(var_name, var_value);

                Some(var.to_string())
            }
            _ => None,
        },

        // Self-evaluating expression
        None => {
            let operand = tokens
                .into_iter()
                .find(|t| t.token_type == TokenType::Operand)
                .expect("Found no operand in a self-evaluating expression.")
                .object
                .expect("Error unwrapping object from self-evaluating expression.");

            Some(operand.to_string())
        }
    }
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
        if let Some(result) = evaluate(expression) {
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
    fn test_add_strings() {
        assert_eq!(
            evaluate("\"hello\" + \" world\""),
            Some(String::from("hello world"))
        );
    }

    #[test]
    fn test_objs() {
        let lhs = Object::new(Box::new(IntObject::new(123)));
        let rhs = Object::new(Box::new(IntObject::new(456)));
        assert_eq!(lhs.to_string(), "123");
        assert_eq!(rhs.to_string(), "456");

        let expected_result = Object::new(Box::new(IntObject::new(579)));
        assert_eq!(lhs + rhs, expected_result);
    }

    #[test]
    fn test_variable() {
        let var = Variable::new(String::from("x"), Object::new(Box::new(IntObject::new(123))));
        assert_eq!(var.to_string(), "x == 123");
    }

    #[test]
    fn test_variable_assignment() {
        let result = evaluate("x = 456");
        assert_eq!(result, Some(String::from("x == 456")));
    }
    
    #[test]
    fn test_file() {
        let lines = read_file("test_input.txt");
        let mut output: Vec<u8> = Vec::new();
        process_file(&lines, &mut output);

        let results = vec!["3", "5", "9", "7.7", "hello world", "x == 7"];

        let expected = results.join("\n") + "\n";
        assert_eq!(output, expected.as_bytes());
    }
}
