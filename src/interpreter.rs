use std::collections::HashMap;
use std::fmt;
use std::error::Error;
use crate::ast::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Function {
        name: String,
        params: Vec<Parameter>,
        body: Vec<Statement>,
        closure: Environment,
    },
    None,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", if *b { "yes" } else { "no" }),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Value::Object(obj) => {
                let items: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                write!(f, "{{{}}}", items.join(", "))
            }
            Value::Function { name, .. } => write!(f, "<function {}>", name),
            Value::None => write!(f, "empty"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    vars: HashMap<String, Value>,
    parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            vars: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Environment) -> Self {
        Environment {
            vars: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.vars.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.vars.get(name) {
            Some(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            None
        }
    }

    pub fn set(&mut self, name: &str, value: Value) -> bool {
        if self.vars.contains_key(name) {
            self.vars.insert(name.to_string(), value);
            true
        } else if let Some(parent) = &mut self.parent {
            parent.set(name, value)
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    UndefinedVariable(String),
    TypeMismatch(String),
    DivisionByZero,
    IndexOutOfBounds,
    InvalidOperation(String),
    FunctionNotFound(String),
    ArgumentMismatch(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            RuntimeError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            RuntimeError::DivisionByZero => write!(f, "Division by zero"),
            RuntimeError::IndexOutOfBounds => write!(f, "Index out of bounds"),
            RuntimeError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            RuntimeError::FunctionNotFound(name) => write!(f, "Function not found: {}", name),
            RuntimeError::ArgumentMismatch(msg) => write!(f, "Argument mismatch: {}", msg),
        }
    }
}

impl Error for RuntimeError {}

pub struct Interpreter {
    environment: Environment,
    output: Vec<String>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut env = Environment::new();
        
        // Built-in functions
        env.define("log".to_string(), Value::Function {
            name: "log".to_string(),
            params: vec![Parameter { name: "message".to_string(), default_value: None }],
            body: vec![],
            closure: Environment::new(),
        });
        
        Interpreter {
            environment: env,
            output: Vec::new(),
        }
    }

    pub fn evaluate(&mut self, program: &Program) -> Result<Value, RuntimeError> {
        let mut last_value = Value::None;
        
        for statement in &program.statements {
            last_value = self.eval_statement(statement)?;
        }
        
        Ok(last_value)
    }

    pub fn get_output(&self) -> &[String] {
        &self.output
    }

    fn eval_statement(&mut self, stmt: &Statement) -> Result<Value, RuntimeError> {
        match stmt {
            Statement::Assignment(Assignment { identifier, value }) => {
                let val = self.eval_expression(value)?;
                self.environment.define(identifier.clone(), val.clone());
                Ok(val)
            }
            Statement::FunctionDef(FunctionDef { name, parameters, body }) => {
                let func = Value::Function {
                    name: name.clone(),
                    params: parameters.clone(),
                    body: body.clone(),
                    closure: self.environment.clone(),
                };
                self.environment.define(name.clone(), func.clone());
                Ok(func)
            }
            Statement::ExpressionStmt(expr) => self.eval_expression(expr),
            Statement::ObjectDef(ObjectDef { name, properties, methods: _ }) => {
                let mut obj = HashMap::new();
                
                for prop in properties {
                    let value = if let Some(default) = &prop.default_value {
                        self.eval_expression(default)?
                    } else {
                        Value::None
                    };
                    obj.insert(prop.name.clone(), value);
                }
                
                let obj_val = Value::Object(obj);
                self.environment.define(name.clone(), obj_val.clone());
                Ok(obj_val)
            }
            _ => Ok(Value::None), // Stub for other statement types
        }
    }

    fn eval_expression(&mut self, expr: &Expression) -> Result<Value, RuntimeError> {
        match expr {
            Expression::Integer(i) => Ok(Value::Integer(*i)),
            Expression::Float(f) => Ok(Value::Float(*f)),
            Expression::StringLiteral(s) => Ok(Value::String(s.clone())),
            Expression::Boolean(b) => Ok(Value::Boolean(*b)),
            Expression::Empty => Ok(Value::None),
            Expression::Now => Ok(Value::String("now".to_string())), // Placeholder
            
            Expression::Identifier(name) => {
                self.environment.get(name)
                    .ok_or_else(|| RuntimeError::UndefinedVariable(name.clone()))
            }
            
            Expression::Array(elements) => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.eval_expression(elem)?);
                }
                Ok(Value::Array(values))
            }
            
            Expression::InterpolatedString(template) => {
                // Simple string interpolation - replace {var} with variable values
                let mut result = template.clone();
                let mut start = 0;
                
                while let Some(open) = result[start..].find('{') {
                    let open_pos = start + open;
                    if let Some(close) = result[open_pos..].find('}') {
                        let close_pos = open_pos + close;
                        let var_name = &result[open_pos + 1..close_pos];
                        
                        if let Some(value) = self.environment.get(var_name) {
                            let replacement = value.to_string();
                            result.replace_range(open_pos..close_pos + 1, &replacement);
                            start = open_pos + replacement.len();
                        } else {
                            return Err(RuntimeError::UndefinedVariable(var_name.to_string()));
                        }
                    } else {
                        break;
                    }
                }
                
                Ok(Value::String(result))
            }
            
            Expression::Binary { left, operator, right } => {
                let left_val = self.eval_expression(left)?;
                let right_val = self.eval_expression(right)?;
                self.eval_binary_op(&left_val, operator, &right_val)
            }
            
            Expression::Unary { operator, operand } => {
                let val = self.eval_expression(operand)?;
                self.eval_unary_op(operator, &val)
            }
            
            Expression::Ternary { condition, true_expr, false_expr } => {
                let cond = self.eval_expression(condition)?;
                if self.is_truthy(&cond) {
                    self.eval_expression(true_expr)
                } else {
                    self.eval_expression(false_expr)
                }
            }
            
            Expression::FunctionCall { name, args, named_args: _ } => {
                self.call_function(name, args)
            }
            
            Expression::Instantiation { type_name, args: _, named_args: _, force_success: _ } => {
                // Simple object instantiation - return the object template
                if let Some(obj_val) = self.environment.get(type_name) {
                    Ok(obj_val)
                } else {
                    Err(RuntimeError::UndefinedVariable(type_name.clone()))
                }
            }
            
            Expression::LogOperation { message } => {
                let msg = self.eval_expression(message)?;
                self.output.push(msg.to_string());
                Ok(Value::None)
            }
            
            Expression::ArrayAccess { array, index, use_at_keyword: _ } => {
                let arr_val = self.eval_expression(array)?;
                let idx_val = self.eval_expression(index)?;
                
                match (arr_val, idx_val) {
                    (Value::Array(arr), Value::Integer(idx)) => {
                        let i = if idx < 0 {
                            arr.len() as i64 + idx
                        } else {
                            idx
                        } as usize;
                        
                        arr.get(i).cloned().ok_or(RuntimeError::IndexOutOfBounds)
                    }
                    _ => Err(RuntimeError::TypeMismatch("Array access requires array and integer".to_string()))
                }
            }
            
            Expression::MethodCall { object, method, args, named_args: _, force_call: _, chaining: _ } => {
                // Evaluate the object expression first
                let mut obj_val = self.eval_expression(object)?; // Use mut if we might modify it (for future env updates)

                // Note: For true mutability of collections stored in variables, this logic would need to:
                // 1. Check if 'object' is an Identifier.
                // 2. If so, operate on the value from the environment and then use `self.environment.set()`.
                // For now, operations return new Value instances or operate on copies.

                match method.as_str() {
                    "push" | "add" => {
                        if let Value::Array(mut arr_copy) = obj_val.clone() { // Operate on a clone for now
                            if args.len() != 1 {
                                return Err(RuntimeError::ArgumentMismatch(format!("'{method}' expects 1 argument, got {}", args.len())));
                            }
                            let item_val = self.eval_expression(&args[0])?;
                            arr_copy.push(item_val);
                            Ok(Value::Array(arr_copy)) // Returns a new array
                        } else {
                            Err(RuntimeError::TypeMismatch(format!("'{method}' can only be called on arrays")))
                        }
                    }
                    "pop" => {
                        if let Value::Array(mut arr_copy) = obj_val.clone() { // Operate on a clone
                            if !args.is_empty() {
                                return Err(RuntimeError::ArgumentMismatch(format!("'{method}' expects 0 arguments, got {}", args.len())));
                            }
                            arr_copy.pop().ok_or_else(|| RuntimeError::InvalidOperation(format!("Cannot '{method}' from empty array")))
                            // Ok(popped_value) - this is fine, pop returns the value
                            // If we were to modify in place: self.environment.set(obj_name, Value::Array(arr_copy))
                        } else {
                            Err(RuntimeError::TypeMismatch(format!("'{method}' can only be called on arrays")))
                        }
                    }
                    "length" | "size" => {
                        if !args.is_empty() {
                            return Err(RuntimeError::ArgumentMismatch(format!("'{method}' expects 0 arguments, got {}", args.len())));
                        }
                        match obj_val {
                            Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
                            Value::String(s) => Ok(Value::Integer(s.len() as i64)),
                            _ => Err(RuntimeError::TypeMismatch(format!("'{method}' can only be called on arrays or strings")))
                        }
                    }
                    "contains" => {
                        if let Value::Array(arr) = &obj_val {
                            if args.len() != 1 {
                                return Err(RuntimeError::ArgumentMismatch(format!("'{method}' expects 1 argument, got {}", args.len())));
                            }
                            let item_to_find = self.eval_expression(&args[0])?;
                            Ok(Value::Boolean(arr.contains(&item_to_find)))
                        } else {
                            Err(RuntimeError::TypeMismatch(format!("'{method}' can only be called on arrays")))
                        }
                    }
                    "remove_at" => { // Assuming 'remove' means remove_at for now
                        if let Value::Array(mut arr_copy) = obj_val.clone() {
                            if args.len() != 1 {
                                return Err(RuntimeError::ArgumentMismatch(format!("'{method}' expects 1 argument (index), got {}", args.len())));
                            }
                            let index_val = self.eval_expression(&args[0])?;
                            if let Value::Integer(idx) = index_val {
                                if idx < 0 || idx as usize >= arr_copy.len() {
                                    return Err(RuntimeError::IndexOutOfBounds);
                                }
                                arr_copy.remove(idx as usize);
                                Ok(Value::Array(arr_copy)) // Returns a new array
                            } else {
                                Err(RuntimeError::TypeMismatch("Array index must be an integer".to_string()))
                            }
                        } else {
                            Err(RuntimeError::TypeMismatch(format!("'{method}' can only be called on arrays")))
                        }
                    }
                    // Placeholder for user-defined methods on objects
                    // This would involve looking up 'method' in the object's definition (if Value::Object stored methods or a type link)
                    // and then calling it similar to how functions are called.
                    _ => {
                        // If it's not a built-in array/string method, and if obj_val is an Object,
                        // we would eventually look for user-defined methods here.
                        // For now, if not a recognized built-in, assume it's an error or a field access (if no args).
                        if args.is_empty() {
                             // Attempt to access a field if obj_val is an Object
                            if let Value::Object(map) = obj_val {
                                return map.get(method).cloned().ok_or_else(|| RuntimeError::InvalidOperation(format!("Method or field '{}' not found on object", method)));
                            }
                        }
                        Err(RuntimeError::InvalidOperation(format!("Method '{}' not found or not applicable", method)))
                    }
                }
            }
            
            Expression::ObjectLiteral(pairs) => {
                let mut obj = std::collections::HashMap::new();
                for (key, value_expr) in pairs {
                    let value = self.eval_expression(value_expr)?;
                    obj.insert(key.clone(), value);
                }
                Ok(Value::Object(obj))
            }
            
            Expression::ParameterRef(name) => {
                // Handle parameter references like $id, $0, etc.
                self.environment.get(name)
                    .ok_or_else(|| RuntimeError::UndefinedVariable(format!("Parameter ${}", name)))
            }
            
            _ => Ok(Value::None), // Stub for other expression types
        }
    }

    fn eval_binary_op(&self, left: &Value, op: &BinaryOp, right: &Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => {
                match op {
                    BinaryOp::Add => Ok(Value::Integer(l + r)),
                    BinaryOp::Subtract => Ok(Value::Integer(l - r)),
                    BinaryOp::Multiply => Ok(Value::Integer(l * r)),
                    BinaryOp::Divide => {
                        if *r == 0 { Err(RuntimeError::DivisionByZero) } 
                        else { Ok(Value::Integer(l / r)) }
                    }
                    BinaryOp::Modulo => {
                        if *r == 0 { Err(RuntimeError::DivisionByZero) } 
                        else { Ok(Value::Integer(l % r)) }
                    }
                    BinaryOp::Power => Ok(Value::Integer(l.pow(*r as u32))),
                    BinaryOp::Equal | BinaryOp::Equals => Ok(Value::Boolean(l == r)),
                    BinaryOp::NotEqual => Ok(Value::Boolean(l != r)),
                    BinaryOp::GreaterThan => Ok(Value::Boolean(l > r)),
                    BinaryOp::LessThan => Ok(Value::Boolean(l < r)),
                    BinaryOp::GreaterThanOrEqual => Ok(Value::Boolean(l >= r)),
                    BinaryOp::LessThanOrEqual => Ok(Value::Boolean(l <= r)),
                    _ => Err(RuntimeError::InvalidOperation(format!("Unsupported operation: {:?}", op)))
                }
            }
            (Value::Float(l), Value::Float(r)) => {
                match op {
                    BinaryOp::Add => Ok(Value::Float(l + r)),
                    BinaryOp::Subtract => Ok(Value::Float(l - r)),
                    BinaryOp::Multiply => Ok(Value::Float(l * r)),
                    BinaryOp::Divide => {
                        if *r == 0.0 { Err(RuntimeError::DivisionByZero) } 
                        else { Ok(Value::Float(l / r)) }
                    }
                    BinaryOp::Power => Ok(Value::Float(l.powf(*r))),
                    BinaryOp::Equal | BinaryOp::Equals => Ok(Value::Boolean((l - r).abs() < f64::EPSILON)),
                    BinaryOp::NotEqual => Ok(Value::Boolean((l - r).abs() >= f64::EPSILON)),
                    BinaryOp::GreaterThan => Ok(Value::Boolean(l > r)),
                    BinaryOp::LessThan => Ok(Value::Boolean(l < r)),
                    BinaryOp::GreaterThanOrEqual => Ok(Value::Boolean(l >= r)),
                    BinaryOp::LessThanOrEqual => Ok(Value::Boolean(l <= r)),
                    _ => Err(RuntimeError::InvalidOperation(format!("Unsupported operation: {:?}", op)))
                }
            }
            (Value::String(l), Value::String(r)) => {
                match op {
                    BinaryOp::Add => Ok(Value::String(format!("{}{}", l, r))),
                    BinaryOp::Equal | BinaryOp::Equals => Ok(Value::Boolean(l == r)),
                    BinaryOp::NotEqual => Ok(Value::Boolean(l != r)),
                    _ => Err(RuntimeError::InvalidOperation(format!("Unsupported string operation: {:?}", op)))
                }
            }
            (Value::Boolean(l), Value::Boolean(r)) => {
                match op {
                    BinaryOp::And => Ok(Value::Boolean(*l && *r)),
                    BinaryOp::Or => Ok(Value::Boolean(*l || *r)),
                    BinaryOp::Equal | BinaryOp::Equals => Ok(Value::Boolean(l == r)),
                    BinaryOp::NotEqual => Ok(Value::Boolean(l != r)),
                    _ => Err(RuntimeError::InvalidOperation(format!("Unsupported boolean operation: {:?}", op)))
                }
            }
            _ => Err(RuntimeError::TypeMismatch("Incompatible types for binary operation".to_string()))
        }
    }

    fn eval_unary_op(&self, op: &UnaryOp, operand: &Value) -> Result<Value, RuntimeError> {
        match (op, operand) {
            (UnaryOp::Not, Value::Boolean(b)) => Ok(Value::Boolean(!b)),
            (UnaryOp::Minus, Value::Integer(i)) => Ok(Value::Integer(-i)),
            (UnaryOp::Minus, Value::Float(f)) => Ok(Value::Float(-f)),
            _ => Err(RuntimeError::InvalidOperation(format!("Unsupported unary operation: {:?}", op)))
    }
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Boolean(b) => *b,
            Value::None => false,
            Value::Integer(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            _ => true,
        }
    }

    fn call_function(&mut self, name: &str, args: &[Expression]) -> Result<Value, RuntimeError> {
        match name {
            "log" => {
                if !args.is_empty() {
                    let msg = self.eval_expression(&args[0])?;
                    self.output.push(msg.to_string());
                }
                Ok(Value::None)
            }
            _ => {
                if let Some(func) = self.environment.get(name) {
                    match func {
                        Value::Function { params, body, closure, .. } => {
                            // Handle default parameters properly
                            let mut func_env = Environment::with_parent(closure);
                            
                            // Bind arguments to parameters
                            for (i, param) in params.iter().enumerate() {
                                let val = if i < args.len() {
                                    // Use provided argument
                                    self.eval_expression(&args[i])?
                                } else if let Some(default) = &param.default_value {
                                    // Use default value
                                    self.eval_expression(default)?
                                } else {
                                    return Err(RuntimeError::ArgumentMismatch(
                                        format!("Missing argument for parameter '{}'", param.name)
                                    ));
                                };
                                func_env.define(param.name.clone(), val);
                            }

                            // Check for too many arguments
                            if args.len() > params.len() {
                                return Err(RuntimeError::ArgumentMismatch(
                                    format!("Expected {} arguments, got {}", params.len(), args.len())
                                ));
                            }

                            // Save current environment and switch to function environment
                            let saved_env = std::mem::replace(&mut self.environment, func_env);
                            
                            // Execute function body
                            let mut result = Value::None;
                            for stmt in &body {
                                result = self.eval_statement(stmt)?;
                            }
                            
                            // Restore environment
                            self.environment = saved_env;
                            Ok(result)
                        }
                        _ => Err(RuntimeError::TypeMismatch(format!("{} is not a function", name)))
                    }
                } else {
                    Err(RuntimeError::FunctionNotFound(name.to_string()))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn test_basic_arithmetic() {
        let input = "result is 10 + 5 * 2";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        
        let mut interpreter = Interpreter::new();
        let _ = interpreter.evaluate(&program).unwrap();
        
        let result = interpreter.environment.get("result").unwrap();
        assert_eq!(result, Value::Integer(20));
    }

    #[test]
    fn test_string_interpolation() {
        let input = r#"
name is 'Coral'
message is 'Hello {name}!'
"#;
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        
        let mut interpreter = Interpreter::new();
        let _ = interpreter.evaluate(&program).unwrap();
        
        let message = interpreter.environment.get("message").unwrap();
        assert_eq!(message, Value::String("Hello Coral!".to_string()));
    }

    #[test]
    fn test_function_definition_and_call() {
        // Fix the test to provide both required arguments
        let input = r#"fn add with x, y
    x + y

result is add 5 3"#;
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        
        let mut interpreter = Interpreter::new();
        let _ = interpreter.evaluate(&program).unwrap();
        
        // Check that the function was defined and called correctly
        let add_func = interpreter.environment.get("add");
        assert!(add_func.is_some());
        
        let result = interpreter.environment.get("result").unwrap();
        assert_eq!(result, Value::Integer(8));
    }
}