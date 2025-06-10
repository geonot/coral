use crate::lexer::Lexer;
use crate::token::{Token, TokenType};
use crate::ast::*;
use crate::semantic::{ParseError, ErrorType, Symbol, SymbolType, Scope};

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    peek_token: Token,
    errors: Vec<ParseError>,
    scope: Scope,
    in_panic_mode: bool,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();
        
        Parser {
            lexer,
            current_token,
            peek_token,
            errors: Vec::new(),
            scope: Scope::new(),
            in_panic_mode: false,
        }
    }

    pub fn parse_program(&mut self) -> Program {
        let mut statements = Vec::new();
        
        while !self.current_token_is(&TokenType::Eof) {
            if self.current_token_is(&TokenType::Newline) {
                self.next_token();
                continue;
            }
            
            match self.parse_statement() {
                Ok(stmt) => {
                    statements.push(stmt);
                    self.in_panic_mode = false;
                }
                Err(err) => {
                    self.errors.push(err);
                    self.recover_from_error();
                }
            }
            
            // Smart token advancement - only if we're at a statement boundary
            if matches!(&self.current_token.kind, 
                       TokenType::Newline | TokenType::Eof | TokenType::Dedent) {
                continue; // Natural boundaries - no advancement needed
            } else {
                // We're in the middle of something - advance once
                self.next_token();
            }
        }
        
        Program { statements }
    }

    pub fn errors(&self) -> &Vec<ParseError> {
        &self.errors
    }

    fn recover_from_error(&mut self) {
        self.in_panic_mode = true;
        
        // Skip tokens until we find a safe synchronization point
        while !self.current_token_is(&TokenType::Eof) {
            if matches!(&self.current_token.kind,
                TokenType::Fn | TokenType::Object | TokenType::Store |
                TokenType::Use | TokenType::Newline | TokenType::Dedent) {
                break;
            }
            self.next_token();
        }
    }

    fn make_error(&self, message: String, error_type: ErrorType) -> ParseError {
        ParseError {
            message,
            line: self.current_token.line,
            col: self.current_token.col,
            length: Some(self.current_token.lexeme.len()),
            error_type,
        }
    }

    // Complete missing parser methods
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        // Skip EOF gracefully
        if self.current_token_is(&TokenType::Eof) {
            // Advance past EOF
            self.next_token();
            // Return an empty statement
            return Ok(Statement::ExpressionStmt(Expression::Empty));
        }
        match &self.current_token.kind {
            TokenType::Use => self.parse_use_statement(),
            TokenType::Fn => self.parse_function_definition(),
            TokenType::Object => self.parse_object_definition(),
            TokenType::Store => {
                if self.peek_token_is(&TokenType::Actor) {
                    self.parse_actor_definition()
                } else {
                    self.parse_store_definition()
                }
            }
            TokenType::Log => {
                self.next_token(); // consume 'log'
                let message = self.parse_expression()?;
                Ok(Statement::ExpressionStmt(Expression::LogOperation {
                    message: Box::new(message)
                }))
            }
            TokenType::Return => {
                self.next_token(); // consume 'return'
                
                // Handle "return log X" pattern
                if self.current_token_is(&TokenType::Log) {
                    self.next_token(); // consume 'log'
                    let message = self.parse_expression()?;
                    Ok(Statement::ExpressionStmt(Expression::LogOperation {
                        message: Box::new(message)
                    }))
                } else {
                    let value = if matches!(&self.current_token.kind, 
                                          TokenType::Newline | TokenType::Eof | TokenType::Dedent) {
                        Expression::Identifier("self".to_string())
                    } else {
                        self.parse_expression()?
                    };
                    Ok(Statement::ExpressionStmt(value))
                }
            }
            TokenType::While => {
                self.next_token(); // consume 'while'
                let condition = self.parse_expression()?;
                let body = self.parse_block()?;
                Ok(Statement::ExpressionStmt(Expression::WhileLoop {
                    condition: Box::new(condition),
                    body,
                }))
            }
            TokenType::Until => {
                self.next_token(); // consume 'until'
                let iterator = if let TokenType::Identifier(name) = &self.current_token.kind {
                    let var_name = name.clone();
                    self.next_token();
                    var_name
                } else {
                    "iterator".to_string()
                };
                
                // Parse 'from' clause if present
                let start_value = if self.current_token_is(&TokenType::From) {
                    self.next_token(); // consume 'from'
                    Some(Box::new(self.parse_expression()?))
                } else {
                    None
                };
                
                // Parse 'by' clause if present
                let step_value = if self.current_token_is(&TokenType::By) {
                    self.next_token(); // consume 'by'
                    Some(Box::new(self.parse_expression()?))
                } else {
                    None
                };
                
                // Parse 'is' clause for end condition
                if self.current_token_is(&TokenType::Is) {
                    self.next_token(); // consume 'is'
                }
                
                let end_condition = self.parse_expression()?;
                let body = self.parse_block()?;
                
                Ok(Statement::ExpressionStmt(Expression::UntilLoop {
                    iterator,
                    start_value,
                    step_value,
                    end_condition: Box::new(end_condition),
                    body,
                }))
            }
            TokenType::Iterate => {
                self.next_token(); // consume 'iterate'
                
                // Parse collection - expect a simple identifier or member expression
                let collection = if let TokenType::Identifier(name) = &self.current_token.kind {
                    let ident = name.clone();
                    self.next_token();
                    
                    // Handle member expressions like "system_status.active_nodes"
                    if self.current_token_is(&TokenType::Dot) {
                        let mut expr = Expression::Identifier(ident);
                        while self.current_token_is(&TokenType::Dot) {
                            self.next_token(); // consume '.'
                            if let TokenType::Identifier(member) = &self.current_token.kind {
                                let member_name = member.clone();
                                self.next_token();
                                expr = Expression::MethodCall {
                                    object: Box::new(expr),
                                    method: member_name,
                                    args: Vec::new(),
                                    named_args: Vec::new(),
                                    force_call: false,
                                    chaining: None,
                                };
                            } else {
                                return Err(self.make_error("Expected property name after '.'".to_string(), ErrorType::UnexpectedToken));
                            }
                        }
                        expr
                    } else {
                        Expression::Identifier(ident)
                    }
                } else {
                    return Err(self.make_error("Expected collection name".to_string(), ErrorType::UnexpectedToken));
                };
                
                // Parse function name - expect a simple identifier
                let function = if let TokenType::Identifier(name) = &self.current_token.kind {
                    let func_name = name.clone();
                    self.next_token();
                    Expression::Identifier(func_name)
                } else {
                    return Err(self.make_error("Expected function name".to_string(), ErrorType::UnexpectedToken));
                };
                
                // Handle the '$' parameter reference - check if it's present
                let param_ref = if matches!(&self.current_token.kind, TokenType::ParameterRef(_)) {
                    let param_name = if let TokenType::ParameterRef(name) = &self.current_token.kind {
                        name.clone()
                    } else {
                        "".to_string()
                    };
                    self.next_token(); // consume the parameter reference
                    Some(Box::new(Expression::ParameterRef(param_name)))
                } else {
                    None
                };
                
                Ok(Statement::ExpressionStmt(Expression::IterateOperation {
                    collection: Box::new(collection),
                    function: Box::new(function),
                    param_ref,
                }))
            }
            TokenType::Unless => {
                // Handle "unless condition" statements
                self.next_token(); // consume 'unless'
                let condition = self.parse_expression()?;
                let body = self.parse_block()?;
                
                Ok(Statement::ExpressionStmt(Expression::UnlessExpression {
                    condition: Box::new(condition),
                    body,
                    is_postfix: false,
                }))
            }
            TokenType::Push => {
                // Handle "push item on collection" statements
                self.next_token(); // consume 'push'
                let item = self.parse_expression()?;
                
                if !self.current_token_is(&TokenType::On) {
                    return Err(self.make_error("Expected 'on' after push item".to_string(), ErrorType::MissingToken));
                }
                self.next_token(); // consume 'on'
                
                let collection = self.parse_expression()?;
                
                Ok(Statement::ExpressionStmt(Expression::MethodCall {
                    object: Box::new(collection),
                    method: "push".to_string(),
                    args: vec![item],
                    named_args: Vec::new(),
                    force_call: false,
                    chaining: None,
                }))
            }
            TokenType::AnnotationMarker => {
                // Handle message handler syntax @method_name
                self.next_token(); // consume '@'
                if let TokenType::Identifier(handler_name) = &self.current_token.kind {
                    let msg_type = handler_name.clone();
                    self.next_token();
                    
                    let parameters = if self.current_token_is(&TokenType::With) {
                        self.next_token();
                        self.parse_parameter_list()?
                    } else {
                        Vec::new()
                    };
                    
                    // Handle single-line message handlers without blocks
                    let body = if self.current_token_is(&TokenType::Newline) && self.peek_token_is(&TokenType::Indent) {
                        self.parse_block()?
                    } else if !matches!(&self.current_token.kind, TokenType::Newline | TokenType::Eof | TokenType::Dedent) {
                        // Simple identifier sequence like "check_blocked log return"
                        let mut stmts = Vec::new();
                        while matches!(&self.current_token.kind, TokenType::Identifier(_)) && 
                              !self.current_token_is(&TokenType::Eof) {
                            if let TokenType::Identifier(name) = &self.current_token.kind {
                                stmts.push(Statement::ExpressionStmt(Expression::Identifier(name.clone())));
                                self.next_token();
                            } else {
                                break;
                            }
                        }
                        stmts
                    } else {
                        Vec::new()
                    };
                    
                    Ok(Statement::ExpressionStmt(Expression::MessageHandler {
                        message_type: msg_type,
                        parameters,
                        body,
                    }))
                } else {
                    Err(self.make_error("Expected message handler name after '@'".to_string(), ErrorType::UnexpectedToken))
                }
            }
            TokenType::Identifier(_) => {
                if self.peek_token_is(&TokenType::Is) {
                    self.parse_assignment()
                } else if self.peek_token_is(&TokenType::Across) {
                    // Handle "check_health across ..." pattern
                    let function_name = if let TokenType::Identifier(name) = &self.current_token.kind {
                        name.clone()
                    } else {
                        return Err(self.make_error("Expected function name".to_string(), ErrorType::UnexpectedToken));
                    };
                    self.next_token(); // consume function name
                    self.next_token(); // consume 'across'
                    
                    let collection = self.parse_expression()?;
                    
                    // Handle optional 'into' clause
                    let result_var = if self.current_token_is(&TokenType::Into) {
                        self.next_token(); // consume 'into'
                        if let TokenType::Identifier(var_name) = &self.current_token.kind {
                            let var = var_name.clone();
                            self.next_token();
                            Some(var)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    
                    // Handle optional 'with' clause for named parameters
                    let named_params = if self.current_token_is(&TokenType::With) {
                        self.next_token(); // consume 'with'
                        self.parse_named_argument_list()?
                    } else {
                        Vec::new()
                    };
                    
                    Ok(Statement::ExpressionStmt(Expression::AcrossOperation {
                        function_name,
                        collection: Box::new(collection),
                        result_var,
                        named_params,
                    }))
                } else {
                    let expr = self.parse_expression()?;
                    Ok(Statement::ExpressionStmt(expr))
                }
            }
            // Skip unexpected indentation gracefully
            TokenType::Indent | TokenType::Dedent => {
                self.next_token();
                self.parse_statement()
            }
            _ => {
                let expr = self.parse_expression()?;
                Ok(Statement::ExpressionStmt(expr))
            }
        }
    }

    fn parse_assignment(&mut self) -> Result<Statement, ParseError> {
        let identifier = if let TokenType::Identifier(name) = &self.current_token.kind {
            name.clone()
        } else {
            return Err(self.make_error("Expected identifier".to_string(), ErrorType::UnexpectedToken));
        };

        let line = self.current_token.line;
        let col = self.current_token.col;
        self.next_token();
        
        if !self.current_token_is(&TokenType::Is) {
            return Err(self.make_error("Expected 'is'".to_string(), ErrorType::MissingToken));
        }
        
        self.next_token();
        let value = self.parse_expression()?;
        
        // Register variable in scope
        let symbol = Symbol {
            name: identifier.clone(),
            symbol_type: SymbolType::Variable(None),
            defined_at: (line, col),
            used_at: Vec::new(),
        };
        let _ = self.scope.define(identifier.clone(), symbol);
        
        Ok(Statement::Assignment(Assignment { identifier, value }))
    }

    fn parse_function_definition(&mut self) -> Result<Statement, ParseError> {
        self.next_token(); // consume 'fn'
        
        let name = if let TokenType::Identifier(name) = &self.current_token.kind {
            name.clone()
        } else {
            return Err(self.make_error("Expected function name".to_string(), ErrorType::UnexpectedToken));
        };
        
        let line = self.current_token.line;
        let col = self.current_token.col;
        self.next_token();
        
        if !self.current_token_is(&TokenType::With) {
            return Err(self.make_error("Expected 'with'".to_string(), ErrorType::MissingToken));
        }
        
        self.next_token();
        let parameters = self.parse_parameter_list()?;
        let body = self.parse_block()?;
        
        // Register function in scope
        let param_names: Vec<String> = parameters.iter().map(|p| p.name.clone()).collect();
        let symbol = Symbol {
            name: name.clone(),
            symbol_type: SymbolType::Function(param_names, None),
            defined_at: (line, col),
            used_at: Vec::new(),
        };
        let _ = self.scope.define(name.clone(), symbol);
        
        Ok(Statement::FunctionDef(FunctionDef { name, parameters, body }))
    }

    fn parse_object_definition(&mut self) -> Result<Statement, ParseError> {
        self.next_token(); // consume 'object'
        
        let name = if let TokenType::Identifier(name) = &self.current_token.kind {
            name.clone()
        } else {
            return Err(self.make_error("Expected object name".to_string(), ErrorType::UnexpectedToken));
        };
        
        let line = self.current_token.line;
        let col = self.current_token.col;
        self.next_token();
        
        let (properties, methods) = self.parse_object_body()?;
        
        // Register object in scope
        let prop_names: Vec<String> = properties.iter().map(|p| p.name.clone()).collect();
        let symbol = Symbol {
            name: name.clone(),
            symbol_type: SymbolType::Object(prop_names),
            defined_at: (line, col),
            used_at: Vec::new(),
        };
        let _ = self.scope.define(name.clone(), symbol);
        
        Ok(Statement::ObjectDef(ObjectDef { name, properties, methods }))
    }

    fn parse_use_statement(&mut self) -> Result<Statement, ParseError> {
        self.next_token(); // consume 'use'
        
        let mut modules = Vec::new();
        
        loop {
            let module = self.parse_module_path()?;
            modules.push(module);
            
            if self.current_token_is(&TokenType::Comma) {
                self.next_token();
            } else {
                break;
            }
        }
        
        Ok(Statement::UseStatement(UseStatement { modules }))
    }

    fn parse_parameter_list(&mut self) -> Result<Vec<Parameter>, ParseError> {
        let mut parameters = Vec::new();
        
        self.skip_newlines();
        
        if matches!(&self.current_token.kind, 
                   TokenType::Newline | TokenType::Eof | TokenType::Indent) {
            return Ok(parameters);
        }
        
        loop {
            if let TokenType::Identifier(name) = &self.current_token.kind {
                let param_name = name.clone();
                self.next_token();
                
                let default_value = if self.current_token_is(&TokenType::Question) {
                    self.next_token();
                    Some(self.parse_expression()?)
                } else if !matches!(&self.current_token.kind,
                                   TokenType::Comma | TokenType::Newline | TokenType::Eof) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                
                parameters.push(Parameter {
                    name: param_name,
                    default_value,
                });
                
                if self.current_token_is(&TokenType::Comma) {
                    self.next_token();
                    self.skip_newlines();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        Ok(parameters)
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements = Vec::new();
        
        self.skip_newlines();
        
        if !self.current_token_is(&TokenType::Indent) {
            return Ok(statements);
        }
        
        self.next_token(); // consume INDENT
        
        while !self.current_token_is(&TokenType::Dedent) && !self.current_token_is(&TokenType::Eof) {
            if self.current_token_is(&TokenType::Newline) {
                self.next_token();
                continue;
            }
            
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    self.errors.push(err);
                    self.recover_from_error();
                }
            }
            
            // Don't automatically advance token here - let parse_statement handle it
            if !matches!(&self.current_token.kind, 
                        TokenType::Dedent | TokenType::Eof | TokenType::Newline) {
                self.next_token();
            }
        }
        
        // Handle EOF gracefully - treat it as an implicit DEDENT
        if self.current_token_is(&TokenType::Eof) {
            // Block is implicitly closed by EOF - this is valid
            return Ok(statements);
        }
        
        if self.current_token_is(&TokenType::Dedent) {
            self.next_token(); // consume DEDENT
        }
        
        Ok(statements)
    }

    fn parse_store_definition(&mut self) -> Result<Statement, ParseError> {
        self.next_token(); // consume 'store'
        
        let name = if let TokenType::Identifier(name) = &self.current_token.kind {
            name.clone()
        } else {
            return Err(self.make_error("Expected store name".to_string(), ErrorType::UnexpectedToken));
        };
        
        self.next_token();
        let (properties, methods, make_method, as_methods) = self.parse_store_body()?;
        
        Ok(Statement::StoreDef(StoreDef { 
            name, 
            properties, 
            methods, 
            make_method, 
            as_methods 
        }))
    }

    fn parse_actor_definition(&mut self) -> Result<Statement, ParseError> {
        self.next_token(); // consume 'store'
        self.next_token(); // consume 'actor'
        
        let name = if let TokenType::Identifier(name) = &self.current_token.kind {
            name.clone()
        } else {
            return Err(self.make_error("Expected actor name".to_string(), ErrorType::UnexpectedToken));
        };
        
        self.next_token();
        let (properties, methods, join_tables, message_handlers, make_method) = self.parse_actor_body()?;
        
        Ok(Statement::ActorDef(ActorDef { 
            name, 
            properties, 
            methods, 
            join_tables, 
            message_handlers, 
            make_method 
        }))
    }

    fn parse_store_body(&mut self) -> Result<(Vec<PropertyDef>, Vec<MethodDef>, Option<MethodDef>, Vec<AsMethodDef>), ParseError> {
        let mut properties = Vec::new();
        let mut methods = Vec::new();
        let mut make_method = None;
        let mut as_methods = Vec::new();
        
        self.skip_newlines();
        
        if !self.current_token_is(&TokenType::Indent) {
            return Ok((properties, methods, make_method, as_methods));
        }
        
        self.next_token(); // consume INDENT
        
        while !self.current_token_is(&TokenType::Dedent) && !self.current_token_is(&TokenType::Eof) {
            if self.current_token_is(&TokenType::Newline) {
                self.next_token();
                continue;
            }
            
            match &self.current_token.kind {
                TokenType::Make => {
                    self.next_token();
                    let body = self.parse_block()?;
                    make_method = Some(MethodDef {
                        name: "make".to_string(),
                        parameters: Vec::new(),
                        body,
                    });
                }
                TokenType::As => {
                    self.next_token();
                    if let TokenType::Identifier(conversion_type) = &self.current_token.kind {
                        let conv_type = conversion_type.clone();
                        self.next_token();
                        let body = self.parse_block()?;
                        as_methods.push(AsMethodDef {
                            conversion_type: conv_type,
                            body,
                        });
                    }
                }
                TokenType::Identifier(name) => {
                    let item_name = name.clone();
                    self.next_token();
                    
                    if self.current_token_is(&TokenType::Question) {
                        self.next_token();
                        let default_value = Some(self.parse_expression()?);
                        properties.push(PropertyDef {
                            name: item_name,
                            default_value,
                            doc_comment: None,
                        });
                    } else if self.is_method_definition() || item_name.starts_with("as_") {
                        // Handle regular methods and as_* conversion methods
                        let parameters = if self.current_token_is(&TokenType::With) {
                            self.next_token();
                            self.parse_parameter_list()?
                        } else {
                            Vec::new()
                        };
                        
                        let body = self.parse_block()?;
                        
                        if item_name.starts_with("as_") {
                            // This is an as_* conversion method
                            let conversion_type = item_name.strip_prefix("as_").unwrap_or(&item_name).to_string();
                            as_methods.push(AsMethodDef {
                                conversion_type,
                                body,
                            });
                        } else {
                            methods.push(MethodDef {
                                name: item_name,
                                parameters,
                                body,
                            });
                        }
                    } else {
                        properties.push(PropertyDef {
                            name: item_name,
                            default_value: None,
                            doc_comment: None,
                        });
                    }
                }
                _ => break,
            }
        }
        
        // Handle EOF gracefully - store body is complete
        if self.current_token_is(&TokenType::Eof) {
            return Ok((properties, methods, make_method, as_methods));
        }
        
        if self.current_token_is(&TokenType::Dedent) {
            self.next_token();
        }
        
        Ok((properties, methods, make_method, as_methods))
    }

    fn parse_actor_body(&mut self) -> Result<(Vec<PropertyDef>, Vec<MethodDef>, Vec<String>, Vec<MessageHandler>, Option<MethodDef>), ParseError> {
        let mut properties = Vec::new();
        let mut methods = Vec::new();
        let mut join_tables = Vec::new();
        let mut message_handlers = Vec::new();
        let mut make_method = None;
        
        self.skip_newlines();
        
        if !self.current_token_is(&TokenType::Indent) {
            return Ok((properties, methods, join_tables, message_handlers, make_method));
        }
        
        self.next_token(); // consume INDENT
        
        while !self.current_token_is(&TokenType::Dedent) && !self.current_token_is(&TokenType::Eof) {
            if self.current_token_is(&TokenType::Newline) {
                self.next_token();
                continue;
            }
            
            match &self.current_token.kind {
                TokenType::Amp | TokenType::AmpRef => {
                    self.next_token();
                    if let TokenType::Identifier(table_name) = &self.current_token.kind {
                        join_tables.push(table_name.clone());
                        self.next_token();
                    }
                }
                TokenType::AnnotationMarker => {
                    self.next_token();
                    if let TokenType::Identifier(handler_name) = &self.current_token.kind {
                        let msg_type = handler_name.clone();
                        self.next_token();
                        
                        let parameters = if self.current_token_is(&TokenType::With) {
                            self.next_token();
                            self.parse_parameter_list()?
                        } else {
                            Vec::new()
                        };
                        
                        let body = if self.current_token_is(&TokenType::Newline) && self.peek_token_is(&TokenType::Indent) {
                            self.parse_block()?
                        } else if !matches!(&self.current_token.kind, TokenType::Newline | TokenType::Eof | TokenType::Dedent) {
                            // Simple identifier sequence like "check_blocked log return"
                            let mut stmts = Vec::new();
                            while matches!(&self.current_token.kind, TokenType::Identifier(_)) {
                                if let TokenType::Identifier(name) = &self.current_token.kind {
                                    stmts.push(Statement::ExpressionStmt(Expression::Identifier(name.clone())));
                                    self.next_token();
                                    // Break immediately on EOF to prevent infinite loop
                                    if self.current_token_is(&TokenType::Eof) {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }
                            stmts
                        } else {
                            Vec::new()
                        };
                        
                        message_handlers.push(MessageHandler {
                            message_type: msg_type,
                            parameters,
                            body,
                        });
                        
                        // Exit immediately if we hit EOF after processing message handler
                        if self.current_token_is(&TokenType::Eof) {
                            break;
                        }
                    }
                }
                TokenType::Make => {
                    self.next_token();
                    let body = self.parse_block()?;
                    make_method = Some(MethodDef {
                        name: "make".to_string(),
                        parameters: Vec::new(),
                        body,
                    });
                }
                TokenType::Identifier(name) => {
                    let item_name = name.clone();
                    self.next_token();
                    
                    if self.current_token_is(&TokenType::Question) {
                        self.next_token();
                        let default_value = Some(self.parse_expression()?);
                        properties.push(PropertyDef {
                            name: item_name,
                            default_value,
                            doc_comment: None,
                        });
                    } else if self.is_method_definition() {
                        let parameters = if self.current_token_is(&TokenType::With) {
                            self.next_token(); // consume 'with'
                            self.parse_parameter_list()?
                        } else {
                            Vec::new()
                        };
                        
                        let body = self.parse_block()?;
                        methods.push(MethodDef {
                            name: item_name,
                            parameters,
                            body,
                        });
                    } else {
                        properties.push(PropertyDef {
                            name: item_name,
                            default_value: None,
                            doc_comment: None,
                        });
                    }
                }
                _ => break,
            }
        }
        
        // Handle EOF gracefully - actor body is complete even without explicit DEDENT
        if self.current_token_is(&TokenType::Eof) {
            return Ok((properties, methods, join_tables, message_handlers, make_method));
        }
        
        if self.current_token_is(&TokenType::Dedent) {
            self.next_token();
        }
        
        Ok((properties, methods, join_tables, message_handlers, make_method))
    }

    // Improved object body parsing with better semantic analysis
    fn parse_object_body(&mut self) -> Result<(Vec<PropertyDef>, Vec<MethodDef>), ParseError> {
        let mut properties = Vec::new();
        let mut methods = Vec::new();
        let mut prop_names = std::collections::HashSet::new();
        
        self.skip_newlines();
        
        if !self.current_token_is(&TokenType::Indent) {
            return Ok((properties, methods));
        }
        
        self.next_token(); // consume INDENT
        let mut object_scope = Scope::with_parent(std::mem::replace(&mut self.scope, Scope::new()));
        
        while !self.current_token_is(&TokenType::Dedent) && !self.current_token_is(&TokenType::Eof) {
            if self.current_token_is(&TokenType::Newline) {
                self.next_token();
                continue;
            }
            
            if let TokenType::Identifier(name) = &self.current_token.kind {
                let item_name = name.clone();
                let item_line = self.current_token.line;
                let item_col = self.current_token.col;
                self.next_token();
                
                // Check for duplicate names
                if prop_names.contains(&item_name) {
                    self.errors.push(ParseError {
                        message: format!("Duplicate property/method name: {}", item_name),
                        line: item_line,
                        col: item_col,
                        length: Some(item_name.len()),
                        error_type: ErrorType::SemanticError,
                    });
                }
                prop_names.insert(item_name.clone());
                
                if self.current_token_is(&TokenType::Question) {
                    // Property with default value
                    self.next_token(); // consume '?'
                    let default_value = Some(self.parse_expression()?);
                    
                    // Register property in scope
                    let symbol = Symbol {
                        name: item_name.clone(),
                        symbol_type: SymbolType::Variable(None),
                        defined_at: (item_line, item_col),
                        used_at: Vec::new(),
                    };
                    let _ = object_scope.define(item_name.clone(), symbol);
                    
                    properties.push(PropertyDef {
                        name: item_name,
                        default_value,
                        doc_comment: None,
                    });
                } else if self.is_method_definition() {
                    // Method definition
                    let parameters = if self.current_token_is(&TokenType::With) {
                        self.next_token(); // consume 'with'
                        self.parse_parameter_list()?
                    } else {
                        Vec::new()
                    };
                    
                    let body = self.parse_block()?;
                    
                    // Register method in scope
                    let param_names: Vec<String> = parameters.iter().map(|p| p.name.clone()).collect();
                    let symbol = Symbol {
                        name: item_name.clone(),
                        symbol_type: SymbolType::Function(param_names, None),
                        defined_at: (item_line, item_col),
                        used_at: Vec::new(),
                    };
                    let _ = object_scope.define(item_name.clone(), symbol);
                    
                    methods.push(MethodDef {
                        name: item_name,
                        parameters,
                        body,
                    });
                } else {
                    // Simple property
                    let symbol = Symbol {
                        name: item_name.clone(),
                        symbol_type: SymbolType::Variable(None),
                        defined_at: (item_line, item_col),
                        used_at: Vec::new(),
                    };
                    let _ = object_scope.define(item_name.clone(), symbol);
                    
                    properties.push(PropertyDef {
                        name: item_name,
                        default_value: None,
                        doc_comment: None,
                    });
                }
                
                // Skip newlines after property/method definition
                self.skip_newlines();
            } else {
                return Err(self.make_error(
                    "Expected property or method name".to_string(),
                    ErrorType::UnexpectedToken
                ));
            }
        }
        
        if self.current_token_is(&TokenType::Dedent) {
            // Don't consume DEDENT here - let parse_program handle it
        }
        
        self.scope = *object_scope.parent.unwrap_or(Box::new(Scope::new()));
        Ok((properties, methods))
    }

    fn is_method_definition(&self) -> bool {
        // Only consider it a method if we explicitly see 'with' for parameters
        // or if there's an immediate indent (function body)
        self.current_token_is(&TokenType::With) ||
        (self.current_token_is(&TokenType::Newline) && 
         self.peek_token_is(&TokenType::Indent))
    }

    fn skip_newlines(&mut self) {
        while self.current_token_is(&TokenType::Newline) {
            self.next_token();
        }
    }

    // Performance-optimized expression parsing
    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_ternary_expression()
    }

    fn parse_ternary_expression(&mut self) -> Result<Expression, ParseError> {
        let expr = self.parse_logical_or_expression()?;
        
        if self.current_token_is(&TokenType::Question) {
            self.next_token(); // consume '?'
            let true_expr = Box::new(self.parse_logical_or_expression()?);
            
            // Check if this is a full ternary or just a default value
            if self.current_token_is(&TokenType::Bang) {
                self.next_token(); // consume '!'
                let false_expr = Box::new(self.parse_ternary_expression()?);
                
                return Ok(Expression::Ternary {
                    condition: Box::new(expr),
                    true_expr,
                    false_expr,
                });
            } else {
                // This is just a default value assignment, not a full ternary
                // Return the true_expr as the result
                return Ok(*true_expr);
            }
        }
        
        Ok(expr)
    }

    // Fast binary expression parsing with precedence climbing
    fn parse_logical_or_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_binary_expression(0)
    }

    fn parse_binary_expression(&mut self, min_precedence: u8) -> Result<Expression, ParseError> {
        let mut left = self.parse_unary_expression()?;
        
        while let Some((op, precedence)) = self.get_binary_operator_precedence() {
            if precedence < min_precedence {
                break;
            }
            
            self.next_token(); // consume operator
            let right = self.parse_binary_expression(precedence + 1)?;
            
            left = Expression::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn get_binary_operator_precedence(&self) -> Option<(BinaryOp, u8)> {
        match &self.current_token.kind {
            TokenType::Or | TokenType::PipePipe => Some((BinaryOp::Or, 1)),
            TokenType::And | TokenType::AmpAmp => Some((BinaryOp::And, 2)),
            TokenType::EqEq => Some((BinaryOp::Equal, 3)),
            TokenType::BangEq => Some((BinaryOp::NotEqual, 3)),
            TokenType::Equals => Some((BinaryOp::Equals, 3)),
            TokenType::Gt => Some((BinaryOp::GreaterThan, 4)),
            TokenType::Lt => Some((BinaryOp::LessThan, 4)),
            TokenType::Gte | TokenType::GtEq => Some((BinaryOp::GreaterThanOrEqual, 4)),
            TokenType::Lte | TokenType::LtEq => Some((BinaryOp::LessThanOrEqual, 4)),
            TokenType::Plus => Some((BinaryOp::Add, 5)),
            TokenType::Minus => Some((BinaryOp::Subtract, 5)),
            TokenType::Star => Some((BinaryOp::Multiply, 6)),
            TokenType::Slash => Some((BinaryOp::Divide, 6)),
            TokenType::Percent => Some((BinaryOp::Modulo, 6)),
            TokenType::DoubleStar => Some((BinaryOp::Power, 8)),
            // Don't treat 'at' as a binary operator - it should be handled in postfix
            _ => None,
        }
    }

    fn parse_unary_expression(&mut self) -> Result<Expression, ParseError> {
        match &self.current_token.kind {
            TokenType::Bang => {
                self.next_token();
                let operand = self.parse_unary_expression()?;
                Ok(Expression::Unary {
                    operator: UnaryOp::Not,
                    operand: Box::new(operand),
                })
            }
            TokenType::Minus => {
                self.next_token();
                let operand = self.parse_unary_expression()?;
                Ok(Expression::Unary {
                    operator: UnaryOp::Minus,
                    operand: Box::new(operand),
                })
            }
            _ => self.parse_postfix_expression(),
        }
    }

    fn parse_postfix_expression(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_primary_expression()?;
        
        loop {
            match &self.current_token.kind {
                TokenType::Dot => {
                    self.next_token();
                    expr = self.parse_method_call(expr)?;
                }
                TokenType::LBracket => {
                    self.next_token();
                    let index = self.parse_expression()?;
                    
                    if !self.current_token_is(&TokenType::RBracket) {
                        return Err(self.make_error("Expected ']'".to_string(), ErrorType::MissingToken));
                    }
                    self.next_token();
                    
                    expr = Expression::ArrayAccess {
                        array: Box::new(expr),
                        index: Box::new(index),
                        use_at_keyword: false,
                    };
                }
                TokenType::At => {
                    self.next_token(); // consume 'at'
                    let index = self.parse_primary_expression()?;
                    
                    expr = Expression::ArrayAccess {
                        array: Box::new(expr),
                        index: Box::new(index),
                        use_at_keyword: true,
                    };
                }
                TokenType::Err => {
                    self.next_token(); // consume 'err'
                    
                    // Parse error handling chain: err log return, err {}, etc.
                    let error_action = if self.current_token_is(&TokenType::Log) {
                        self.next_token(); // consume 'log'
                        if self.current_token_is(&TokenType::Return) {
                            self.next_token(); // consume 'return'
                            ErrorAction::LogReturn
                        } else {
                            ErrorAction::ReturnLogError
                        }
                    } else if self.current_token_is(&TokenType::Return) {
                        self.next_token(); // consume 'return'
                        ErrorAction::ReturnLogError
                    } else if self.current_token_is(&TokenType::LBrace) {
                        let default_val = self.parse_primary_expression()?;
                        ErrorAction::DefaultValue(default_val)
                    } else {
                        let default_val = self.parse_primary_expression()?;
                        ErrorAction::DefaultValue(default_val)
                    };
                    
                    expr = Expression::ErrorHandling {
                        expression: Box::new(expr),
                        error_action: Box::new(error_action),
                    };
                }
                TokenType::As => {
                    self.next_token(); // consume 'as'
                    if let TokenType::Identifier(target_type) = &self.current_token.kind {
                        let type_name = target_type.clone();
                        self.next_token();
                        expr = Expression::AsConversion {
                            expression: Box::new(expr),
                            target_type: type_name,
                        };
                    } else {
                        return Err(self.make_error("Expected type name after 'as'".to_string(), ErrorType::UnexpectedToken));
                    }
                }
                _ => break,
            }
        }
        
        Ok(expr)
    }

    fn parse_primary_expression(&mut self) -> Result<Expression, ParseError> {
        match &self.current_token.kind {
            TokenType::Integer(value) => {
                let int_val = value.parse::<i64>().map_err(|_| {
                    self.make_error("Invalid integer".to_string(), ErrorType::InvalidSyntax)
                })?;
                self.next_token();
                Ok(Expression::Integer(int_val))
            }
            TokenType::Float(value) => {
                let float_val = value.parse::<f64>().map_err(|_| {
                    self.make_error("Invalid float".to_string(), ErrorType::InvalidSyntax)
                })?;
                self.next_token();
                Ok(Expression::Float(float_val))
            }
            TokenType::StringLiteral(value) => {
                let string_val = value.clone();
                self.next_token();
                Ok(Expression::StringLiteral(string_val))
            }
            TokenType::InterpolatedString(value) => {
                let string_val = value.clone();
                self.next_token();
                Ok(Expression::InterpolatedString(string_val))
            }
            TokenType::Boolean(value) => {
                let bool_val = *value;
                self.next_token();
                Ok(Expression::Boolean(bool_val))
            }
            TokenType::Empty => {
                self.next_token();
                Ok(Expression::Empty)
            }
            TokenType::Now => {
                self.next_token();
                Ok(Expression::Now)
            }
            TokenType::ParameterRef(name) => {
                let param_name = name.clone();
                self.next_token();
                Ok(Expression::ParameterRef(param_name))
            }
            TokenType::LBracket => self.parse_array_literal(),
            TokenType::LBrace => self.parse_object_literal(),
            TokenType::LParen => {
                self.next_token();
                let expr = self.parse_expression()?;
                
                if !self.current_token_is(&TokenType::RParen) {
                    return Err(self.make_error("Expected ')'".to_string(), ErrorType::MissingToken));
                }
                self.next_token();
                
                Ok(expr)
            }
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.next_token();
                
                if self.current_token_is(&TokenType::Bang) {
                    self.next_token();
                    let (args, named_args) = self.parse_argument_list()?;
                    Ok(Expression::Instantiation {
                        type_name: name,
                        args,
                        named_args,
                        force_success: true,
                    })
                } else if self.current_token_is(&TokenType::With) {
                    self.next_token();
                    let named_args = self.parse_named_argument_list()?;
                    Ok(Expression::Instantiation {
                        type_name: name,
                        args: Vec::new(),
                        named_args,
                        force_success: false,
                    })
                } else if self.current_token_is(&TokenType::LParen) {
                    let (args, named_args) = self.parse_argument_list()?;
                    Ok(Expression::FunctionCall { name, args, named_args })
                } else if matches!(&self.current_token.kind,
                    TokenType::Integer(_) | TokenType::Float(_) | TokenType::StringLiteral(_) |
                    TokenType::InterpolatedString(_) | TokenType::Boolean(_) | TokenType::Identifier(_) |
                    TokenType::LBracket | TokenType::ParameterRef(_)) {
                    // Check if this is a method call like "collection add item"
                    if let TokenType::Identifier(potential_method) = &self.current_token.kind {
                        let method_name = potential_method.clone();
                        if matches!(method_name.as_str(), "add" | "remove" | "contains" | "length" | "size") {
                            self.next_token(); // consume method name
                            let (args, named_args) = self.parse_argument_list()?;
                            return Ok(Expression::MethodCall {
                                object: Box::new(Expression::Identifier(name)),
                                method: method_name,
                                args,
                                named_args,
                                force_call: false,
                                chaining: None,
                            });
                        }
                    }
                    // Space-separated function call
                    let (args, named_args) = self.parse_argument_list()?;
                    Ok(Expression::FunctionCall { name, args, named_args })
                } else {
                    // Simple identifier
                    Ok(Expression::Identifier(name))
                }
            }
            _ => Err(self.make_error(
                format!("Unexpected token: {:?}", self.current_token.kind),
                ErrorType::UnexpectedToken
            )),
        }
    }

    fn parse_array_literal(&mut self) -> Result<Expression, ParseError> {
        self.next_token(); // consume '['
        let mut elements = Vec::new();
        
        while !self.current_token_is(&TokenType::RBracket) && !self.current_token_is(&TokenType::Eof) {
            elements.push(self.parse_expression()?);
            
            if self.current_token_is(&TokenType::Comma) {
                self.next_token();
            } else {
                break;
            }
        }
        
        if !self.current_token_is(&TokenType::RBracket) {
            return Err(self.make_error("Expected ']'".to_string(), ErrorType::MissingToken));
        }
        self.next_token();
        
        Ok(Expression::Array(elements))
    }

    fn parse_method_call(&mut self, object: Expression) -> Result<Expression, ParseError> {
        if let TokenType::Identifier(method_name) = &self.current_token.kind {
            let method = method_name.clone();
            self.next_token();
            
            let force_call = if self.current_token_is(&TokenType::Bang) {
                self.next_token();
                true
            } else {
                false
            };
            
            let (args, named_args) = if matches!(&self.current_token.kind,
                                               TokenType::Identifier(_) | TokenType::Integer(_) | 
                                               TokenType::Float(_) | TokenType::StringLiteral(_) |
                                               TokenType::LParen) {
                self.parse_argument_list()?
            } else {
                (Vec::new(), Vec::new())
            };
            
            // Check for method chaining with 'then' or 'and'
            let chaining = if self.current_token_is(&TokenType::Then) || self.current_token_is(&TokenType::And) {
                Some(Box::new(self.parse_method_chain()?))
            } else {
                None
            };
            
            Ok(Expression::MethodCall {
                object: Box::new(object),
                method,
                args,
                named_args,
                force_call,
                chaining,
            })
        } else {
            Err(self.make_error("Expected method name".to_string(), ErrorType::UnexpectedToken))
        }
    }

    fn parse_method_chain(&mut self) -> Result<MethodChain, ParseError> {
        let connector = if self.current_token_is(&TokenType::Then) {
            self.next_token();
            ChainConnector::Then
        } else if self.current_token_is(&TokenType::And) {
            self.next_token();
            ChainConnector::And
        } else {
            return Err(self.make_error("Expected 'then' or 'and'".to_string(), ErrorType::UnexpectedToken));
        };

        // Expect a dot after 'then' or 'and'
        if !self.current_token_is(&TokenType::Dot) {
            return Err(self.make_error("Expected '.' after method chain connector".to_string(), ErrorType::MissingToken));
        }
        self.next_token(); // consume '.'

        // Parse method name
        let method = if let TokenType::Identifier(method_name) = &self.current_token.kind {
            let name = method_name.clone();
            self.next_token();
            name
        } else {
            return Err(self.make_error("Expected method name".to_string(), ErrorType::UnexpectedToken));
        };

        // Check for force call (!)
        let force_call = if self.current_token_is(&TokenType::Bang) {
            self.next_token();
            true
        } else {
            false
        };

        // Parse arguments if present
        let args = if matches!(&self.current_token.kind,
                             TokenType::Identifier(_) | TokenType::Integer(_) |
                             TokenType::Float(_) | TokenType::StringLiteral(_) |
                             TokenType::LParen) {
            let (args, _) = self.parse_argument_list()?;
            args
        } else {
            Vec::new()
        };

        // Check for further chaining
        let next = if self.current_token_is(&TokenType::Then) || self.current_token_is(&TokenType::And) {
            Some(Box::new(self.parse_method_chain()?))
        } else {
            None
        };

        Ok(MethodChain {
            connector,
            method,
            args,
            force_call,
            next,
        })
    }

    fn parse_argument_list(&mut self) -> Result<(Vec<Expression>, Vec<(String, Expression)>), ParseError> {
        let mut args = Vec::new();
        let mut named_args = Vec::new();
        
        if self.current_token_is(&TokenType::LParen) {
            self.next_token();
            
            while !self.current_token_is(&TokenType::RParen) && !self.current_token_is(&TokenType::Eof) {
                // Skip newlines in argument lists
                if self.current_token_is(&TokenType::Newline) {
                    self.next_token();
                    continue;
                }
                
                args.push(self.parse_expression()?);
                
                if self.current_token_is(&TokenType::Comma) {
                    self.next_token();
                    self.skip_newlines(); // Allow newlines after commas
                } else {
                    break;
                }
            }
            
            if self.current_token_is(&TokenType::RParen) {
                self.next_token();
            }
        } else {
            // Parse space-separated arguments, handling newlines and indented continuations
            while matches!(&self.current_token.kind,
                          TokenType::Integer(_) | TokenType::Float(_) | TokenType::StringLiteral(_) |
                          TokenType::InterpolatedString(_) | TokenType::Boolean(_) | TokenType::Identifier(_) |
                          TokenType::LBracket | TokenType::ParameterRef(_) | TokenType::LParen) &&
                  !matches!(&self.current_token.kind, TokenType::Eof) {
                
                // Don't consume if it looks like the start of the next statement
                if matches!(&self.current_token.kind, TokenType::Identifier(_)) &&
                   matches!(&self.peek_token.kind, TokenType::Is) {
                    break;
                }
                
                // Handle named arguments with identifiers followed by values
                if let TokenType::Identifier(name) = &self.current_token.kind {
                    let arg_name = name.clone();
                    self.next_token();
                    
                    // Check if this is a named argument (identifier followed by value)
                    if !matches!(&self.current_token.kind, 
                                TokenType::Newline | TokenType::Eof | TokenType::Comma |
                                TokenType::Is | TokenType::Equals) {
                        let value = self.parse_primary_expression()?;
                        named_args.push((arg_name, value));
                        
                        if self.current_token_is(&TokenType::Comma) {
                            self.next_token();
                            self.skip_newlines();
                        }
                        continue;
                    } else {
                        // Put the identifier back as a regular argument
                        args.push(Expression::Identifier(arg_name));
                    }
                } else {
                    args.push(self.parse_primary_expression()?);
                }
                
                // Handle continuation on next line with proper indentation
                if self.current_token_is(&TokenType::Newline) {
                    self.next_token();
                    // If the next token is an indent, we're continuing the argument list
                    if self.current_token_is(&TokenType::Indent) {
                        self.next_token(); // consume indent
                        continue;
                    } else {
                        break; // End of arguments
                    }
                }
            }
        }
        
        Ok((args, named_args))
    }

    fn parse_named_argument_list(&mut self) -> Result<Vec<(String, Expression)>, ParseError> {
        let mut named_args = Vec::new();
        
        while !matches!(&self.current_token.kind, TokenType::Newline | TokenType::Eof) {
            if let TokenType::Identifier(name) = &self.current_token.kind {
                let arg_name = name.clone();
                self.next_token();
                
                let value = self.parse_expression()?;
                named_args.push((arg_name, value));
                
                if self.current_token_is(&TokenType::Comma) {
                    self.next_token();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        Ok(named_args)
    }

    fn parse_module_path(&mut self) -> Result<String, ParseError> {
        let mut path_parts = Vec::new();
        
        if let TokenType::Identifier(name) = &self.current_token.kind {
            path_parts.push(name.clone());
            self.next_token();
            
            while self.current_token_is(&TokenType::Dot) {
                self.next_token();
                if let TokenType::Identifier(name) = &self.current_token.kind {
                    path_parts.push(name.clone());
                    self.next_token();
                } else {
                    return Err(self.make_error("Expected identifier".to_string(), ErrorType::UnexpectedToken));
                }
            }
        } else {
            return Err(self.make_error("Expected module name".to_string(), ErrorType::UnexpectedToken));
        }
        
        Ok(path_parts.join("."))
    }

    // Essential helper methods
    fn next_token(&mut self) {
        self.current_token = std::mem::replace(&mut self.peek_token, self.lexer.next_token());
    }

    fn current_token_is(&self, token_type: &TokenType) -> bool {
        std::mem::discriminant(&self.current_token.kind) == std::mem::discriminant(token_type)
    }

    fn peek_token_is(&self, token_type: &TokenType) -> bool {
        std::mem::discriminant(&self.peek_token.kind) == std::mem::discriminant(token_type)
    }

    fn parse_object_literal(&mut self) -> Result<Expression, ParseError> {
        self.next_token(); // consume '{'
        let mut pairs = Vec::new();
        
        while !self.current_token_is(&TokenType::RBrace) && !self.current_token_is(&TokenType::Eof) {
            if let TokenType::Identifier(key) = &self.current_token.kind {
                let key_name = key.clone();
                self.next_token();
                
                if self.current_token_is(&TokenType::Is) {
                    self.next_token();
                    let value = self.parse_expression()?;
                    pairs.push((key_name, value));
                } else {
                    // Key without explicit value, assume identifier with same name
                    pairs.push((key_name.clone(), Expression::Identifier(key_name)));
                }
                
                if self.current_token_is(&TokenType::Comma) {
                    self.next_token();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        if !self.current_token_is(&TokenType::RBrace) {
            return Err(self.make_error("Expected '}'".to_string(), ErrorType::MissingToken));
        }
        self.next_token();
        
        Ok(Expression::ObjectLiteral(pairs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_assignment() {
        let input = "x is 42";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        
        let program = parser.parse_program();
        assert_eq!(parser.errors().len(), 0);
        assert_eq!(program.statements.len(), 1);
        
        if let Statement::Assignment(assignment) = &program.statements[0] {
            assert_eq!(assignment.identifier, "x");
            if let Expression::Integer(val) = assignment.value {
                assert_eq!(val, 42);
            } else {
                panic!("Expected integer value");
            }
        } else {
            panic!("Expected assignment statement");
        }
    }

    #[test]
    fn test_function_definition() {
        let input = "fn greet with name\n    'hello {name}'";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        
        let program = parser.parse_program();
        assert_eq!(parser.errors().len(), 0);
        assert_eq!(program.statements.len(), 1);
        
        if let Statement::FunctionDef(func_def) = &program.statements[0] {
            assert_eq!(func_def.name, "greet");
            assert_eq!(func_def.parameters.len(), 1);
            assert_eq!(func_def.parameters[0].name, "name");
        } else {
            panic!("Expected function definition");
        }
    }

    #[test]
    fn test_binary_expression() {
        let input = "result is x + y * 2";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        
        let program = parser.parse_program();
        
        // Should parse without errors
        assert_eq!(parser.errors().len(), 0, "Parsing should succeed without errors");
        assert_eq!(program.statements.len(), 1);
        
        if let Statement::Assignment(assignment) = &program.statements[0] {
            assert_eq!(assignment.identifier, "result");
            
            // The expression should be: x + (y * 2) due to precedence
            if let Expression::Binary { left, operator, right } = &assignment.value {
                assert_eq!(operator, &BinaryOp::Add);
                
                if let Expression::Identifier(name) = left.as_ref() {
                    assert_eq!(name, "x");
                } else {
                    panic!("Expected identifier 'x' on left side");
                }
                
                if let Expression::Binary { left: y_expr, operator: mult_op, right: two_expr } = right.as_ref() {
                    assert_eq!(mult_op, &BinaryOp::Multiply);
                    
                    if let Expression::Identifier(y_name) = y_expr.as_ref() {
                        assert_eq!(y_name, "y");
                    } else {
                        panic!("Expected identifier 'y'");
                    }
                    
                    if let Expression::Integer(val) = two_expr.as_ref() {
                        assert_eq!(*val, 2);
                    } else {
                        panic!("Expected integer 2");
                    }
                } else {
                    panic!("Expected multiplication on right side");
                }
            } else {
                panic!("Expected binary expression");
            }
        } else {
            panic!("Expected assignment statement");
        }
    }

    #[test]
    fn test_object_definition() {
        let input = "object Person\n    name\n    age ? 0";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        
        let program = parser.parse_program();
        assert_eq!(parser.errors().len(), 0);
        assert_eq!(program.statements.len(), 1);
        
        if let Statement::ObjectDef(obj_def) = &program.statements[0] {
            assert_eq!(obj_def.name, "Person");
            assert_eq!(obj_def.properties.len(), 2);
            assert_eq!(obj_def.properties[0].name, "name");
            assert_eq!(obj_def.properties[1].name, "age");
            assert!(obj_def.properties[1].default_value.is_some());
        } else {
            panic!("Expected object definition");
        }
    }

    #[test]
    fn test_complex_expressions() {
        let input = "result is (x + y) * z ? 42 ! 0";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        
        let program = parser.parse_program();
        assert_eq!(parser.errors().len(), 0);
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_function_call_with_args() {
        let input = "result is add 1 2";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        
        let program = parser.parse_program();
        assert_eq!(parser.errors().len(), 0);
        assert_eq!(program.statements.len(), 1);
        
        if let Statement::Assignment(assignment) = &program.statements[0] {
            if let Expression::FunctionCall { name, args, .. } = &assignment.value {
                assert_eq!(name, "add");
                assert_eq!(args.len(), 2);
            } else {
                panic!("Expected function call");
            }
        } else {
            panic!("Expected assignment statement");
        }
    }

    #[test]
    fn test_array_operations() {
        let input = "item is list at 0";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        
        let program = parser.parse_program();
        assert_eq!(parser.errors().len(), 0);
        assert_eq!(program.statements.len(), 1);
        
        if let Statement::Assignment(assignment) = &program.statements[0] {
            if let Expression::ArrayAccess { array, index, use_at_keyword } = &assignment.value {
                assert!(use_at_keyword);
                if let Expression::Identifier(name) = array.as_ref() {
                    assert_eq!(name, "list");
                }
                if let Expression::Integer(val) = index.as_ref() {
                    assert_eq!(*val, 0);
                }
            } else {
                panic!("Expected array access");
            }
        } else {
            panic!("Expected assignment statement");
        }
    }

    #[test]
    fn test_error_recovery() {
        let input = "x is\ny is 42";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        
        let program = parser.parse_program();
        assert!(parser.errors().len() > 0); // Should have parsing error
        assert_eq!(program.statements.len(), 1); // Should recover and parse second statement
    }
}