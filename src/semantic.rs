use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub length: Option<usize>,
    pub error_type: ErrorType,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.col, self.message)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    UnexpectedToken,
    MissingToken,
    InvalidSyntax,
    IndentationError,
    SemanticError,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub defined_at: (usize, usize), // line, col
    pub used_at: Vec<(usize, usize)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Variable(Option<String>), // type hint
    Function(Vec<String>, Option<String>), // params, return type
    Object(Vec<String>), // properties
    Store(Vec<String>), // properties
    Actor(Vec<String>), // properties + message handlers
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub symbols: std::collections::HashMap<String, Symbol>,
    pub parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            symbols: std::collections::HashMap::new(),
            parent: None,
        }
    }
    
    pub fn with_parent(parent: Scope) -> Self {
        Self {
            symbols: std::collections::HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }
    
    pub fn define(&mut self, name: String, symbol: Symbol) -> Result<(), ParseError> {
        if self.symbols.contains_key(&name) {
            return Err(ParseError {
                message: format!("Symbol '{}' already defined", name),
                line: symbol.defined_at.0,
                col: symbol.defined_at.1,
                length: Some(name.len()),
                error_type: ErrorType::SemanticError,
            });
        }
        self.symbols.insert(name, symbol);
        Ok(())
    }
    
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name).or_else(|| {
            self.parent.as_ref().and_then(|p| p.lookup(name))
        })
    }
}