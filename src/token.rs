#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    Identifier(String),
    Integer(String),
    Float(String),
    StringLiteral(String),
    InterpolatedString(String),
    CharLiteral(char),
    Boolean(bool),
    ErrorCode(String),
    ParameterRef(String),  // $id, $0, $1, etc.
    
    // Keywords
    Fn, Class, Actor, Persistent, Struct, Enum, Store, Object,
    If, Else, For, In, While, Match, Return,
    Break, Continue, Pass, Extends, Implements, Use,
    This, Super, Where, Select, OrderBy, Aspect,
    Override, Init, Is, As, Import, Underscore,
    
    // Coral-specific keywords
    With, Across, Into, Unless, Until, Iterate,
    At, Err, Log, Make, Empty, Now, By, From, On,
    Then, And, Or, // Method chaining and logical operators
    Push, Pop, // Collection operations
    Gt, Lt, Equals, Gte, Lte, // Coral comparison operators
    
    // Operators
    Plus, Minus, Star, Slash, Percent, DoubleStar,
    EqEq, BangEq, LtEq, GtEq,
    AmpAmp, PipePipe, Bang,
    Eq, PlusEq, MinusEq, StarEq, SlashEq, PercentEq, DoubleStarEq,
    AmpEq, PipeEq, CaretEq, LtLtEq, GtGtEq,
    Dot, Comma, Colon, DoubleColon,
    LParen, RParen, LBracket, RBracket,
    Arrow,
    DotDot, DotDotEq,
    Question,
    QuestionDot,
    DoubleQuestion,
    Tilde, Amp, Pipe, Caret, LtLt, GtGt,
    AmpRef, // & for join table references
    
    // Structure tokens
    Indent,
    Dedent,
    Newline,
    AnnotationMarker, // @
    LBrace,
    RBrace,
    
    // Special
    Eof,
    Illegal(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub col: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

impl Token {
    pub fn new(kind: TokenType, lexeme: String, line: usize, col: usize, start_byte: usize, end_byte: usize) -> Self {
        Token { kind, lexeme, line, col, start_byte, end_byte }
    }

    pub fn eof(line: usize, col: usize, byte_pos: usize) -> Self {
        Token::new(TokenType::Eof, "".to_string(), line, col, byte_pos, byte_pos)
    }

    pub fn identifier_or_keyword(ident_str: String, line: usize, col: usize, start_byte: usize, end_byte: usize) -> Self {
        let kind = match ident_str.as_str() {
            "fn" => TokenType::Fn,
            "class" => TokenType::Class,
            "actor" => TokenType::Actor,
            "persistent" => TokenType::Persistent,
            "struct" => TokenType::Struct,
            "enum" => TokenType::Enum,
            "store" => TokenType::Store,
            "object" => TokenType::Object,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "for" => TokenType::For,
            "in" => TokenType::In,
            "while" => TokenType::While,
            "match" => TokenType::Match,
            "return" => TokenType::Return,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            "pass" => TokenType::Pass,
            "extends" => TokenType::Extends,
            "implements" => TokenType::Implements,
            "use" => TokenType::Use,
            "this" => TokenType::This,
            "super" => TokenType::Super,
            "where" => TokenType::Where,
            "select" => TokenType::Select,
            "orderby" => TokenType::OrderBy,
            "aspect" => TokenType::Aspect,
            "override" => TokenType::Override,
            "init" => TokenType::Init,
            "is" => TokenType::Is,
            "as" => TokenType::As,
            "import" => TokenType::Import,
            "_" => TokenType::Underscore,
            "true" => TokenType::Boolean(true),
            "false" => TokenType::Boolean(false),
            "with" => TokenType::With,
            "across" => TokenType::Across,
            "into" => TokenType::Into,
            "unless" => TokenType::Unless,
            "until" => TokenType::Until,
            "iterate" => TokenType::Iterate,
            "at" => TokenType::At,
            "err" => TokenType::Err,
            "log" => TokenType::Log,
            "make" => TokenType::Make,
            "empty" => TokenType::Empty,
            "now" => TokenType::Now,
            "by" => TokenType::By,
            "yes" => TokenType::Boolean(true),
            "no" => TokenType::Boolean(false),
            "gt" => TokenType::Gt,
            "lt" => TokenType::Lt,
            "equals" => TokenType::Equals,
            "then" => TokenType::Then,
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "push" => TokenType::Push,
            "pop" => TokenType::Pop,
            "gte" => TokenType::Gte,
            "lte" => TokenType::Lte,
            _ => TokenType::Identifier(ident_str.clone()),
        };
        Token::new(kind, ident_str, line, col, start_byte, end_byte)
    }
}