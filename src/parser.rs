use crate::ast::{
    Program, Stmt, StmtKind, Expr, ExprKind, Type, 
    BinaryOp, UnaryOp, Literal, Parameter, Field, MessageHandler, ObjectMethod, Argument,
    SourceSpan
};
use crate::lexer::{Token, TokenType};
use std::collections::HashMap;

/// Parser errors with source location information
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken {
        expected: String,
        found: Token,
    },
    UnexpectedEof,
    InvalidSyntax {
        message: String,
        span: SourceSpan,
    },
    DuplicateDefinition {
        name: String,
        span: SourceSpan,
    },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken { expected, found } => {
                write!(f, "Expected {}, found '{}' at line {}, column {}", 
                       expected, found.lexeme, found.line, found.column)
            }
            ParseError::UnexpectedEof => write!(f, "Unexpected end of file"),
            ParseError::InvalidSyntax { message, span } => {
                write!(f, "Syntax error: {} at {}:{}", message, span.start_line, span.start_col)
            }
            ParseError::DuplicateDefinition { name, span } => {
                write!(f, "Duplicate definition of '{}' at {}:{}", name, span.start_line, span.start_col)
            }
        }
    }
}

impl std::error::Error for ParseError {}

impl ParseError {
    pub fn span(&self) -> Option<&SourceSpan> {
        match self {
            ParseError::InvalidSyntax { span, .. } => Some(span),
            ParseError::DuplicateDefinition { span, .. } => Some(span),
            _ => None,
        }
    }
}

type ParseResult<T> = Result<T, ParseError>;

/// Recursive descent parser for Coral
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    file_name: std::sync::Arc<str>, // Shared to avoid cloning
}

impl Parser {
    pub fn new(tokens: Vec<Token>, file_name: String) -> Self {
        Self {
            tokens,
            current: 0,
            file_name: file_name.into(), // Convert to Arc<str>
        }
    }
    
    pub fn parse(&mut self) -> ParseResult<Program> {
        // Pre-allocate based on token count heuristic
        let estimated_statements = (self.tokens.len() / 8).max(4);
        let mut statements = Vec::with_capacity(estimated_statements);
        let start_span = self.current_span();
        
        while !self.is_at_end() {
            self.skip_newlines();
            // Skip any unexpected indentation at the top level
            while self.check(TokenType::Indent) || self.check(TokenType::Dedent) {
                self.advance();
            }
            if !self.is_at_end() {
                statements.push(self.parse_statement()?);
            }
        }
        
        let end_span = if statements.is_empty() {
            start_span.clone()
        } else {
            statements.last().unwrap().span.clone()
        };
        
        Ok(Program {
            statements,
            span: SourceSpan::new(
                self.file_name.clone(),
                start_span.start_line,
                start_span.start_col,
                end_span.end_line,
                end_span.end_col,
            ),
        })
    }
    
    /// Parse and resolve types for a program
    pub fn parse_and_resolve(&mut self) -> ParseResult<Program> {
        self.parse()
    }
    
    // Statement parsing
    """    fn parse_statement(&mut self) -> ParseResult<Stmt> {
        self.skip_newlines();
        match self.peek().token_type {
            TokenType::Fn => self.parse_function_statement(),
            TokenType::Object => self.parse_object_statement(),
            TokenType::Store => self.parse_store_statement(),
            TokenType::Actor => self.parse_actor_statement(),
            TokenType::If => self.parse_if_statement(),
            TokenType::Unless => self.parse_unless_statement(),
            TokenType::While => self.parse_while_statement(),
            TokenType::Until => self.parse_until_statement(),
            TokenType::Iterate => self.parse_iterate_statement(),
            TokenType::Return => self.parse_return_statement(),
            TokenType::Break => self.parse_break_statement(),
            TokenType::Continue => self.parse_continue_statement(),
            TokenType::Import => self.parse_import_statement(),
            _ => {
                let expr = self.parse_expression()?;

                // Postfix unless
                if self.match_token(TokenType::Unless) {
                    let condition = self.parse_expression()?;
                    let body_stmt = Stmt::new(expr.span.clone(), StmtKind::Expression(expr));
                    let span = self.span_between(&body_stmt.span, &condition.span);
                    return Ok(Stmt::new(span, StmtKind::Unless {
                        condition,
                        body: vec![body_stmt],
                    }));
                }

                // Assignment
                if self.check(TokenType::Is) {
                    self.advance(); // consume 'is'
                    let value = self.parse_expression()?;
                    let span = self.span_between(&expr.span, &value.span);
                    return Ok(Stmt::new(span, StmtKind::Assignment { target: expr, value }));
                }

                // Regular expression statement
                let span = expr.span.clone();
                let stmt = Stmt::new(span, StmtKind::Expression(expr));
                self.skip_newlines();
                return Ok(stmt);
            }
        }
    }""
    
    fn parse_function_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.advance(); // consume 'fn'
        let name_token = self.consume(TokenType::Identifier, "Expected function name")?;
        self.consume(TokenType::LeftParen, "Expected '(' after function name")?;
        let params = self.parse_parameter_list()?;
        self.consume(TokenType::RightParen, "Expected ')' after parameters")?;
        let return_type = if self.match_token(TokenType::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        self.skip_newlines(); // Skip any newlines before the body block

        // The parse_block_statements function will handle consuming the Indent and Dedent
        let body = self.parse_block_statements()?;

        let span = self.span_from_token(&start);
        Ok(Stmt::new(span, StmtKind::Function {
            name: name_token.lexeme,
            params,
            return_type,
            body,
        }))
    }
    
    fn parse_object_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.advance(); // consume 'object'
        let name_token = self.consume(TokenType::Identifier, "Expected object name")?;
        self.consume(TokenType::Newline, "Expected newline after object name")?;
        
        let (fields, methods) = self.parse_object_body()?;
        
        let span = self.span_from_token(&start);
        Ok(Stmt::new(span, StmtKind::Object { 
            name: name_token.lexeme,
            fields, 
            methods 
        }))
    }
    
    fn parse_store_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.advance(); // consume 'store'
        
        let name_token = self.consume(TokenType::Identifier, "Expected store name")?;
        let name = name_token.lexeme.clone();
        
        self.skip_newlines();
        
        let (fields, methods) = self.parse_object_body()?;
        
        let span = self.span_from_token(&start);
        Ok(Stmt::new(span, StmtKind::Store {
            name,
            fields,
            methods,
        }))
    }
    
    fn parse_actor_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.advance(); // consume 'actor'

        let name_token = self.consume(TokenType::Identifier, "Expected actor name")?;
        let name = name_token.lexeme.clone();

        self.skip_newlines();

        let (fields, _methods, handlers) = self.parse_actor_body()?;

        let span = self.span_from_token(&start);
        Ok(Stmt::new(span, StmtKind::Actor { name, fields, handlers }))
    }

    fn parse_actor_body(&mut self) -> ParseResult<(Vec<Field>, Vec<ObjectMethod>, Vec<MessageHandler>)> {
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut handlers = Vec::new();

        if !self.match_token(TokenType::Indent) {
            return Err(ParseError::UnexpectedToken {
                expected: "indented block".to_string(),
                found: self.peek().clone(),
            });
        }

        while !self.check(TokenType::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(TokenType::Dedent) || self.is_at_end() {
                break;
            }

            if self.check(TokenType::At) {
                // Message handler
                handlers.push(self.parse_message_handler()?);
            } else {
                // Field or method - reuse logic from parse_object_member_or_method
                let (mut parsed_fields, mut parsed_methods) = self.parse_object_member_or_method()?;
                fields.append(&mut parsed_fields);
                methods.append(&mut parsed_methods);
            }
            self.skip_newlines();
        }

        if self.check(TokenType::Dedent) {
            self.advance();
        }

        Ok((fields, methods, handlers))
    }

    // This function will parse a single field or a single method
    fn parse_object_member_or_method(&mut self) -> ParseResult<(Vec<Field>, Vec<ObjectMethod>)> {
        let mut fields = Vec::new();
        let mut methods = Vec::new();

        let name_token = self.consume(TokenType::Identifier, "Expected field or method name")?;
        let name = name_token.lexeme.clone();
        let start_span_for_member = self.token_to_span(&name_token);

        if self.check(TokenType::Colon) {
            self.advance(); // consume ':'
            self.skip_newlines();
            if self.check(TokenType::Indent) {
                // This is a method: name: (starts indented block)
                let params = Vec::new(); // TODO: Handle method parameters
                let return_type = None;
                let body = self.parse_block_statements()?;
                let span = self.span_between(&start_span_for_member, &body.last().map(|s| &s.span).unwrap_or(&start_span_for_member));
                methods.push(ObjectMethod { name, params, return_type, body, span });
            } else if self.check(TokenType::Newline) {
                self.advance(); // consume Newline
                let span = self.span_between(&start_span_for_member, &self.token_to_span(&self.previous()));
                fields.push(Field { name, type_: Type::Unknown, default_value: None, span });
            } else {
                // This is a field: name: type
                let type_ = self.parse_type()?;
                let default_value = if self.match_token(TokenType::Question) {
                    Some(self.parse_expression()?)
                } else if self.match_token(TokenType::Equal) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                let span = self.span_between(&start_span_for_member, &self.token_to_span(&self.previous()));
                fields.push(Field { name, type_, default_value, span });
            }
        } else if self.check(TokenType::Question) {
            self.advance(); // consume '?'
            let default_value = Some(self.parse_expression()?);
            let type_ = Type::Unknown;

            let span = self.span_between(&start_span_for_member, &self.token_to_span(&self.previous()));
            fields.push(Field { name, type_, default_value, span });
        } else if self.check(TokenType::LeftParen) {
            self.advance(); // consume '('
            let params = self.parse_parameter_list()?;
            self.consume(TokenType::RightParen, "Expected ')' after parameters")?;

            let return_type = if self.match_token(TokenType::Arrow) {
                Some(self.parse_type()?)
            } else {
                None
            };

            self.consume(TokenType::Colon, "Expected ':' before method body")?;
            self.skip_newlines();
            let body = self.parse_block_statements()?;

            let span = self.span_between(&start_span_for_member, &body.last().map(|s| &s.span).unwrap_or(&start_span_for_member));
            methods.push(ObjectMethod { name, params, return_type, body, span });
        } else {
            let type_ = Type::Unknown;
            let default_value = None;

            let span = self.span_between(&start_span_for_member, &self.token_to_span(&self.previous()));
            fields.push(Field { name, type_, default_value, span });
        }
        Ok((fields, methods))
    }

    fn parse_message_handler(&mut self) -> ParseResult<MessageHandler> {
        let start_token = self.consume(TokenType::At, "Expected '@' for message handler")?; // consume '@'
        let message_type = self.parse_type()?;

        self.consume(TokenType::Arrow, "Expected '=>' after message type")?;
        self.consume(TokenType::Colon, "Expected ':' after '=>'")?;
        self.skip_newlines();
        let body = self.parse_block_statements()?;

        let span = self.token_to_span(&start_token);
        Ok(MessageHandler { message_type, body, span })
    }
    
    fn parse_if_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.advance(); // consume 'if'
        
        let condition = self.parse_expression()?;
        self.skip_newlines();
        let then_branch = self.parse_block_statements()?;
        
        let else_branch = if self.match_token(TokenType::Else) {
            self.skip_newlines();
            if self.check(TokenType::If) {
                // else if
                Some(vec![self.parse_if_statement()?])
            } else {
                let stmts = self.parse_block_statements()?;
                Some(stmts)
            }
        } else {
            None
        };
        
        let span = self.span_from_token(&start);
        Ok(Stmt::new(span, StmtKind::If {
            condition,
            then_branch,
            else_branch,
        }))
    }
    
    fn parse_unless_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.advance(); // consume 'unless'
        let condition = self.parse_expression()?;
        self.skip_newlines();
        let body = self.parse_block_statements()?;
        let span = self.span_from_token(&start);
        Ok(Stmt::new(span, StmtKind::Unless { condition, body }))
    }

    fn parse_while_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.advance(); // consume 'while'
        let condition = self.parse_expression()?;
        self.skip_newlines();
        let body = self.parse_block_statements()?;
        let span = self.span_from_token(&start);
        Ok(Stmt::new(span, StmtKind::While { condition, body }))
    }

    fn parse_until_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.advance(); // consume 'until'
        let condition = self.parse_expression()?;
        self.skip_newlines();
        let body = self.parse_block_statements()?;
        let span = self.span_from_token(&start);
        Ok(Stmt::new(span, StmtKind::Until { condition, body }))
    }

    fn parse_iterate_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.advance(); // consume 'iterate'
        let iterable = self.parse_expression()?;
        self.skip_newlines();
        let body = self.parse_block_statements()?;
        let span = self.span_from_token(&start);
        Ok(Stmt::new(
            span,
            StmtKind::Iterate {
                iterable,
                body,
            },
        ))
    }
    
    fn parse_return_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.advance(); // consume 'return'
        
        let value = if self.check(TokenType::Newline) || self.check(TokenType::Eof) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        
        let span = self.span_from_token(&start);
        Ok(Stmt::new(span, StmtKind::Return(value)))
    }
    
    fn parse_break_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.advance(); // consume 'break'
        
        let span = self.span_from_token(&start);
        Ok(Stmt::new(span, StmtKind::Break))
    }
    
    fn parse_continue_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.advance(); // consume 'continue'
        
        let span = self.span_from_token(&start);
        Ok(Stmt::new(span, StmtKind::Continue))
    }
    
    fn parse_import_statement(&mut self) -> ParseResult<Stmt> {
        let start = self.advance(); // consume 'import'
        
        let module_token = self.consume(TokenType::String, "Expected module name")?;
        let module = module_token.lexeme.clone();
        
        let items = if self.match_token(TokenType::LeftBrace) {
            let mut items = Vec::new();
            
            if !self.check(TokenType::RightBrace) {
                loop {
                    let item = self.consume(TokenType::Identifier, "Expected import item")?;
                    items.push(item.lexeme.clone());
                    
                    if !self.match_token(TokenType::Comma) {
                        break;
                    }
                }
            }
            
            self.consume(TokenType::RightBrace, "Expected '}' after import items")?;
            Some(items)
        } else {
            None
        };
        
        let span = self.span_from_token(&start);
        Ok(Stmt::new(span, StmtKind::Import { module, items }))
    }

    

    
    
    // Expression parsing with precedence climbing
    fn parse_expression(&mut self) -> ParseResult<Expr> {
        self.parse_ternary()
    }
    
    fn parse_ternary(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_logical_or()?;
        
        if self.match_token(TokenType::Question) {
            let then_expr = self.parse_ternary()?; // Allow nested ternary (right-associative)
            self.consume(TokenType::Bang, "Expected '!' after ternary then branch")?;
            let else_expr = self.parse_ternary()?; // Allow nested ternary (right-associative)
            
            let span = self.span_between(&expr.span, &else_expr.span);
            expr = Expr::new(span, ExprKind::If {
                condition: Box::new(expr),
                then_branch: Box::new(then_expr),
                else_branch: Some(Box::new(else_expr)),
            });
        }
        
        Ok(expr)
    }
    
    fn parse_logical_or(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_logical_and()?;
        
        while self.match_token(TokenType::Or) || self.match_token(TokenType::LogicalOr) {
            let right = self.parse_logical_and()?;
            let span = self.span_between(&expr.span, &right.span);
            expr = Expr::new(span, ExprKind::binary(BinaryOp::Or, expr, right));
        }
        
        Ok(expr)
    }
    
    fn parse_logical_and(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_equality()?;
        
        while self.match_token(TokenType::And) || self.match_token(TokenType::LogicalAnd) {
            let right = self.parse_equality()?;
            let span = self.span_between(&expr.span, &right.span);
            expr = Expr::new(span, ExprKind::binary(BinaryOp::And, expr, right));
        }
        
        Ok(expr)
    }
    
    fn parse_equality(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_comparison()?;
        
        loop {
            let op = if self.match_token(TokenType::Equals) || self.match_token(TokenType::EqualEqual) {
                Some(BinaryOp::Eq)
            } else if self.match_token(TokenType::BangEqual) {
                Some(BinaryOp::Ne)
            } else {
                None
            };

            if let Some(op) = op {
                let right = self.parse_comparison()?;
                let span = self.span_between(&expr.span, &right.span);
                expr = Expr::new(span, ExprKind::binary(op, expr, right));
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    fn parse_comparison(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_bitwise_or()?;
        
        while let Some(op) = self.match_comparison_op() {
            let right = self.parse_bitwise_or()?;
            let span = self.span_between(&expr.span, &right.span);
            expr = Expr::new(span, ExprKind::binary(op, expr, right));
        }
        
        Ok(expr)
    }
    
    fn parse_bitwise_or(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_bitwise_xor()?;
        
        while self.match_token(TokenType::Pipe) {
            let right = self.parse_bitwise_xor()?;
            let span = self.span_between(&expr.span, &right.span);
            expr = Expr::new(span, ExprKind::binary(BinaryOp::BitOr, expr, right));
        }
        
        Ok(expr)
    }
    
    fn parse_bitwise_xor(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_bitwise_and()?;
        
        while self.match_token(TokenType::Caret) {
            let right = self.parse_bitwise_and()?;
            let span = self.span_between(&expr.span, &right.span);
            expr = Expr::new(span, ExprKind::binary(BinaryOp::BitXor, expr, right));
        }
        
        Ok(expr)
    }
    
    fn parse_bitwise_and(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_shift()?;
        
        while self.match_token(TokenType::Ampersand) {
            let right = self.parse_shift()?;
            let span = self.span_between(&expr.span, &right.span);
            expr = Expr::new(span, ExprKind::binary(BinaryOp::BitAnd, expr, right));
        }
        
        Ok(expr)
    }
    
    fn parse_shift(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_term()?;
        
        while let Some(op) = self.match_shift_op() {
            let right = self.parse_term()?;
            let span = self.span_between(&expr.span, &right.span);
            expr = Expr::new(span, ExprKind::binary(op, expr, right));
        }
        
        Ok(expr)
    }
    
    fn parse_term(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_factor()?;
        
        while let Some(op) = self.match_term_op() {
            let right = self.parse_factor()?;
            let span = self.span_between(&expr.span, &right.span);
            expr = Expr::new(span, ExprKind::binary(op, expr, right));
        }
        
        Ok(expr)
    }
    
    fn parse_factor(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_unary()?;
        
        while let Some(op) = self.match_factor_op() {
            let right = self.parse_unary()?;
            let span = self.span_between(&expr.span, &right.span);
            expr = Expr::new(span, ExprKind::binary(op, expr, right));
        }
        
        Ok(expr)
    }
    
    fn parse_unary(&mut self) -> ParseResult<Expr> {
        if let Some(op) = self.match_unary_op() {
            let operand = self.parse_unary()?;
            let span = self.span_from_current();
            return Ok(Expr::new(span, ExprKind::Unary {
                op,
                operand: Box::new(operand),
            }));
        }
        
        self.parse_postfix()
    }
    
    fn parse_postfix(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_primary()?;
        
        loop {
            if self.match_token(TokenType::LeftParen) {
                // Function call
                let args = self.parse_argument_list()?;
                self.consume(TokenType::RightParen, "Expected ')' after arguments")?;
                
                let span = self.span_from_current();
                expr = Expr::new(span, ExprKind::call(expr, args));
            } else if self.match_token(TokenType::LeftBracket) {
                // Index access
                let index = self.parse_expression()?;
                self.consume(TokenType::RightBracket, "Expected ']' after index")?;
                
                let span = self.span_between(&expr.span, &index.span);
                expr = Expr::new(span, ExprKind::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                });
            } else if self.match_token(TokenType::Dot) {
                // Field access or list append
                let field_token = self.consume(TokenType::Identifier, "Expected field name or 'put' for list append")?;
                let field_name = field_token.lexeme.clone();

                if field_name == "put" {
                    // List append: list.put item
                    let element = self.parse_expression()?;
                    let span = self.span_between(&expr.span, &element.span);
                    expr = Expr::new(span, ExprKind::ListAppend {
                        list: Box::new(expr),
                        element: Box::new(element),
                    });
                } else if field_name == "across" {
                    // Across expression: callee.across(iterable)
                    self.consume(TokenType::LeftParen, "Expected '(' after 'across'")?;
                    let iterable = self.parse_expression()?;
                    self.consume(TokenType::RightParen, "Expected ')' after iterable")?;
                    
                    let into = if self.match_token(TokenType::Dot) {
                        self.consume(TokenType::Identifier, "Expected 'into' after '.'")?;
                        self.consume(TokenType::LeftParen, "Expected '(' after 'into'")?;
                        let into_token = self.consume(TokenType::Identifier, "Expected identifier in \"into\"")?;
                        self.consume(TokenType::RightParen, "Expected ')' after 'into' identifier")?;
                        Some(into_token.lexeme)
                    } else {
                        None
                    };

                    let span = self.span_between(&expr.span, &self.token_to_span(&self.previous()));
                    expr = Expr::new(span, ExprKind::Across {
                        callee: Box::new(expr),
                        iterable: Box::new(iterable),
                        into,
                    });
                } else {
                    // Regular field access
                    let span = self.span_from_token(&field_token);
                    expr = Expr::new(span, ExprKind::FieldAccess {
                        object: Box::new(expr),
                        field: field_name,
                    });
                }
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    fn parse_primary(&mut self) -> ParseResult<Expr> {   
        let token = self.peek().clone();
        let span = self.token_to_span(&token);
        
        match &token.token_type {
            TokenType::Integer => {
                self.advance();
                let lexeme_str = token.lexeme.as_str();
                let value = if lexeme_str.starts_with("0x") || lexeme_str.starts_with("0X") {
                    i64::from_str_radix(&lexeme_str[2..], 16)
                } else if lexeme_str.starts_with('b') {
                    i64::from_str_radix(&lexeme_str[1..], 2)
                } else {
                    lexeme_str.parse::<i64>()
                }
                .map_err(|e| ParseError::InvalidSyntax {
                    message: format!("Invalid integer literal '{}': {}", lexeme_str, e),
                    span: span.clone(),
                })?;
                Ok(Expr::new(span, ExprKind::literal(Literal::Integer(value))))
            }
            TokenType::Float => {
                self.advance();
                let value = token.lexeme.parse::<f64>()
                    .map_err(|e| ParseError::InvalidSyntax {
                        message: format!("Invalid float literal '{}': {}", token.lexeme, e),
                        span: span.clone(),
                    })?;
                Ok(Expr::new(span, ExprKind::literal(Literal::Float(value))))
            }
            TokenType::String => {
                self.advance();
                Ok(Expr::new(span, ExprKind::literal(Literal::String(token.lexeme.clone()))))
            }
            TokenType::InterpolatedString => {
                self.advance();
                let parts = self.parse_string_interpolation_from_content(&token.lexeme)?;
                Ok(Expr::new(span, ExprKind::StringInterpolation { parts }))
            }
            TokenType::True => {
                self.advance();
                Ok(Expr::new(span, ExprKind::literal(Literal::Bool(true))))
            }
            TokenType::False => {
                self.advance();
                Ok(Expr::new(span, ExprKind::literal(Literal::Bool(false))))
            }
            // Coral-specific literals
            TokenType::No => {
                self.advance();
                Ok(Expr::new(span, ExprKind::literal(Literal::No)))
            }
            TokenType::Yes => {
                // In Coral, 'yes' is equivalent to 'true'
                self.advance();
                Ok(Expr::new(span, ExprKind::literal(Literal::Bool(true))))
            }
            TokenType::Empty => {
                self.advance();
                Ok(Expr::new(span, ExprKind::literal(Literal::Empty)))
            }
            TokenType::Now => {
                self.advance();
                Ok(Expr::new(span, ExprKind::literal(Literal::Now)))
            }
            TokenType::Identifier => {
                self.advance();
                Ok(Expr::new(span, ExprKind::identifier(token.lexeme.clone())))
            }
            TokenType::Dollar => {
                // $ refers to the current iteration item in Coral
                self.advance();
                Ok(Expr::new(span, ExprKind::identifier("$".to_string())))
            }
            TokenType::LeftParen => {
                self.advance();
                
                // Check if this is a parenthesized expression, list literal, or map literal
                if self.check(TokenType::RightParen) {
                    // Empty list
                    self.advance(); // consume ')'
                    return Ok(Expr::new(span, ExprKind::ListLiteral(Vec::new())));
                }
                
                // Parse first expression
                let first_expr = self.parse_expression()?;
                
                if self.match_token(TokenType::Colon) {
                    // This is a map literal
                    let mut pairs = Vec::new();
                    let first_value = self.parse_expression()?;
                    pairs.push((first_expr, first_value));
                    
                    while self.match_token(TokenType::Comma) {
                        let key = self.parse_expression()?;
                        self.consume(TokenType::Colon, "Expected ':' after map key")?;
                        let value = self.parse_expression()?;
                        pairs.push((key, value));
                    }
                    
                    self.consume(TokenType::RightParen, "Expected ')' after map pairs")?;
                    Ok(Expr::new(span, ExprKind::MapLiteral(pairs)))
                } else if self.match_token(TokenType::Comma) {
                    // This is a list literal - parse remaining elements
                    let mut elements = vec![first_expr];
                    
                    if !self.check(TokenType::RightParen) {
                        loop {
                            elements.push(self.parse_expression()?);
                            if !self.match_token(TokenType::Comma) {
                                break;
                            }
                        }
                    }
                    
                    self.consume(TokenType::RightParen, "Expected ')' after list elements")?;
                    Ok(Expr::new(span, ExprKind::ListLiteral(elements)))
                } else {
                    // This is a parenthesized expression
                    self.consume(TokenType::RightParen, "Expected ')' after expression")?;
                    Ok(first_expr)
                }
            }
            TokenType::LeftBracket => {
                self.advance();
                let elements = if self.check(TokenType::RightBracket) {
                    Vec::new()
                } else {
                    self.parse_expression_list()?
                };
                self.consume(TokenType::RightBracket, "Expected ']' after list elements")?;
                Ok(Expr::new(span, ExprKind::ListLiteral(elements)))
            }
            TokenType::LeftBrace => {
                // Only parse map literals, no braced statement blocks
                self.parse_map_literal()
            }
            TokenType::If => self.parse_if_expression(),
            TokenType::Fn => self.parse_lambda_expression(),
            _ => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: token.clone(),
            }),
        }
    }
    
    fn parse_if_expression(&mut self) -> ParseResult<Expr> {
        let start = self.advance(); // consume 'if'
        
        let condition = self.parse_expression()?;
        self.consume(TokenType::Colon, "Expected ':' after if condition")?;
        self.skip_newlines();
        let then_stmts = self.parse_block_statements()?;
        
        let else_branch = if self.match_token(TokenType::Else) {
            self.consume(TokenType::Colon, "Expected ':' after else")?;
            self.skip_newlines();
            if self.check(TokenType::If) {
                Some(Box::new(self.parse_if_expression()?))
            } else {
                let else_stmts = self.parse_block_statements()?;
                
                let span = self.span_from_current();
                Some(Box::new(Expr::new(span, ExprKind::Block(else_stmts))))
            }
        } else {
            None
        };
        
        let span = self.span_from_token(&start);
        Ok(Expr::new(span.clone(), ExprKind::If {
            condition: Box::new(condition),
            then_branch: Box::new(Expr::new(span.clone(), ExprKind::Block(then_stmts))),
            else_branch,
        }))
    }
    
    fn parse_lambda_expression(&mut self) -> ParseResult<Expr> {
        let start = self.advance(); // consume 'fn'
        
        self.consume(TokenType::LeftParen, "Expected '(' after 'fn'")?;
        let params = self.parse_parameter_list()?;
        self.consume(TokenType::RightParen, "Expected ')' after parameters")?;
        
        if self.match_token(TokenType::Arrow) {
            let body = self.parse_expression()?;
            let span = self.span_from_token(&start);
            Ok(Expr::new(span, ExprKind::Lambda {
                params,
                body: Box::new(body),
            }))
        } else {
            self.skip_newlines();
            let stmts = self.parse_block_statements()?;
            
            let span = self.span_from_token(&start);
            Ok(Expr::new(span.clone(), ExprKind::Lambda {
                params,
                body: Box::new(Expr::new(span.clone(), ExprKind::Block(stmts))),
            }))
        }
    }
    
    fn parse_map_literal(&mut self) -> ParseResult<Expr> {
        let start = self.advance(); // consume '{'
        let mut pairs = Vec::new();
        
        if !self.check(TokenType::RightBrace) {
            loop {
                let key = self.parse_expression()?;
                self.consume(TokenType::Colon, "Expected ':' after map key")?;
                let value = self.parse_expression()?;
                pairs.push((key, value));
                
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        
        self.consume(TokenType::RightBrace, "Expected '}' after map pairs")?;
        
        let span = self.span_from_token(&start);
        Ok(Expr::new(span, ExprKind::MapLiteral(pairs)))
    }
    
    // Helper methods for parsing components
    fn parse_parameter_list(&mut self) -> ParseResult<Vec<Parameter>> {
        let mut params = Vec::with_capacity(4); // Most functions have 0-4 parameters
        
        if !self.check(TokenType::RightParen) {
            loop {
                let name_token = self.consume(TokenType::Identifier, "Expected parameter name")?;
                let name = name_token.lexeme.clone();
                
                // Updated parsing logic for type and default value
                let type_ = if self.match_token(TokenType::Colon) {
                    self.parse_type()?
                } else {
                    Type::Unknown
                };

                let default_value = if self.match_token(TokenType::Question) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                
                let span = self.token_to_span(&name_token);
                params.push(Parameter { 
                    name, 
                    type_, 
                    default_value,
                    span 
                });
                
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        
        Ok(params)
    }

    
    
    fn parse_argument_list(&mut self) -> ParseResult<Vec<Argument>> {
        let mut args = Vec::new();
        if self.check(TokenType::RightParen) {
            return Ok(args);
        }
        loop {
            let name = if self.check(TokenType::Identifier)
                && self
                    .tokens
                    .get(self.current + 1)
                    .map_or(false, |t| t.token_type == TokenType::Colon)
            {
                let name_token = self.advance();
                self.advance(); // consume ':'
                Some(name_token.lexeme)
            } else {
                None
            };
            let value = self.parse_expression()?;
            let span = value.span.clone();
            args.push(Argument { name, value, span });
            if !self.match_token(TokenType::Comma) {
                break;
            }
        }
        Ok(args)
    }
    
    fn parse_expression_list(&mut self) -> ParseResult<Vec<Expr>> {
        let mut exprs = Vec::with_capacity(2); // Most expression lists are small
        
        loop {
            exprs.push(self.parse_expression()?);
            
            if !self.match_token(TokenType::Comma) {
                break;
            }
        }
        
        Ok(exprs)
    }

    
    fn parse_object_body(&mut self) -> ParseResult<(Vec<Field>, Vec<ObjectMethod>)> {
        let mut fields = Vec::new();
        let mut methods = Vec::new();

        if !self.match_token(TokenType::Indent) {
            return Err(ParseError::UnexpectedToken {
                expected: "indented block".to_string(),
                found: self.peek().clone(),
            });
        }

        while !self.check(TokenType::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(TokenType::Dedent) {
                break;
            }

            if self.check(TokenType::Fn) {
                let func_stmt = self.parse_function_statement()?;
                if let StmtKind::Function { name, params, return_type, body } = func_stmt.kind {
                    methods.push(ObjectMethod {
                        name,
                        params,
                        return_type,
                        body,
                        span: func_stmt.span,
                    });
                }
                continue;
            }

            let name_token = self.consume(TokenType::Identifier, "Expected field or method name")?;
            let name = name_token.lexeme.clone();

            if self.check(TokenType::Colon) {
                self.advance(); // consume ':'
                let type_ = self.parse_type()?;
                let default_value = if self.match_token(TokenType::Question) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                let span = self.token_to_span(&name_token);
                fields.push(Field { name, type_, default_value, span });
            } else if self.check(TokenType::Question) {
                self.advance(); // consume '?'
                let default_value = Some(self.parse_expression()?);
                let type_ = Type::Unknown;
                let span = self.token_to_span(&name_token);
                fields.push(Field { name, type_, default_value, span });
            } else if self.check(TokenType::LeftParen) {
                self.advance(); // consume '('
                let params = self.parse_parameter_list()?;
                self.consume(TokenType::RightParen, "Expected ')' after parameters")?;
                let return_type = if self.match_token(TokenType::Arrow) {
                    Some(self.parse_type()?)
                } else {
                    None
                };
                self.consume(TokenType::Colon, "Expected ':' before method body")?;
                self.skip_newlines();
                let body = self.parse_block_statements()?;
                let span = self.token_to_span(&name_token);
                methods.push(ObjectMethod { name, params, return_type, body, span });
            } else {
                let type_ = Type::Unknown;
                let default_value = None;
                let span = self.token_to_span(&name_token);
                fields.push(Field { name, type_, default_value, span });
            }

            self.skip_newlines();
        }

        if !self.match_token(TokenType::Dedent) {
            return Err(ParseError::UnexpectedToken {
                expected: "dedent to close block".to_string(),
                found: self.peek().clone(),
            });
        }

        Ok((fields, methods))
    }
    
    fn _parse_message_handlers(&mut self) -> ParseResult<Vec<MessageHandler>> {
        let mut handlers = Vec::new();
        
        // Expect an Indent token to start the handlers block
        if !self.match_token(TokenType::Indent) {
            return Err(ParseError::UnexpectedToken {
                expected: "indented block".to_string(),
                found: self.peek().clone(),
            });
        }
        
        while !self.check(TokenType::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(TokenType::Dedent) || self.is_at_end() {
                break;
            }
            
            let start_token = self.peek().clone();
            let message_type = self.parse_type()?;
            
            self.consume(TokenType::Arrow, "Expected '=>' after message type")?;
            self.consume(TokenType::Colon, "Expected ':' after '=>'")?;
            self.skip_newlines();
            let body = self.parse_block_statements()?;
            
            let span = self.token_to_span(&start_token);
            handlers.push(MessageHandler { message_type, body, span });
        }
        
        // Consume the closing Dedent token
        if self.check(TokenType::Dedent) {
            self.advance();
        }
        
        Ok(handlers)
    }
    
        /// Parse an indented block of statements
    fn parse_block_statements(&mut self) -> ParseResult<Vec<Stmt>> {
        if !self.match_token(TokenType::Indent) {
            return Err(ParseError::UnexpectedToken {
                expected: "indented block".to_string(),
                found: self.peek().clone(),
            });
        }

        let mut statements = Vec::new();
        while !self.check(TokenType::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(TokenType::Dedent) {
                break;
            }
            statements.push(self.parse_statement()?);
        }

        if !self.match_token(TokenType::Dedent) {
            return Err(ParseError::UnexpectedToken {
                expected: "dedent to close block".to_string(),
                found: self.peek().clone(),
            });
        }

        Ok(statements)
    }
    
    fn parse_type(&mut self) -> ParseResult<Type> {
        let token = self.peek().clone();
        
        match &token.token_type {
            TokenType::Identifier => {
                self.advance();
                match token.lexeme.as_str() {
                    "i8" => Ok(Type::I8),
                    "i16" => Ok(Type::I16),
                    "i32" => Ok(Type::I32),
                    "i64" => Ok(Type::I64),
                    "f32" => Ok(Type::F32),
                    "f64" => Ok(Type::F64),
                    "bool" => Ok(Type::Bool),
                    "string" => Ok(Type::String),
                    "unit" => Ok(Type::Unit),
                    name => {
                        // User-defined type - for now just create an object type
                        Ok(Type::Object {
                            name: name.to_string(),
                            fields: HashMap::new(),
                        })
                    }
                }
            }
            TokenType::LeftParen => {
                self.advance();
                
                // Check if this is a list type (Type) or map type (Key: Value)
                if self.check(TokenType::RightParen) {
                    // Empty list type: ()
                    self.advance(); // consume ')'
                    return Ok(Type::List(Box::new(Type::Unknown)));
                }
                
                let first_type = self.parse_type()?;
                
                if self.match_token(TokenType::Colon) {
                    // This is a map type: (KeyType: ValueType)
                    let value_type = self.parse_type()?;
                    self.consume(TokenType::RightParen, "Expected ')' after map types")?;
                    Ok(Type::Map(Box::new(first_type), Box::new(value_type)))
                } else {
                    // This is a list type: (ElementType)
                    self.consume(TokenType::RightParen, "Expected ')' after list element type")?;
                    Ok(Type::List(Box::new(first_type)))
                }
            }
            TokenType::Fn => {
                self.advance();
                self.consume(TokenType::LeftParen, "Expected '(' after 'fn' in function type")?;
                
                let mut param_types = Vec::new();
                if !self.check(TokenType::RightParen) {
                    loop {
                        param_types.push(self.parse_type()?);
                        if !self.match_token(TokenType::Comma) {
                            break;
                        }
                    }
                }
                
                self.consume(TokenType::RightParen, "Expected ')' after function parameters")?;
                self.consume(TokenType::Arrow, "Expected '->' after function parameters")?;
                let return_type = self.parse_type()?;
                
                Ok(Type::Function {
                    params: param_types,
                    return_type: Box::new(return_type),
                })
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "type".to_string(),
                found: token.clone(),
            }),
        }
    }
    
    /// Skip only newline tokens, NOT indent/dedent tokens
    /// Optimized version that caches is_at_end check
    fn skip_newlines(&mut self) {
        while self.current < self.tokens.len() && 
              self.tokens[self.current].token_type == TokenType::Newline {
            self.current += 1;
        }
    }
    


    // Operator matching helpers - optimized with lookup tables
    fn match_comparison_op(&mut self) -> Option<BinaryOp> {
        if self.current >= self.tokens.len() {
            return None;
        }
        
        let op = match self.tokens[self.current].token_type {
            TokenType::Gt => Some(BinaryOp::Gt),
            TokenType::Gte => Some(BinaryOp::Ge),
            TokenType::Lt => Some(BinaryOp::Lt),
            TokenType::Lte => Some(BinaryOp::Le),
            _ => None,
        };
        
        if op.is_some() {
            self.current += 1;
        }
        op
    }
    
    fn match_shift_op(&mut self) -> Option<BinaryOp> {
        if self.current >= self.tokens.len() {
            return None;
        }
        
        let op = match self.tokens[self.current].token_type {
            TokenType::LeftShift => Some(BinaryOp::Shl),
            TokenType::RightShift => Some(BinaryOp::Shr),
            _ => None,
        };
        
        if op.is_some() {
            self.current += 1;
        }
        op
    }
    
    fn match_term_op(&mut self) -> Option<BinaryOp> {
        if self.current >= self.tokens.len() {
            return None;
        }
        
        let op = match self.tokens[self.current].token_type {
            TokenType::Plus => Some(BinaryOp::Add),
            TokenType::Minus => Some(BinaryOp::Sub),
            _ => None,
        };
        
        if op.is_some() {
            self.current += 1;
        }
        op
    }
    
    fn match_factor_op(&mut self) -> Option<BinaryOp> {
        if self.current >= self.tokens.len() {
            return None;
        }
        
        let op = match self.tokens[self.current].token_type {
            TokenType::Star => Some(BinaryOp::Mul),
            TokenType::Slash => Some(BinaryOp::Div),
            TokenType::Percent => Some(BinaryOp::Mod),
            _ => None,
        };
        
        if op.is_some() {
            self.current += 1;
        }
        op
    }
    
    fn match_unary_op(&mut self) -> Option<UnaryOp> {
        if self.current >= self.tokens.len() {
            return None;
        }
        
        let op = match self.tokens[self.current].token_type {
            TokenType::Bang => Some(UnaryOp::Not),
            TokenType::Minus => Some(UnaryOp::Neg),
            TokenType::Tilde => Some(UnaryOp::BitNot),
            _ => None,
        };
        
        if op.is_some() {
            self.current += 1;
        }
        
        op
    }
    

    
    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }
    
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.peek().token_type == TokenType::Eof
    }
    
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }
    
    fn _check_at_offset(&self, offset: usize, token_type: TokenType) -> bool {
        if self.current + offset >= self.tokens.len() {
            false
        } else {
            self.tokens[self.current + offset].token_type == token_type
        }
    }
    
    fn consume(&mut self, token_type: TokenType, message: &str) -> ParseResult<Token> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: message.to_string(),
                found: self.peek().clone(),
            })
        }
    }
    
    // Source span helpers
    fn current_span(&self) -> SourceSpan {
        self.token_to_span(self.peek())
    }
    
    fn span_from_current(&self) -> SourceSpan {
        // Safe bounds checking - performance impact is negligible in parsing context
        if self.current > 0 {
            let token = &self.tokens[self.current - 1];
            self.token_to_span(token)
        } else {
            self.current_span()
        }
    }
    
    fn span_from_token(&self, token: &Token) -> SourceSpan {
        self.token_to_span(token)
    }
    
    fn token_to_span(&self, token: &Token) -> SourceSpan {
        // No more filename cloning - Arc<str> is cheap to clone
        SourceSpan::new(
            self.file_name.clone(),
            token.line as u32,
            token.column as u32,
            token.line as u32,
            (token.column + token.lexeme.len()) as u32,
        )
    }
    
    fn span_between(&self, start: &SourceSpan, end: &SourceSpan) -> SourceSpan {
        SourceSpan::new(
            self.file_name.clone(), // Arc<str> clone is cheap
            start.start_line,
            start.start_col,
            end.end_line,
            end.end_col,
        )
    }
    
    fn parse_string_interpolation_from_content(&mut self, content: &str) -> ParseResult<Vec<crate::ast::StringPart>> {
        use crate::ast::StringPart;
        
        let mut parts = Vec::with_capacity(4); // Pre-allocate for typical case
        let mut current_text = String::with_capacity(content.len());
        let mut chars = content.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                '{' => {
                    if chars.peek() == Some(&'{') {
                        // Escaped brace: {{
                        chars.next();
                        current_text.push('{');
                    } else {
                        // Start of interpolation - use optimized parsing
                        if !current_text.is_empty() {
                            parts.push(StringPart::Literal(std::mem::take(&mut current_text)));
                        }
                        
                        let expr = self.parse_interpolated_expression(&mut chars)?;
                        parts.push(StringPart::Expression(expr));
                    }
                }
                '}' => {
                    if chars.peek() == Some(&'}') {
                        // Escaped brace: }}
                        chars.next();
                        current_text.push('}');
                    } else {
                        current_text.push(ch);
                    }
                }
                '\\' => {
                    self.handle_escape_sequence(&mut current_text, &mut chars);
                }
                _ => {
                    current_text.push(ch);
                }
            }
        }
        
        // Add remaining text
        if !current_text.is_empty() {
            parts.push(StringPart::Literal(current_text));
        } else if !parts.is_empty() {
            // If we ended with an expression, add empty literal for consistency with tests
            if matches!(parts.last(), Some(StringPart::Expression(_))) {
                parts.push(StringPart::Literal(String::new()));
            }
        }
        
        // Ensure we always have at least one part
        if parts.is_empty() {
            parts.push(StringPart::Literal(String::new()));
        }
        
        Ok(parts)
    }

    /// Parse an interpolated expression using inline tokenization to avoid allocations
    fn parse_interpolated_expression(&mut self, chars: &mut std::iter::Peekable<std::str::Chars>) -> ParseResult<Expr> {
        let mut expr_text = String::new();
        let mut brace_depth = 1;
        
        // Extract expression content with minimal allocations
        while let Some(expr_ch) = chars.next() {
            match expr_ch {
                '{' => {
                    brace_depth += 1;
                    expr_text.push(expr_ch);
                }
                '}' => {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        break;
                    }
                    expr_text.push(expr_ch);
                }
                _ => expr_text.push(expr_ch),
            }
        }
        
        if brace_depth > 0 {
            return Err(ParseError::InvalidSyntax {
                message: "Unclosed interpolation brace in string".to_string(),
                span: self.span_from_current(),
            });
        }
        
        if expr_text.trim().is_empty() {
            return Err(ParseError::InvalidSyntax {
                message: "Empty interpolation expression".to_string(),
                span: self.span_from_current(),
            });
        }
        
        // Fast path for simple identifiers (most common case)
        let trimmed = expr_text.trim();
        if self.is_simple_identifier(trimmed) {
            return Ok(Expr::new(
                self.span_from_current(),
                ExprKind::identifier(trimmed.to_string())
            ));
        }
        
        // Fall back to full parsing for complex expressions
        self.parse_expression_from_string(&expr_text)
    }
    
    /// Check if a string is a simple identifier (performance optimization)
    fn is_simple_identifier(&self, s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$')
    }

    fn handle_escape_sequence(&mut self, current_text: &mut String, chars: &mut std::iter::Peekable<std::str::Chars>) {
        if let Some(escaped) = chars.next() {
            match escaped {
                'n' => current_text.push('\n'),
                't' => current_text.push('\t'),
                'r' => current_text.push('\r'),
                '\\' => current_text.push('\\'),
                '\'' => current_text.push('\''),
                '"' => current_text.push('"'),
                '{' => current_text.push('{'),
                '}' => current_text.push('}'),
                c => {
                    current_text.push('\\');
                    current_text.push(c);
                }
            }
        } else {
            current_text.push('\\');
        }
    }
    
    fn parse_expression_from_string(&mut self, expr_str: &str) -> ParseResult<Expr> {
        // Create a mini lexer and parser for the expression
        use crate::lexer::Lexer;
        
        let mut lexer = Lexer::new(expr_str.to_string(), self.file_name.to_string());
        let tokens = lexer.tokenize().map_err(|e| ParseError::InvalidSyntax {
            message: format!("Lexer error in interpolated expression '{}': {}", expr_str, e),
            span: self.span_from_current(),
        })?;
        
        let mut expr_parser = Parser::new(tokens, self.file_name.to_string());
        expr_parser.parse_expression().map_err(|e| {
            // Preserve the original error type and provide better context
            match e {
                ParseError::UnexpectedToken { expected, found } => ParseError::InvalidSyntax {
                    message: format!(
                        "Invalid interpolated expression '{}': expected {}, found {}",
                        expr_str, expected, found.lexeme
                    ),
                    span: self.span_from_current(),
                },
                ParseError::InvalidSyntax { message, .. } => ParseError::InvalidSyntax {
                    message: format!("Invalid interpolated expression '{}': {}", expr_str, message),
                    span: self.span_from_current(),
                },
                other => other, // Preserve other error types as-is
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::*;
    use crate::ast::StringPart;

    fn parse_expression(input: &str) -> ParseResult<Expr> {
        let mut lexer = Lexer::new(input.to_string(), "test".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens, "test".to_string());
        parser.parse_expression()
    }
    
    fn parse_statement(input: &str) -> ParseResult<Stmt> {
        let mut lexer = Lexer::new(input.to_string(), "test".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens, "test".to_string());
        parser.parse_statement()
    }
    
    #[test]
    fn test_literal_expressions() {
        let expr = parse_expression("42").unwrap();
        assert!(matches!(expr.kind, ExprKind::Literal(Literal::Integer(42))));
        
        let expr = parse_expression("3.14").unwrap();
        assert!(matches!(expr.kind, ExprKind::Literal(Literal::Float(f)) if (f - 3.14).abs() < f64::EPSILON));
        
        let expr = parse_expression("true").unwrap();
        assert!(matches!(expr.kind, ExprKind::Literal(Literal::Bool(true))));
        
        let expr = parse_expression("\"hello\"").unwrap();
        assert!(matches!(expr.kind, ExprKind::Literal(Literal::String(ref s)) if s == "hello"));
    }
    
    #[test]
    fn test_binary_expressions() {
        let expr = parse_expression("1 + 2").unwrap();
        if let ExprKind::Binary { op, .. } = expr.kind {
            assert_eq!(op, BinaryOp::Add);
        } else {
            panic!("Expected binary expression");
        }
        
        let expr = parse_expression("a == b").unwrap();
        if let ExprKind::Binary { op, .. } = expr.kind {
            assert_eq!(op, BinaryOp::Eq);
        } else {
            panic!("Expected binary expression");
        }
    }
    
    #[test]
    fn test_function_call() {
        let expr = parse_expression("foo(1, 2, 3)").unwrap();
        if let ExprKind::Call { args, .. } = expr.kind {
            assert_eq!(args.len(), 3);
        } else {
            panic!("Expected function call");
        }
    }
    
    #[test]
    fn test_assignment_statement() {
        let stmt = parse_statement("x is 42").unwrap();
        if let StmtKind::Assignment { target, value } = stmt.kind {
            assert!(matches!(target.kind, ExprKind::Identifier(ref s) if s == "x"));
            assert!(matches!(value.kind, ExprKind::Literal(Literal::Integer(42))));
        } else {
            panic!("Expected assignment statement");
        }
    }
    
    #[test]
    fn test_function_definition() {
        // Test indentation-based function syntax without colons
        let code = "fn add(a: i32, b: i32) -> i32\n    return a + b";
        let stmt = parse_statement(code).unwrap();
        if let StmtKind::Function { name, params, return_type, .. } = stmt.kind {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].name, "a");
            assert_eq!(params[1].name, "b");
            assert!(matches!(return_type, Some(Type::I32)));
        } else {
            panic!("Expected function definition");
        }
    }
    
    #[test]
    fn test_assignment_with_is() {
        let stmt = parse_statement("foo is 99").unwrap();
        if let StmtKind::Assignment { target, value } = stmt.kind {
            assert!(matches!(target.kind, ExprKind::Identifier(ref s) if s == "foo"));
            assert!(matches!(value.kind, ExprKind::Literal(Literal::Integer(99))));
        } else {
            panic!("Expected assignment statement");
        }
    }

    #[test]
    fn test_assignment_with_equal() {
        let stmt = parse_statement("bar is 7").unwrap();
        if let StmtKind::Assignment { target, value } = stmt.kind {
            assert!(matches!(target.kind, ExprKind::Identifier(ref s) if s == "bar"));
            assert!(matches!(value.kind, ExprKind::Literal(Literal::Integer(7))));
        } else {
            panic!("Expected assignment statement");
        }
    }

    #[test]
    fn test_assignment_with_is_and_expression() {
        let stmt = parse_statement("result is 1 + 2 * 3").unwrap();
        if let StmtKind::Assignment { target, value } = stmt.kind {
            assert!(matches!(target.kind, ExprKind::Identifier(ref s) if s == "result"));
            // Should parse as binary expression
            if let ExprKind::Binary { op, .. } = value.kind {
                assert_eq!(op, BinaryOp::Add);
            } else {
                panic!("Expected binary expression");
            }
        } else {
            panic!("Expected assignment statement");
        }
    }

    #[test]
    fn test_string_interpolation() {
        let expr = parse_expression("'hello {name}'").unwrap();
        if let ExprKind::StringInterpolation { parts } = &expr.kind {
            assert_eq!(parts.len(), 3);
            
            if let crate::ast::StringPart::Literal(s) = &parts[0] {
                assert_eq!(s, "hello ");
            } else {
                panic!("Expected literal part");
            }
            
            if let crate::ast::StringPart::Expression(expr) = &parts[1] {
                if let ExprKind::Identifier(name) = &expr.kind {
                    assert_eq!(name, "name");
                } else {
                    panic!("Expected identifier expression");
                }
            } else {
                panic!("Expected expression part");
            }
            
            if let crate::ast::StringPart::Literal(s) = &parts[2] {
                assert_eq!(s, "");
            } else {
                panic!("Expected literal part");
            }
        } else {
            panic!("Expected string interpolation");
        }
    }
    
    #[test]
    fn test_nested_string_interpolation() {
        let expr = parse_expression("'Value: {name}'").unwrap();
        if let ExprKind::StringInterpolation { parts } = expr.kind {
            assert_eq!(parts.len(), 3);
            assert!(matches!(parts[0], StringPart::Literal(ref s) if s == "Value: "));
            assert!(matches!(parts[1], StringPart::Expression(_)));
            assert!(matches!(parts[2], StringPart::Literal(ref s) if s == ""));
        } else {
            panic!("Expected string interpolation expression");
        }
    }
    
    #[test]
    fn test_string_interpolation_with_escaped_braces() {
        let expr = parse_expression("'Value: {{value}}'").unwrap();
        if let ExprKind::StringInterpolation { parts } = &expr.kind {
            assert_eq!(parts.len(), 1);
            assert!(matches!(parts[0], StringPart::Literal(ref s) if s == "Value: {value}"));
        } else {
            panic!("Expected string interpolation expression");
        }
    }
    
    #[test]
    fn test_function_definition_with_default_value() {
        let code = "fn test_func(a, b ? 10)\n    return a + b";
        let stmt = parse_statement(code).unwrap();
        if let StmtKind::Function { name, params, .. } = stmt.kind {
            assert_eq!(name, "test_func");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].name, "a");
            assert!(params[0].default_value.is_none());
            assert_eq!(params[1].name, "b");
            assert!(params[1].default_value.is_some());
            if let Some(expr) = &params[1].default_value {
                assert!(matches!(expr.kind, ExprKind::Literal(Literal::Integer(10))));
            }
        } else {
            panic!("Expected function definition");
        }
    }
}
