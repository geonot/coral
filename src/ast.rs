#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment(Assignment),
    FunctionDef(FunctionDef),
    ObjectDef(ObjectDef),
    StoreDef(StoreDef),
    ActorDef(ActorDef),
    ExpressionStmt(Expression),
    UseStatement(UseStatement),
    Comment(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub identifier: String,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub default_value: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectDef {
    pub name: String,
    pub properties: Vec<PropertyDef>,
    pub methods: Vec<MethodDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StoreDef {
    pub name: String,
    pub properties: Vec<PropertyDef>,
    pub methods: Vec<MethodDef>,
    pub make_method: Option<MethodDef>,
    pub as_methods: Vec<AsMethodDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActorDef {
    pub name: String,
    pub properties: Vec<PropertyDef>,
    pub methods: Vec<MethodDef>,
    pub join_tables: Vec<String>,
    pub message_handlers: Vec<MessageHandler>,
    pub make_method: Option<MethodDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyDef {
    pub name: String,
    pub default_value: Option<Expression>,
    pub doc_comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodDef {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageHandler {
    pub message_type: String,
    pub parameters: Vec<Parameter>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AsMethodDef {
    pub conversion_type: String,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UseStatement {
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier(String),
    Integer(i64),
    Float(f64),
    StringLiteral(String),
    InterpolatedString(String),
    Boolean(bool),
    Array(Vec<Expression>),
    ObjectLiteral(Vec<(String, Expression)>),
    ParameterRef(String),
    
    // Binary operations
    Binary {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>,
    },
    
    // Unary operations
    Unary {
        operator: UnaryOp,
        operand: Box<Expression>,
    },
    
    // Ternary conditional
    Ternary {
        condition: Box<Expression>,
        true_expr: Box<Expression>,
        false_expr: Box<Expression>,
    },
    
    // Function/method calls
    FunctionCall {
        name: String,
        args: Vec<Expression>,
        named_args: Vec<(String, Expression)>,
    },
    
    MethodCall {
        object: Box<Expression>,
        method: String,
        args: Vec<Expression>,
        named_args: Vec<(String, Expression)>,
        force_call: bool, // for method!
        chaining: Option<Box<MethodChain>>,
    },
    
    // Array access
    ArrayAccess {
        array: Box<Expression>,
        index: Box<Expression>,
        use_at_keyword: bool, // true for "at", false for "@"
    },
    
    // Object instantiation
    Instantiation {
        type_name: String,
        args: Vec<Expression>,
        named_args: Vec<(String, Expression)>,
        force_success: bool, // for object!
    },
    
    // Special Coral constructs
    AcrossIteration {
        operation: Box<Expression>,
        collection: Box<Expression>,
        with_args: Vec<Expression>,
        into_var: Option<String>,
    },
    
    IterateStatement {
        collection: Box<Expression>,
        operation: Box<Expression>,
        placeholder: Box<Expression>,
    },
    
    // Error handling
    ErrorHandling {
        expression: Box<Expression>,
        error_action: Box<ErrorAction>,
    },
    
    // Data conversion
    AsConversion {
        expression: Box<Expression>,
        target_type: String,
    },
    
    // Control flow
    IfExpression {
        condition: Box<Expression>,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    
    UnlessExpression {
        condition: Box<Expression>,
        body: Vec<Statement>,
        is_postfix: bool,
    },
    
    WhileLoop {
        condition: Box<Expression>,
        body: Vec<Statement>,
    },
    
    UntilLoop {
        iterator: String,
        start_value: Option<Box<Expression>>,
        step_value: Option<Box<Expression>>,
        end_condition: Box<Expression>,
        body: Vec<Statement>,
    },
    
    // Operations
    LogOperation {
        message: Box<Expression>,
    },
    
    PushOperation {
        item: Box<Expression>,
    },
    
    IterateOperation {
        collection: Box<Expression>,
        function: Box<Expression>,
        param_ref: Option<Box<Expression>>,
    },
    
    AcrossOperation {
        function_name: String,
        collection: Box<Expression>,
        result_var: Option<String>,
        named_params: Vec<(String, Expression)>,
    },
    
    UnlessBlock {
        condition: Box<Expression>,
        action: Box<Expression>,
    },
    
    MessageHandler {
        message_type: String,
        parameters: Vec<Parameter>,
        body: Vec<Statement>,
    },
    
    // Literals
    Empty,
    Now,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodChain {
    pub connector: ChainConnector, // "then" or "and"
    pub method: String,
    pub args: Vec<Expression>,
    pub force_call: bool,
    pub next: Option<Box<MethodChain>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChainConnector {
    Then,
    And,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    
    // Coral-style comparisons
    GreaterThan,
    LessThan,
    Equals,
    GreaterThanOrEqual,
    LessThanOrEqual,
    
    // Traditional comparisons
    Equal,
    NotEqual,
    
    // Logical
    And,
    Or,
    
    // Bitwise
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not,
    Minus,
    BitwiseNot,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorAction {
    LogReturn,
    DefaultValue(Expression),
    ReturnLogError,
}