use crate::ast::*;
use std::collections::HashMap;
use crate::codegen::{LLVMValue, LLVMFunction};

/// Symbol table for tracking variable and function declarations
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
    pub functions: HashMap<String, LLVMFunction>,
    pub variables: HashMap<String, LLVMValue>,
}

/// Symbol information
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub type_: Type,
    pub kind: SymbolKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Variable,
    Function {
        params: Vec<Type>,
        return_type: Type,
    },
    Type,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()], // Start with global scope
            functions: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    pub fn with_parent(parent: Self) -> Self {
        let mut new_table = Self::new();
        new_table.scopes = parent.scopes;
        new_table.enter_scope();
        new_table
    }
    
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    
    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }
    
    pub fn define(&mut self, symbol: Symbol) -> Result<(), String> {
        let current_scope = self.scopes.last_mut().unwrap();
        
        if current_scope.contains_key(&symbol.name) {
            return Err(format!("Symbol '{}' already defined in this scope", symbol.name));
        }
        
        current_scope.insert(symbol.name.clone(), symbol);
        Ok(())
    }
    
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    pub fn define_variable(&mut self, name: String, value: LLVMValue) {
        self.variables.insert(name, value);
    }

    pub fn lookup_variable(&self, name: &str) -> Option<&LLVMValue> {
        self.variables.get(name)
    }

    pub fn define_function(&mut self, name: String, func: LLVMFunction) {
        self.functions.insert(name, func);
    }

    pub fn lookup_function(&self, name: &str) -> Option<&LLVMFunction> {
        self.functions.get(name)
    }
}

/// Semantic analyzer for type checking and other semantic validations
pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    errors: Vec<SemanticError>,
}

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub message: String,
    pub span: SourceSpan,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
        }
    }
    
    pub fn analyze(&mut self, program: &mut Program) -> Result<(), Vec<SemanticError>> {
        self.visit_program(program);
        
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }
    
    fn visit_program(&mut self, program: &mut Program) {
        for stmt in &mut program.statements {
            self.visit_stmt(stmt);
        }
    }
    
    fn visit_stmt(&mut self, stmt: &mut Stmt) {
        match &mut stmt.kind {
            StmtKind::Function { name, params, return_type, body } => {
                let param_types: Vec<Type> = params.iter().map(|p| p.type_.clone()).collect();
                let ret_type = return_type.clone().unwrap_or(Type::Unit);
                
                let symbol = Symbol {
                    name: name.clone(),
                    type_: Type::Function {
                        params: param_types.clone(),
                        return_type: Box::new(ret_type.clone()),
                    },
                    kind: SymbolKind::Function {
                        params: param_types,
                        return_type: ret_type,
                    },
                    span: stmt.span.clone(),
                };
                
                if let Err(err) = self.symbol_table.define(symbol) {
                    self.error(err, stmt.span.clone());
                }
                
                // Enter function scope
                self.symbol_table.enter_scope();
                
                // Add parameters to scope
                for param in params {
                    let param_symbol = Symbol {
                        name: param.name.clone(),
                        type_: param.type_.clone(),
                        kind: SymbolKind::Variable,
                        span: param.span.clone(),
                    };
                    
                    if let Err(err) = self.symbol_table.define(param_symbol) {
                        self.error(err, param.span.clone());
                    }
                }
                
                // Analyze function body
                for body_stmt in body {
                    self.visit_stmt(body_stmt);
                }
                
                self.symbol_table.exit_scope();
            }
            
            StmtKind::If { condition, then_branch, else_branch } => {
                let cond_type = self.visit_expr(condition);
                if cond_type != Type::Bool {
                    self.error(format!(
                        "If condition must be boolean, found {:?}",
                        cond_type
                    ), condition.span.clone());
                }
                
                self.symbol_table.enter_scope();
                for stmt in then_branch {
                    self.visit_stmt(stmt);
                }
                self.symbol_table.exit_scope();
                
                if let Some(else_stmts) = else_branch {
                    self.symbol_table.enter_scope();
                    for stmt in else_stmts {
                        self.visit_stmt(stmt);
                    }
                    self.symbol_table.exit_scope();
                }
            }
            
            StmtKind::While { condition, body } => {
                let cond_type = self.visit_expr(condition);
                if cond_type != Type::Bool {
                    self.error(format!(
                        "While condition must be boolean, found {:?}",
                        cond_type
                    ), condition.span.clone());
                }
                
                self.symbol_table.enter_scope();
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                self.symbol_table.exit_scope();
            }
            
            StmtKind::Unless { condition, body } => {
                let cond_type = self.visit_expr(condition);
                if cond_type != Type::Bool {
                    self.error(format!(
                        "Unless condition must be boolean, found {:?}",
                        cond_type
                    ), condition.span.clone());
                }
                
                self.symbol_table.enter_scope();
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                self.symbol_table.exit_scope();
            }
            
            StmtKind::Until { condition, body } => {
                let cond_type = self.visit_expr(condition);
                if cond_type != Type::Bool {
                    self.error(format!(
                        "Until condition must be boolean, found {:?}",
                        cond_type
                    ), condition.span.clone());
                }
                
                self.symbol_table.enter_scope();
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                self.symbol_table.exit_scope();
            }
            
            StmtKind::For { variable, iterable, body } => {
                let iterable_type = self.visit_expr(iterable);
                
                // Determine element type from iterable
                let element_type = match &iterable_type {
                    Type::List(elem_type) => *elem_type.clone(),
                    Type::String => Type::String, // Characters
                    _ => {
                        self.error(format!(
                            "Cannot iterate over type {:?}",
                            iterable_type
                        ), iterable.span.clone());
                        Type::Unknown
                    }
                };
                
                self.symbol_table.enter_scope();
                
                // Define loop variable
                let loop_var_symbol = Symbol {
                    name: variable.clone(),
                    type_: element_type,
                    kind: SymbolKind::Variable,
                    span: stmt.span.clone(),
                };
                
                if let Err(err) = self.symbol_table.define(loop_var_symbol) {
                    self.error(err, stmt.span.clone());
                }
                
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                self.symbol_table.exit_scope();
            }
            
            StmtKind::Iterate { iterable, body } => {
                let iterable_type = self.visit_expr(iterable);
                
                // Determine element type and verify iterability
                let element_type = match &iterable_type {
                    Type::List(elem_type) => *elem_type.clone(),
                    Type::String => Type::String, // Or Type::Char if you add one
                    _ => {
                        self.error(format!(
                            "Cannot iterate over type {:?}",
                            iterable_type
                        ), iterable.span.clone());
                        Type::Unknown
                    }
                };
                
                self.symbol_table.enter_scope();
                
                // Define $ symbol in the iterate scope
                let dollar_symbol = Symbol {
                    name: "$".to_string(),
                    type_: element_type,
                    kind: SymbolKind::Variable,
                    span: iterable.span.clone(),
                };
                
                if let Err(err) = self.symbol_table.define(dollar_symbol) {
                    self.error(err, iterable.span.clone());
                }
                
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                self.symbol_table.exit_scope();
            }
            
            StmtKind::Assignment { target, value } => {
                let value_type = self.visit_expr(value);
                
                // Check if this is a variable assignment
                if let ExprKind::Identifier(name) = &target.kind {
                    // Check if variable already exists
                    if let Some(_) = self.symbol_table.lookup(name) {
                        // Variable exists, check type compatibility
                        let target_type = self.visit_expr(target);
                        if !self.types_compatible(&value_type, &target_type) {
                            self.error(format!(
                                "Assignment type mismatch: cannot assign {:?} to {:?}",
                                value_type, target_type
                            ), stmt.span.clone());
                        }
                    } else {
                        // Variable doesn't exist, declare it with the value's type
                        let symbol = Symbol {
                            name: name.clone(),
                            type_: value_type.clone(),
                            kind: SymbolKind::Variable,
                            span: target.span.clone(),
                        };
                        
                        if let Err(err) = self.symbol_table.define(symbol) {
                            self.error(err, stmt.span.clone());
                        }
                    }
                } else {
                    // For other types of targets (field access, indexing), just type check
                    let target_type = self.visit_expr(target);
                    if !self.types_compatible(&value_type, &target_type) {
                        self.error(format!(
                            "Assignment type mismatch: cannot assign {:?} to {:?}",
                            value_type, target_type
                        ), stmt.span.clone());
                    }
                }
            }
            
            StmtKind::Return(expr) => {
                if let Some(expr) = expr {
                    self.visit_expr(expr);
                }
            }
            
            StmtKind::Expression(expr) => {
                self.visit_expr(expr);
            }
            
            StmtKind::ErrorHandler { handler, inner } => {
                // Recursively analyze the guarded statement
                self.visit_stmt(inner);
                // Analyze error handler actions
                for action in &mut handler.actions {
                    match action {
                        ErrorAction::Log(opt_expr) => {
                            if let Some(expr) = opt_expr {
                                self.visit_expr(expr);
                                let ty = expr.type_.clone();
                                // Log should accept any type, but warn if not string or convertible
                                if ty != Type::String && ty != Type::Unknown {
                                    self.error(format!("Log action expects string or convertible, found {:?}", ty), expr.span.clone());
                                }
                            }
                        }
                        ErrorAction::Return(opt_expr) => {
                            if let Some(expr) = opt_expr {
                                self.visit_expr(expr);
                                let ty = expr.type_.clone();
                                // Try to get function return type from symbol table
                                let ret_type = self.current_function_return_type();
                                if let Some(expected) = ret_type {
                                    if !self.types_compatible(&ty, &expected) {
                                        self.error(format!("Return type mismatch in error handler: expected {:?}, found {:?}", expected, ty), expr.span.clone());
                                    }
                                }
                            }
                        }
                        ErrorAction::Custom(expr) => {
                            self.visit_expr(expr);
                        }
                    }
                }
                // TODO: propagate error types if needed (Result<T, E>), for now just type check
            }
            
            _ => {
                // Handle other statement types
            }
        }
    }
    
    fn visit_expr(&mut self, expr: &mut Expr) -> Type {
        let inferred_type = match &mut expr.kind {
            ExprKind::Literal(lit) => self.literal_type(lit),
            
            ExprKind::Identifier(name) => {
                if let Some(symbol) = self.symbol_table.lookup(name) {
                    symbol.type_.clone()
                } else {
                    self.error(format!("Undefined variable: {}", name), expr.span.clone());
                    Type::Unknown
                }
            }
            
            ExprKind::Binary { op, left, right } => {
                let left_type = self.visit_expr(left);
                let right_type = self.visit_expr(right);
                self.binary_result_type(op, &left_type, &right_type, &expr.span)
            }
            
            ExprKind::Unary { op, operand } => {
                let operand_type = self.visit_expr(operand);
                self.unary_result_type(op, &operand_type, &expr.span)
            }
            
            ExprKind::Call { callee, args } => {
                let callee_type = self.visit_expr(callee);
                let arg_types: Vec<Type> = args.iter_mut().map(|arg| self.visit_expr(arg)).collect();
                
                match callee_type {
                    Type::Function { params, return_type } => {
                        if params.len() != arg_types.len() {
                            self.error(format!(
                                "Function call argument count mismatch: expected {}, found {}",
                                params.len(), arg_types.len()
                            ), expr.span.clone());
                        } else {
                            for (param_type, arg_type) in params.iter().zip(arg_types.iter()) {
                                if !self.types_compatible(arg_type, param_type) {
                                    self.error(format!(
                                        "Function call argument type mismatch: expected {:?}, found {:?}",
                                        param_type, arg_type
                                    ), expr.span.clone());
                                }
                            }
                        }
                        *return_type
                    }
                    _ => {
                        self.error(format!(
                            "Cannot call non-function type: {:?}",
                            callee_type
                        ), expr.span.clone());
                        Type::Unknown
                    }
                }
            }
            
            ExprKind::If { condition, then_branch, else_branch } => {
                let cond_type = self.visit_expr(condition);
                if cond_type != Type::Bool {
                    self.error(format!(
                        "If condition must be boolean, found {:?}",
                        cond_type
                    ), condition.span.clone());
                }
                
                let then_type = self.visit_expr(then_branch);
                
                if let Some(else_expr) = else_branch {
                    let else_type = self.visit_expr(else_expr);
                    if !self.types_compatible(&then_type, &else_type) {
                        self.error(format!(
                            "If branch type mismatch: then branch is {:?}, else branch is {:?}",
                            then_type, else_type
                        ), expr.span.clone());
                    }
                }
                
                then_type
            }
            
            ExprKind::Block(stmts) => {
                self.symbol_table.enter_scope();
                
                let mut last_type = Type::Unit;
                for stmt in stmts {
                    self.visit_stmt(stmt);
                    
                    // If the last statement is an expression, use its type
                    if let StmtKind::Expression(expr) = &stmt.kind {
                        last_type = expr.type_.clone();
                    }
                }
                
                self.symbol_table.exit_scope();
                last_type
            }
            
            ExprKind::StringInterpolation { parts } => {
                // Visit all expression parts to type check them
                for part in parts {
                    if let crate::ast::StringPart::Expression(expr) = part {
                        self.visit_expr(expr);
                    }
                }
                Type::String
            }
            
            _ => Type::Unknown,
        };
        
        // Update the expression's type
        expr.type_ = inferred_type.clone();
        inferred_type
    }
    
    fn literal_type(&self, lit: &Literal) -> Type {
        match lit {
            Literal::Integer(_) => Type::I32, // Default to i32
            Literal::Float(_) => Type::F64,   // Default to f64
            Literal::String(_) => Type::String,
            Literal::Bool(_) => Type::Bool,
            Literal::Unit => Type::Unit,
            // Coral-specific literals
            Literal::No => Type::Unit,        // `no` represents null/none
            Literal::Yes => Type::Bool,
            Literal::Empty => Type::List(Box::new(Type::Unknown)), // Empty collection
            Literal::None => Type::Unit,
            Literal::Now => Type::I64,        // Timestamp as i64
            Literal::Err => Type::Unknown,
        }
    }
    
    fn binary_result_type(&mut self, op: &BinaryOp, left: &Type, right: &Type, span: &SourceSpan) -> Type {
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if left.is_numeric() && right.is_numeric() {
                    if left.is_float() || right.is_float() {
                        Type::F64 // Promote to floating point
                    } else {
                        Type::I32 // Default integer type
                    }
                } else {
                    self.error(format!(
                        "Arithmetic operation requires numeric types, found {:?} and {:?}",
                        left, right
                    ), span.clone());
                    Type::Unknown
                }
            }
            
            BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                if self.types_compatible(left, right) {
                    Type::Bool
                } else {
                    self.error(format!(
                        "Comparison requires compatible types, found {:?} and {:?}",
                        left, right
                    ), span.clone());
                    Type::Bool
                }
            }
            
            BinaryOp::And | BinaryOp::Or => {
                if *left == Type::Bool && *right == Type::Bool {
                    Type::Bool
                } else {
                    self.error(format!(
                        "Logical operation requires boolean types, found {:?} and {:?}",
                        left, right
                    ), span.clone());
                    Type::Bool
                }
            }
            
            _ => Type::Unknown,
        }
    }
    
    fn unary_result_type(&mut self, op: &UnaryOp, operand: &Type, span: &SourceSpan) -> Type {
        match op {
            UnaryOp::Not => {
                if *operand == Type::Bool {
                    Type::Bool
                } else {
                    self.error(format!(
                        "Logical not requires boolean type, found {:?}",
                        operand
                    ), span.clone());
                    Type::Bool
                }
            }
            
            UnaryOp::Neg => {
                if operand.is_numeric() {
                    operand.clone()
                } else {
                    self.error(format!(
                        "Numeric negation requires numeric type, found {:?}",
                        operand
                    ), span.clone());
                    Type::Unknown
                }
            }
            
            UnaryOp::BitNot => {
                if operand.is_integer() {
                    operand.clone()
                } else {
                    self.error(format!(
                        "Bitwise not requires integer type, found {:?}",
                        operand
                    ), span.clone());
                    Type::Unknown
                }
            }
        }
    }
    
    fn types_compatible(&self, actual: &Type, expected: &Type) -> bool {
        match (actual, expected) {
            (Type::Unknown, _) | (_, Type::Unknown) => true,
            (a, b) if a == b => true,
            // Widening conversions for integers
            (Type::I8, Type::I16) | (Type::I8, Type::I32) | (Type::I8, Type::I64) => true,
            (Type::I16, Type::I32) | (Type::I16, Type::I64) => true,
            (Type::I32, Type::I64) => true,
            // Widening conversions for floats
            (Type::F32, Type::F64) => true,
            // Integer to float conversion
            (Type::I8, Type::F32) | (Type::I8, Type::F64) => true,
            (Type::I16, Type::F32) | (Type::I16, Type::F64) => true,
            (Type::I32, Type::F32) | (Type::I32, Type::F64) => true,
            (Type::I64, Type::F32) | (Type::I64, Type::F64) => true,
            // List compatibility (covariant)
            (Type::List(actual_inner), Type::List(expected_inner)) => {
                self.types_compatible(actual_inner, expected_inner)
            }
            // Map compatibility (covariant values, invariant keys for now)
            (Type::Map(actual_key, actual_value), Type::Map(expected_key, expected_value)) => {
                self.types_compatible(actual_key, expected_key) && self.types_compatible(actual_value, expected_value)
            }
            // Function compatibility (contravariant parameters, covariant return)
            (Type::Function { params: actual_params, return_type: actual_return },
             Type::Function { params: expected_params, return_type: expected_return }) => {
                if actual_params.len() != expected_params.len() { return false; }
                let params_compatible = actual_params.iter().zip(expected_params.iter()).all(|(a, e)| {
                    // Contravariant: expected parameter type must be compatible with actual
                    self.types_compatible(e, a)
                });
                let return_compatible = self.types_compatible(actual_return, expected_return);
                params_compatible && return_compatible
            }
            // Object subtyping (structural)
            (Type::Object { fields: actual_fields, .. }, Type::Object { fields: expected_fields, .. }) => {
                expected_fields.iter().all(|(name, expected_type)| {
                    actual_fields.get(name).map_or(false, |actual_type| {
                        self.types_compatible(actual_type, expected_type)
                    })
                })
            }
            // Result type compatibility
            (Type::Result(actual_ok, actual_err), Type::Result(expected_ok, expected_err)) => {
                self.types_compatible(actual_ok, expected_ok) && self.types_compatible(actual_err, expected_err)
            }
            _ => false,
        }
    }
    
    fn error(&mut self, message: String, span: SourceSpan) {
        self.errors.push(SemanticError { message, span });
    }
    
    /// Helper to get the current function's return type for error handler validation
    fn current_function_return_type(&self) -> Option<Type> {
        // Search scopes from innermost to outermost for a function symbol
        for scope in self.symbol_table.scopes.iter().rev() {
            for symbol in scope.values() {
                if let SymbolKind::Function { return_type, .. } = &symbol.kind {
                    return Some(return_type.clone());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_variable_analysis() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let mut program = Program {
            statements: vec![
                Stmt::new(
                    SourceSpan::default(),
                    StmtKind::Assignment {
                        target: Expr::new(
                            SourceSpan::default(),
                            ExprKind::Identifier("x".to_string())
                        ),
                        value: Expr::new(
                            SourceSpan::default(),
                            ExprKind::Literal(Literal::Integer(42))
                        ),
                    }
                )
            ],
            span: SourceSpan::default(),
        };
        
        let result = analyzer.analyze(&mut program);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_error_handler_semantics() {
        use crate::ast::*;
        let span = SourceSpan::default();
        let mut analyzer = SemanticAnalyzer::new();
        // Function with i32 return type
        let mut program = Program {
            statements: vec![
                Stmt::new(
                    span.clone(),
                    StmtKind::Function {
                        name: "f".to_string(),
                        params: vec![],
                        return_type: Some(Type::I32),
                        body: vec![
                            // Guarded statement with error handler: log and return
                            Stmt::new(
                                span.clone(),
                                StmtKind::ErrorHandler {
                                    handler: ErrorHandler {
                                        actions: vec![
                                            ErrorAction::Log(Some(Expr::new(span.clone(), ExprKind::literal(Literal::String("fail".to_string()))))),
                                            ErrorAction::Return(Some(Expr::new(span.clone(), ExprKind::literal(Literal::Integer(42))))),
                                        ],
                                        span: span.clone(),
                                    },
                                    inner: Box::new(Stmt::new(
                                        span.clone(),
                                        StmtKind::Assignment {
                                            target: Expr::new(span.clone(), ExprKind::identifier("x")),
                                            value: Expr::new(span.clone(), ExprKind::literal(Literal::Integer(1))),
                                        }
                                    )),
                                }
                            ),
                        ],
                    }
                )
            ],
            span: span.clone(),
        };
        let result = analyzer.analyze(&mut program);
        assert!(result.is_ok(), "Error handler semantic analysis should pass for valid log/return");
        // Now test type mismatch in return
        let mut program_bad = Program {
            statements: vec![
                Stmt::new(
                    span.clone(),
                    StmtKind::Function {
                        name: "g".to_string(),
                        params: vec![],
                        return_type: Some(Type::I32),
                        body: vec![
                            Stmt::new(
                                span.clone(),
                                StmtKind::ErrorHandler {
                                    handler: ErrorHandler {
                                        actions: vec![
                                            ErrorAction::Return(Some(Expr::new(span.clone(), ExprKind::literal(Literal::String("oops".to_string()))))),
                                        ],
                                        span: span.clone(),
                                    },
                                    inner: Box::new(Stmt::new(
                                        span.clone(),
                                        StmtKind::Assignment {
                                            target: Expr::new(span.clone(), ExprKind::identifier("y")),
                                            value: Expr::new(span.clone(), ExprKind::literal(Literal::Integer(2))),
                                        }
                                    )),
                                }
                            ),
                        ],
                    }
                )
            ],
            span: span.clone(),
        };
        let result_bad = analyzer.analyze(&mut program_bad);
        assert!(result_bad.is_err(), "Error handler semantic analysis should fail for return type mismatch");
    }
}
