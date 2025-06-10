pub mod token;
pub mod lexer;
pub mod ast;
pub mod parser;
pub mod semantic;
pub mod interpreter;

use lexer::Lexer;
use parser::Parser;
use ast::Program;
use semantic::ParseError;
use interpreter::Interpreter;

pub fn parse_coral(input: &str) -> Result<Program, Vec<ParseError>> {
    let lexer = Lexer::new(input.to_string());
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program();
    let errors = parser.errors();
    
    if errors.is_empty() {
        Ok(program)
    } else {
        Err(errors.clone())
    }
}

pub fn run_coral(input: &str) -> Result<interpreter::Value, Box<dyn std::error::Error>> {
    let program = match parse_coral(input) {
        Ok(prog) => prog,
        Err(errors) => {
            let error_msg = errors.iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(format!("Parse errors: {}", error_msg).into());
        }
    };
    
    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&program)?;
    
    // Print any output
    for line in interpreter.get_output() {
        println!("{}", line);
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coral_parsing() {
        let input = r#"message is 'hello coral'
count is 42

fn greet with name, greeting 'Hello'
    '{greeting}, {name}!'

object user
    name
    email
    age ? 0
    
    introduce
        'Hi, I am {name}'

store task
    description
    priority ? 1
    complete ? no
    
    make
        log create $description, $priority"#;
        
        match parse_coral(input) {
            Ok(program) => {
                println!("Successfully parsed {} statements", program.statements.len());
                for stmt in &program.statements {
                    println!("Statement: {:?}", stmt);
                }
            }
            Err(errors) => {
                println!("Parse errors:");
                for error in errors {
                    println!("  - {}", error);
                }
                panic!("Parsing failed");
            }
        }
    }

    #[test]
    fn test_coral_execution() {
        let input = r#"x is 10
y is 20
result is x + y
log result"#;
        
        match run_coral(input) {
            Ok(value) => {
                println!("Program executed successfully, result: {}", value);
            }
            Err(error) => {
                println!("Execution error: {}", error);
                panic!("Execution failed");
            }
        }
    }

    #[test]
    fn test_function_with_interpolation() {
        let input = r#"fn greet with name
    'Hello {name}!'

name is 'Coral'
greeting is greet name
log greeting"#;
        
        match run_coral(input) {
            Ok(_) => {
                println!("Function interpolation test passed");
            }
            Err(error) => {
                println!("Error: {}", error);
                panic!("Function test failed");
            }
        }
    }

    #[test]
    fn test_ternary_expressions() {
        let input = r#"age is 25
status is age gt 18 ? 'adult' ! 'minor'
log status"#;
        
        match run_coral(input) {
            Ok(_) => {
                println!("Ternary expression test passed");
            }
            Err(error) => {
                println!("Error: {}", error);
                panic!("Ternary test failed");
            }
        }
    }

    #[test]
    fn test_array_operations() {
        let input = r#"numbers is [1, 2, 3, 4, 5]
first is numbers at 0
second is numbers at 1
log first
log second"#;
        
        match run_coral(input) {
            Ok(_) => {
                println!("Array operations test passed");
            }
            Err(error) => {
                println!("Error: {}", error);
                panic!("Array test failed");
            }
        }
    }

    #[test]
    fn test_object_creation() {
        let input = r#"object person
    name
    age ? 0

user is person
log user"#;
        
        match run_coral(input) {
            Ok(_) => {
                println!("Object creation test passed");
            }
            Err(error) => {
                println!("Error: {}", error);
                panic!("Object test failed");
            }
        }
    }

    #[test]
    fn test_error_handling() {
        let input = "x is undefined_var + 5";
        
        match run_coral(input) {
            Ok(_) => panic!("Should have failed with undefined variable"),
            Err(_) => {
                println!("Error handling test passed - correctly caught undefined variable");
            }
        }
    }
}