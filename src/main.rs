use std::env;
use std::fmt::Display;
use std::io::{self, Write};
use std::ops::Add;
use std::collections::HashMap;

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
enum OperatorType {
    Add,
    Assign,
}


#[derive(Debug, Clone, PartialEq)]
enum LiteralValue {
    Int(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
enum OperandType {
    Literal(LiteralValue),
    Identifier,
}

#[derive(Debug, Clone, PartialEq)]
enum TokenType {
    Operator(OperatorType),
    Operand(OperandType),
}

#[derive(Debug, Clone, PartialEq)]
struct Token {
    representation: String,
    token_type: TokenType,
}

impl Token {
    fn new(literal: &str) -> Token {
        let mut representation = String::from(literal);

        let token_type = match literal {
            "+" => TokenType::Operator(OperatorType::Add),
            "=" => TokenType::Operator(OperatorType::Assign),
            _ => {
                if let Ok(val) = literal.parse::<i64>() {
                    TokenType::Operand(OperandType::Literal(LiteralValue::Int(val)))
                } else if let Ok(val) = literal.parse::<f64>() {
                    TokenType::Operand(OperandType::Literal(LiteralValue::Float(val)))
                } else if literal.starts_with('"') && literal.ends_with('"') {
                    representation = String::from(&literal[1..literal.len() - 1]);
                    TokenType::Operand(OperandType::Literal(
                        LiteralValue::String(representation.clone())))
                } else {
                    TokenType::Operand(OperandType::Identifier)
                }
            }
        };

        Token { representation, token_type }
    }
}

trait ObjectTrait: std::fmt::Debug {
    fn to_string(&self) -> String;
    fn add(&self, other: &dyn ObjectTrait) -> Box<dyn ObjectTrait>;
    fn as_any(&self) -> &dyn std::any::Any;
    fn clone_obj(&self) -> Box<dyn ObjectTrait>;
    fn equals(&self, other: &dyn ObjectTrait) -> bool;
}

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

struct Environment {
    variables: HashMap<String, Object>,
}

impl Environment {
    fn new() -> Self {
        Environment {
            variables: HashMap::new(),
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

fn get_token_value(token: &Token, environment: &Environment) -> Result<Object, String> {
    match &token.token_type {
        TokenType::Operand(OperandType::Literal(literal)) => {
            match literal {
                LiteralValue::Int(val) => Ok(Object::new(Box::new(IntObject::new(*val)))),
                LiteralValue::Float(val) => Ok(Object::new(Box::new(FloatObject::new(*val)))),
                LiteralValue::String(val) => Ok(Object::new(Box::new(StringObject::new(val.clone())
                ))),
            }
        }
        TokenType::Operand(OperandType::Identifier) => {
            environment.variables.get(&token.representation).cloned()
                .ok_or_else(|| format!("Undefined variable: {}", token.representation))
        }
        _ => Err(format!("Invalid token: {}", token.representation)),
    }
}

fn evaluate(expression: &str, environment: &mut Environment) -> Result<Option<String>, String> {
    let tokens = parse(expression);

    match &tokens[..] {
        [] => Err(String::from("Empty expression")),
        [token] => {
            let value = get_token_value(token, environment)?;
            Ok(Some(value.to_string()))
        },
        [lhs, operator, rhs] => {
            match &operator.token_type {
                TokenType::Operator(OperatorType::Add) => {
                    let lhs_object = get_token_value(lhs, environment)?;
                    let rhs_object = get_token_value(rhs, environment)?;

                    let result = lhs_object + rhs_object;
                    Ok(Some(result.to_string()))
                },
                TokenType::Operator(OperatorType::Assign) => {
                    let value = get_token_value(rhs, environment)?;
                    environment.variables.insert(lhs.representation.clone(), value.clone());
                    Ok(None)
                }
                _ => Err(format!("Unknown operator: {}", operator.representation))
            }
        }
        _ => Err(String::from("Invalid expression"))
    }
}

fn repl(mut writer: impl Write, mut reader: impl io::BufRead, environment: &mut Environment) {
    loop {
        let expression = read_expression(&mut writer, &mut reader);
        if is_terminator(&expression) {
            break;
        }

        match evaluate(&expression, environment) {
            Ok(result) => {
                if let Some(result) = result {
                    writeln!(writer, "{result}").expect("Failed to write evaluation to REPL.");
                }
            }
            Err(error) => println!("Error: {error}"),
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

fn process_file(lines: &[String], writer: &mut impl Write, environment: &mut Environment) {
    for expression in lines {
        match evaluate(expression, environment) {
            Ok(result) => {
                if let Some(result) = result {
                    writeln!(writer, "{result}").expect("Failed to write evaluation to file.");
                }
            },
            Err(error) => writeln!(writer, "Error: {error}").expect("Failed to write error."),
        }
    }
}

fn main() {
    let mut environment = Environment::new();

    match env::args().nth(1) {
        Some(path) => process_file(&read_file(&path), &mut io::stdout(), &mut environment),
        None => repl(io::stdout(), io::stdin().lock(), &mut environment),
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
        assert_eq!(token.representation, "123");
    }

    #[test]
    fn test_parse() {
        let input_expressions = "123 456";

        let tokens = vec![Token::new("123"), Token::new("456")];

        assert_eq!(parse(&input_expressions), tokens);
    }

    #[test]
    fn test_evaluate() {
        let input_expressions = "123";
        assert_eq!(evaluate(&input_expressions, &mut Environment::new()),
            Ok(Some(String::from("123"))));
    }

    #[test]
    fn test_add_integers() {
        assert_eq!(evaluate("1 + 2", &mut Environment::new()),
                   Ok(Some(String::from("3"))));
    }

    #[test]
    fn test_add_floats() {
        let result: f64 = evaluate("1.4 + 2.3", &mut Environment::new())
            .unwrap().unwrap().parse().unwrap();
        let expected: f64 = 3.7;

        assert_floats_eq(result, expected);
    }

    #[test]
    fn test_add_strings() {
        assert_eq!(
            evaluate("\"hello\" + \" world\"", &mut Environment::new()),
            Ok(Some(String::from("hello world")))
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
        let mut env = Environment::new();
        let name = String::from("x");
        let val = Object::new(Box::new(IntObject::new(123)));

        env.variables.insert(name, val.clone());
        assert_eq!(env.variables.get("x"), Some(&val));
    }
    
    #[test]
    fn test_file() {
        let lines = read_file("test_input.txt");
        let mut output: Vec<u8> = Vec::new();
        process_file(&lines, &mut output, &mut Environment::new());

        let results = vec!["3", "5", "9", "7.7", "hello world"];

        let expected = results.join("\n") + "\n";
        assert_eq!(output, expected.as_bytes());
    }
}
