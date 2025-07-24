use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};

/// Source location information
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    pub file: std::sync::Arc<str>, // Use Arc<str> for shared filename
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

impl SourceSpan {
    pub fn new(file: impl Into<std::sync::Arc<str>>, start_line: u32, start_col: u32, end_line: u32, end_col: u32) -> Self {
        Self { 
            file: file.into(), 
            start_line, 
            start_col, 
            end_line, 
            end_col 
        }
    }
    
    pub fn single_char(file: impl Into<std::sync::Arc<str>>, line: u32, col: u32) -> Self {
        Self::new(file, line, col, line, col + 1)
    }
}

// Global node ID counter for unique AST node identification
static NODE_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Unique identifier for each AST node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

impl NodeId {
    pub fn new() -> Self {
        NodeId(NODE_COUNTER.fetch_add(1, Ordering::SeqCst))
    }
    
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

/// Comprehensive type system for Coral
#[derive(Debug, Clone, Eq)]
pub enum Type {
    // Primitive types
    I8, I16, I32, I64,
    F32, F64,
    Bool,
    String,
    
    // Collection types
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),
    
    // Function types
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    
    // User-defined types
    Object {
        name: String,
        fields: HashMap<String, Type>,
    },
    Store {
        name: String,
        value_type: Box<Type>,
    },
    Actor {
        name: String,
        message_types: Vec<Type>,
    },
    
    // Type variables for inference
    TypeVar(u32),
    
    // Result type for error handling
    Result(Box<Type>, Box<Type>),

    Pipe(Box<Type>),
    
    // Unit type
    Unit,
    
    // Unknown type (for inference)
    Unknown,
}

use std::hash::{Hash, Hasher};

impl Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Type::I8 => "i8".hash(state),
            Type::I16 => "i16".hash(state),
            Type::I32 => "i32".hash(state),
            Type::I64 => "i64".hash(state),
            Type::F32 => "f32".hash(state),
            Type::F64 => "f64".hash(state),
            Type::Bool => "bool".hash(state),
            Type::String => "string".hash(state),
            Type::Unit => "unit".hash(state),
            Type::List(t) => {
                "list".hash(state);
                t.hash(state);
            }
            Type::Map(k, v) => {
                "map".hash(state);
                k.hash(state);
                v.hash(state);
            }
            Type::Function { params, return_type } => {
                "function".hash(state);
                params.hash(state);
                return_type.hash(state);
            }
            Type::Object { name, .. } => {
                "object".hash(state);
                name.hash(state);
            }
            Type::Store { name, .. } => {
                "store".hash(state);
                name.hash(state);
            }
            Type::Actor { name, .. } => {
                "actor".hash(state);
                name.hash(state);
            }
            Type::TypeVar(id) => {
                "typevar".hash(state);
                id.hash(state);
            }
            Type::Result(o, e) => {
                "result".hash(state);
                o.hash(state);
                e.hash(state);
            }
            Type::Pipe(t) => {
                "pipe".hash(state);
                t.hash(state);
            }
            Type::Unknown => "unknown".hash(state),
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::I8, Type::I8) => true,
            (Type::I16, Type::I16) => true,
            (Type::I32, Type::I32) => true,
            (Type::I64, Type::I64) => true,
            (Type::F32, Type::F32) => true,
            (Type::F64, Type::F64) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::String, Type::String) => true,
            (Type::Unit, Type::Unit) => true,
            (Type::List(a), Type::List(b)) => a == b,
            (Type::Map(k1, v1), Type::Map(k2, v2)) => k1 == k2 && v1 == v2,
            (Type::Function { params: p1, return_type: r1 }, Type::Function { params: p2, return_type: r2 }) => {
                p1 == p2 && r1 == r2
            }
            (Type::Object { name: n1, .. }, Type::Object { name: n2, .. }) => n1 == n2,
            (Type::Store { name: n1, .. }, Type::Store { name: n2, .. }) => n1 == n2,
            (Type::Actor { name: n1, .. }, Type::Actor { name: n2, .. }) => n1 == n2,
            (Type::TypeVar(id1), Type::TypeVar(id2)) => id1 == id2,
            (Type::Result(o1, e1), Type::Result(o2, e2)) => o1 == o2 && e1 == e2,
            (Type::Pipe(t1), Type::Pipe(t2)) => t1 == t2,
            (Type::Unknown, Type::Unknown) => true,
            _ => false,
        }
    }
}

impl Type {
    pub fn new_type_var() -> Self {
        static TYPE_VAR_COUNTER: AtomicU32 = AtomicU32::new(0);
        Type::TypeVar(TYPE_VAR_COUNTER.fetch_add(1, Ordering::SeqCst))
    }
    
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::F32 | Type::F64)
    }
    
    pub fn is_integer(&self) -> bool {
        matches!(self, Type::I8 | Type::I16 | Type::I32 | Type::I64)
    }
    
    pub fn is_float(&self) -> bool {
        matches!(self, Type::F32 | Type::F64)
    }
    
    /// Get the size in bytes for primitive types
    pub fn size_bytes(&self) -> Option<usize> {
        match self {
            Type::I8 => Some(1),
            Type::I16 => Some(2),
            Type::I32 => Some(4),
            Type::I64 => Some(8),
            Type::F32 => Some(4),
            Type::F64 => Some(8),
            Type::Bool => Some(1),
            Type::Unit => Some(0),
            _ => None,
        }
    }
    
    /// Get LLVM type string representation
    pub fn to_llvm_type(&self) -> String {
        match self {
            Type::I8 => "i8".to_string(),
            Type::I16 => "i16".to_string(),
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::F32 => "float".to_string(),
            Type::F64 => "double".to_string(),
            Type::Bool => "i1".to_string(),
            Type::String => "i8*".to_string(),
            Type::Unit => "void".to_string(),
            Type::List(elem_type) => format!("{}*", elem_type.to_llvm_type()),
            Type::Function { params, return_type } => {
                let param_types: Vec<String> = params.iter().map(|t| t.to_llvm_type()).collect();
                format!("{}({})*", return_type.to_llvm_type(), param_types.join(", "))
            }
            _ => "i8*".to_string(), // Default to pointer for complex types
        }
    }
    
    /// Display type information
    pub fn to_string(&self) -> String {
        match self {
            Type::I8 => "i8".to_string(),
            Type::I16 => "i16".to_string(),
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "string".to_string(),
            Type::List(inner) => format!("list<{}>", inner.to_string()),
            Type::Map(key, value) => format!("map<{}, {}>", key.to_string(), value.to_string()),
            Type::Function { params, return_type } => {
                let param_types: Vec<String> = params.iter().map(|t| t.to_string()).collect();
                format!("({}) -> {}", param_types.join(", "), return_type.to_string())
            },
            Type::Object { name, .. } => format!("object {}", name),
            Type::Store { name, value_type } => format!("store {} of {}", name, value_type.to_string()),
            Type::Actor { name, .. } => format!("actor {}", name),
            Type::TypeVar(id) => format!("T{}", id),
            Type::Result(ok, err) => format!("Result<{}, {}>", ok.to_string(), err.to_string()),
            Type::Pipe(inner) => format!("pipe<{}>", inner.to_string()),
            Type::Unit => "unit".to_string(),
            Type::Unknown => "unknown".to_string(),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::I8 => write!(f, "i8"),
            Type::I16 => write!(f, "i16"),
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::List(inner) => write!(f, "list<{}>", inner),
            Type::Map(key, value) => write!(f, "map<{}, {}>", key, value),
            Type::Function { params, return_type } => {
                write!(f, "(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", return_type)
            },
            Type::Object { name, .. } => write!(f, "object {}", name),
            Type::Store { name, value_type } => write!(f, "store {} of {}", name, value_type),
            Type::Actor { name, .. } => write!(f, "actor {}", name),
            Type::TypeVar(id) => write!(f, "T{}", id),
            Type::Result(ok, err) => write!(f, "Result<{}, {}>", ok, err),
            Type::Pipe(inner) => write!(f, "pipe<{}>", inner),
            Type::Unit => write!(f, "unit"),
            Type::Unknown => write!(f, "unknown"),
        }
    }
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    // Arithmetic
    Add, Sub, Mul, Div, Mod,
    
    // Comparison
    Eq, Ne, Lt, Le, Gt, Ge,
    Is, // Type or value identity comparison (x is y)
    // Logical
    And, Or, Xor, // Added Xor for logical exclusive or
    
    // Bitwise
    BitAnd, BitOr, BitXor, Shl, Shr,
}

/// Trait for binary comparators that can be used as truth-value queries (qfn)
pub trait QueryComparator {
    /// Returns true if this BinaryOp is a query comparator (eq, ne, lt, le, gt, ge, is, xor)
    fn is_query(&self) -> bool;
}

impl QueryComparator for BinaryOp {
    fn is_query(&self) -> bool {
        matches!(self, BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge | BinaryOp::Is | BinaryOp::Xor)
    }
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Not, Neg, BitNot,
}

/// Literal values including Coral-specific literals
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Unit,
    No,       // Represents null/nil/none
    Yes,
    Empty,    // Represents empty collection
    None,
    Now,      // Represents current timestamp
    Err,      // Represents an error (for error handling)
}

/// Parts of an interpolated string
#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String),    // Static text like "hello "
    Expression(Expr),   // Embedded expression like {name}
}

/// Expressions with full AST node information
#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub id: NodeId,
    pub span: SourceSpan,
    pub type_: Type,
    pub kind: ExprKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Literal(Literal),
    Identifier(String),
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Argument>,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    ListLiteral(Vec<Expr>),
    MapLiteral(Vec<(Expr, Expr)>),
    ListAppend {
        list: Box<Expr>,
        element: Box<Expr>,
    },
    MapInsert {
        map: Box<Expr>,
        key: Box<Expr>,
        value: Box<Expr>,
    },
    Across {
        callee: Box<Expr>,
        iterable: Box<Expr>,
        into: Option<String>,
    },
    StringInterpolation {
        // For 'hello {name}' -> parts: ["hello ", name_expr, ""]
        parts: Vec<StringPart>,
    },
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },
    Block(Vec<Stmt>),
    Lambda {
        params: Vec<Parameter>,
        body: Box<Expr>,
    },
    Pipe {
        name: String,
        source: String,
        destination: String,
        nocopy: bool,
    },
    Io {
        op: String,
        args: Vec<Expr>,
        nocopy: bool,
    },
}

impl Expr {
    pub fn new(span: SourceSpan, kind: ExprKind) -> Self {
        Self {
            id: NodeId::new(),
            span,
            type_: Type::Unknown,
            kind,
        }
    }
    
    pub fn with_type(mut self, type_: Type) -> Self {
        self.type_ = type_;
        self
    }
}

/// Statements with full AST node information
#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub id: NodeId,
    pub span: SourceSpan,
    pub kind: StmtKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    Expression(Expr),
    Assignment {
        target: Expr,
        value: Expr,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    Unless {
        condition: Expr,
        body: Vec<Stmt>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Until {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Iterate {
        iterable: Expr,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    Break,
    Continue,
    Function {
        name: String,
        params: Vec<Parameter>,
        return_type: Option<Type>,
        body: Vec<Stmt>,
    },
    Object {
        name: String,
        fields: Vec<Field>,
        methods: Vec<ObjectMethod>,  // Methods within the object
    },
    Store {
        name: String,
        fields: Vec<Field>,
        methods: Vec<ObjectMethod>,
    },
    Actor {
        name: String,
        fields: Vec<Field>,
        handlers: Vec<MessageHandler>,
    },
    Import {
        module: String,
        items: Option<Vec<String>>,
    },
    ErrorHandler {
        handler: ErrorHandler,
        inner: Box<Stmt>, // The statement/expression being guarded
    },
    Pipe {
        name: String,
        source: String,
        destination: String,
        nocopy: bool,
    },
    Io {
        op: String,
        args: Vec<Expr>,
        nocopy: bool,
    },
}

impl Stmt {
    pub fn new(span: SourceSpan, kind: StmtKind) -> Self {
        Self {
            id: NodeId::new(),
            span,
            kind,
        }
    }
}

/// Error handler chain for statements like `call() err log return`
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorHandler {
    pub actions: Vec<ErrorAction>,
    pub span: SourceSpan,
}

/// Individual error handling actions (log, return, custom)
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorAction {
    Log(Option<Expr>),      // log or log expr
    Return(Option<Expr>),   // return or return expr
    Custom(Expr),           // custom error handler expression
}

/// An argument passed in a function call
#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    pub name: Option<String>,
    pub value: Expr,
    pub span: SourceSpan,
}

/// Function parameter with optional default value
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_: Type,
    pub default_value: Option<Expr>,  // For default parameters: fn greet(name, greeting 'hello')
    pub span: SourceSpan,
}

/// Object field definition
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub type_: Type,
    pub default_value: Option<Expr>,
    pub span: SourceSpan,
}

/// Object method definition (methods within objects)
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectMethod {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Vec<Stmt>,
    pub span: SourceSpan,
}

/// Actor message handler
#[derive(Debug, Clone, PartialEq)]
pub struct MessageHandler {
    pub message_type: Type,
    pub body: Vec<Stmt>,
    pub span: SourceSpan,
}

/// Top-level program
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
    pub span: SourceSpan,
}

/// AST visitor trait for traversals
pub trait Visitor<T> {
    fn visit_program(&mut self, program: &Program) -> T;
    fn visit_stmt(&mut self, stmt: &Stmt) -> T;
    fn visit_expr(&mut self, expr: &Expr) -> T;
    fn visit_type(&mut self, type_: &Type) -> T;
}

/// Mutable AST visitor trait for transformations
pub trait VisitorMut {
    fn visit_program_mut(&mut self, program: &mut Program);
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt);
    fn visit_expr_mut(&mut self, expr: &mut Expr);
    fn visit_type_mut(&mut self, type_: &mut Type);
}

/// Utility functions for AST construction
impl ExprKind {
    pub fn literal(lit: Literal) -> Self {
        ExprKind::Literal(lit)
    }
    
    pub fn identifier(name: impl Into<String>) -> Self {
        ExprKind::Identifier(name.into())
    }
    
    pub fn binary(op: BinaryOp, left: Expr, right: Expr) -> Self {
        ExprKind::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
    
    pub fn call(callee: Expr, args: Vec<Argument>) -> Self {
        ExprKind::Call {
            callee: Box::new(callee),
            args,
        }
    }
}

impl From<Literal> for ExprKind {
    fn from(lit: Literal) -> Self {
        ExprKind::Literal(lit)
    }
}

impl StmtKind {
    pub fn function(name: impl Into<String>, params: Vec<Parameter>, body: Vec<Stmt>) -> Self {
        StmtKind::Function {
            name: name.into(),
            params,
            return_type: None,
            body,
        }
    }
}

/// Trait for truth-value inference for any expression or value
pub trait Truthy {
    /// Returns true if the value is considered 'true' in Coral
    fn is_truthy(&self) -> bool;
}

impl Truthy for Literal {
    fn is_truthy(&self) -> bool {
        match self {
            Literal::Bool(false) => false,
            Literal::Err => false,
            _ => true,
        }
    }
}

/// Helper for creating source spans during testing
impl Default for SourceSpan {
    fn default() -> Self {
        SourceSpan::new("test".to_string(), 1, 1, 1, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_uniqueness() {
        let id1 = NodeId::new();
        let id2 = NodeId::new();
        assert_ne!(id1, id2);
    }
    
    #[test]
    fn test_type_predicates() {
        assert!(Type::I32.is_numeric());
        assert!(Type::I32.is_integer());
        assert!(!Type::I32.is_float());
        
        assert!(Type::F64.is_numeric());
        assert!(!Type::F64.is_integer());
        assert!(Type::F64.is_float());
        
        assert!(!Type::Bool.is_numeric());
    }
    
    #[test]
    fn test_expr_construction() {
        let span = SourceSpan::default();
        let expr = Expr::new(span.clone(), ExprKind::literal(Literal::Integer(42)));
        
        assert_eq!(expr.span, span);
        assert_eq!(expr.type_, Type::Unknown);
        assert!(matches!(expr.kind, ExprKind::Literal(Literal::Integer(42))));
    }
    
    #[test]
    fn test_binary_expr_construction() {
        let span = SourceSpan::default();
        let left = Expr::new(span.clone(), ExprKind::literal(Literal::Integer(1)));
        let right = Expr::new(span.clone(), ExprKind::literal(Literal::Integer(2)));
        
        let binary = Expr::new(span.clone(), ExprKind::binary(BinaryOp::Add, left, right));
        
        if let ExprKind::Binary { op, .. } = &binary.kind {
            assert_eq!(*op, BinaryOp::Add);
        } else {
            panic!("Expected binary expression");
        }
    }
    
    #[test]
    fn test_query_comparator_trait() {
        assert!(BinaryOp::Eq.is_query());
        assert!(BinaryOp::Is.is_query());
        assert!(BinaryOp::Xor.is_query());
        assert!(!BinaryOp::Add.is_query());
        assert!(!BinaryOp::And.is_query());
    }
}