use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    // Literals
    Integer,
    Float,
    String,
    InterpolatedString,  // Single-quoted strings that may contain interpolation
    True,
    False,
    Identifier,
    
    // Keywords
    Let,
    Fn,
    Is,            // Primary assignment operator in Coral
    Object,
    Store,
    Actor,
    Use,
    Mod,
    If,
    Then,
    Else,
    While,
    For,
    In,
    Until,
    Unless,
    Iterate,
    Across,
    Return,
    Break,
    Continue,
    Import,
    From,
    To,
    Nocopy,
    Err,
    No,
    Yes,
    Empty,
    Now,
    Plus,          // +
    Minus,         // -
    Star,          // *
    Slash,         // /
    Percent,       // %
    Equal,         // =
    EqualEqual,    // ==
    BangEqual,     // !=
    Less,          // <
    LessEqual,     // <=
    Greater,       // >
    GreaterEqual,  // >=
    And,           // and
    Or,            // or
    Equals,        // equals
    Gt,            // gt
    Gte,           // gte
    Lt,            // lt
    Lte,           // lte
    LogicalAnd,    // &&
    LogicalOr,     // ||
    Bang,          // !
    Question,      // ?
    Dot,           // .
    Comma,         // ,
    Colon,         // :
    Semicolon,     // ;
    Arrow,         // ->
    At,            // @
    Ampersand,     // &
    Dollar,        // $
    Pipe,          // |
    Caret,         // ^
    Tilde,         // ~
    LeftShift,     // <<
    RightShift,    // >>
    LeftParen,     // (
    RightParen,    // )
    LeftBrace,     // {
    RightBrace,    // }
    LeftBracket,   // [
    RightBracket,  // ]
    PipeKeyword,   // pipe
    IoKeyword,     // io
    
    // Special
    Newline,
    Indent,
    Dedent,
    Error,     // For unknown characters
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, column: usize, lexeme: String) -> Self {
        Self {
            token_type,
            lexeme,
            line,
            column,
        }
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
    line: usize,
    column: usize,
    keywords: HashMap<String, TokenType>,
    indent_stack: Vec<usize>,
    token_buffer: Vec<Token>,
    at_line_start: bool,
}

impl Lexer {
    pub fn new(input: String, _file_name: String) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.get(0).copied();
        
        let mut keywords = HashMap::new();
        keywords.insert("let".to_string(), TokenType::Let);
        keywords.insert("fn".to_string(), TokenType::Fn);
        keywords.insert("is".to_string(), TokenType::Is);
        keywords.insert("object".to_string(), TokenType::Object);
        keywords.insert("store".to_string(), TokenType::Store);
        keywords.insert("actor".to_string(), TokenType::Actor);
        keywords.insert("use".to_string(), TokenType::Use);
        keywords.insert("mod".to_string(), TokenType::Mod);
        keywords.insert("if".to_string(), TokenType::If);
        keywords.insert("then".to_string(), TokenType::Then);
        keywords.insert("else".to_string(), TokenType::Else);
        keywords.insert("while".to_string(), TokenType::While);
        keywords.insert("for".to_string(), TokenType::For);
        keywords.insert("in".to_string(), TokenType::In);
        keywords.insert("until".to_string(), TokenType::Until);
        keywords.insert("unless".to_string(), TokenType::Unless);
        keywords.insert("iterate".to_string(), TokenType::Iterate);
        keywords.insert("across".to_string(), TokenType::Across);
        keywords.insert("return".to_string(), TokenType::Return);
        keywords.insert("break".to_string(), TokenType::Break);
        keywords.insert("continue".to_string(), TokenType::Continue);
        keywords.insert("import".to_string(), TokenType::Import);
        keywords.insert("from".to_string(), TokenType::From);
        keywords.insert("to".to_string(), TokenType::To);
        keywords.insert("nocopy".to_string(), TokenType::Nocopy);
        keywords.insert("err".to_string(), TokenType::Err);
        keywords.insert("no".to_string(), TokenType::No);
        keywords.insert("yes".to_string(), TokenType::Yes);
        keywords.insert("empty".to_string(), TokenType::Empty);
        keywords.insert("now".to_string(), TokenType::Now);
        keywords.insert("true".to_string(), TokenType::True);
        keywords.insert("false".to_string(), TokenType::False);
        keywords.insert("and".to_string(), TokenType::And);
        keywords.insert("or", TokenType::Or);
        keywords.insert("equals", TokenType::Equals);
        keywords.insert("gt", TokenType::Gt);
        keywords.insert("gte", TokenType::Gte);
        keywords.insert("lt", TokenType::Lt);
        keywords.insert("lte", TokenType::Lte);
        keywords.insert("pipe", TokenType::PipeKeyword);
        keywords.insert("io".to_string(), TokenType::IoKeyword);
        
        Self {
            input: chars,
            position: 0,
            current_char,
            line: 1,
            column: 1,
            keywords,
            indent_stack: vec![0],
            token_buffer: Vec::new(),
            at_line_start: true,
        }
    }
    
    fn advance(&mut self) {
        if self.current_char == Some('\n') {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }
    
    fn peek(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn read_string(&mut self, quote_char: char) -> String {
        let mut value = String::new();
        self.advance(); // Skip opening quote
        
        while let Some(ch) = self.current_char {
            if ch == quote_char {
                self.advance(); // Skip closing quote
                break;
            } else if ch == '\\' {
                self.advance();
                match self.current_char {
                    Some('n') => value.push('\n'),
                    Some('t') => value.push('\t'),
                    Some('r') => value.push('\r'),
                    Some('\\') => value.push('\\'),
                    Some('"') => value.push('"'),
                    Some('\'') => value.push('\''),
                    Some(c) => {
                        value.push('\\');
                        value.push(c);
                    }
                    None => value.push('\\'),
                }
            } else {
                value.push(ch);
            }
            self.advance();
        }
        
        value
    }
    
    fn read_number(&mut self) -> (TokenType, String) {
        let mut value = String::new();
        let mut is_float = false;
        
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                value.push(ch);
                self.advance();
            } else if ch == '.' && !is_float {
                if let Some(next) = self.peek() {
                    if next.is_ascii_digit() {
                        is_float = true;
                        value.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        if is_float {
            (TokenType::Float, value)
        } else {
            (TokenType::Integer, value)
        }
    }
    
    fn read_hex_number(&mut self) -> (TokenType, String) {
        let mut value = String::new();
        self.advance(); // Skip '0'
        self.advance(); // Skip 'x'
        
        while let Some(ch) = self.current_char {
            if ch.is_ascii_hexdigit() {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        (TokenType::Integer, format!("0x{}", value))
    }
    
    fn read_binary_number(&mut self) -> (TokenType, String) {
        let mut value = String::new();
        self.advance(); // Skip 'b'
        
        while let Some(ch) = self.current_char {
            if ch == '0' || ch == '1' {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        (TokenType::Integer, format!("b{}", value))
    }
    
    fn read_identifier(&mut self) -> String {
        let mut value = String::new();
        
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        value
    }
    
    fn handle_indentation(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut indent_level = 0;
        
        // Count indentation
        while let Some(ch) = self.current_char {
            if ch == ' ' {
                indent_level += 1;
                self.advance();
            } else if ch == '\t' {
                indent_level += 4; // Treat tab as 4 spaces
                self.advance();
            } else {
                break;
            }
        }
        
        let current_indent = *self.indent_stack.last().unwrap();
        
        if indent_level > current_indent {
            // Increased indentation
            self.indent_stack.push(indent_level);
            tokens.push(Token::new(
                TokenType::Indent,
                self.line,
                self.column,
                " ".repeat(indent_level),
            ));
        } else if indent_level < current_indent {
            // Decreased indentation
            while let Some(&stack_indent) = self.indent_stack.last() {
                if stack_indent <= indent_level {
                    break;
                }
                self.indent_stack.pop();
                tokens.push(Token::new(
                    TokenType::Dedent,
                    self.line,
                    self.column,
                    "".to_string(),
                ));
            }
        }
        
        tokens
    }
    
    /// Pop a token from the buffer if available
    fn pop_buffered_token(&mut self) -> Option<Token> {
        if self.token_buffer.is_empty() {
            None
        } else {
            Some(self.token_buffer.remove(0))
        }
    }
    
    /// Process indentation at the start of a line
    fn process_line_indentation(&mut self) {
        let indent_tokens = self.handle_indentation();
        if !indent_tokens.is_empty() {
            self.token_buffer.extend(indent_tokens);
        }
        self.at_line_start = false;
    }
    
    /// Generate dedent tokens at EOF for all remaining indentation levels
    fn generate_eof_dedents(&mut self) -> Option<Token> {
        if self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            Some(Token::new(TokenType::Dedent, self.line, self.column, "".to_string()))
        } else {
            None
        }
    }
    
    pub fn next_token(&mut self) -> Token {
        // Return buffered tokens first (from indentation handling)
        if let Some(token) = self.pop_buffered_token() {
            return token;
        }
        
        loop {
            // Handle indentation at the start of a line
            if self.at_line_start && self.current_char.is_some() {
                self.process_line_indentation();
                if let Some(token) = self.pop_buffered_token() {
                    return token;
                }
            }
            
            match self.current_char {
                None => {
                    // At EOF, generate dedent tokens for all remaining indentation levels
                    if let Some(dedent_token) = self.generate_eof_dedents() {
                        return dedent_token;
                    }
                    return Token::new(TokenType::Eof, self.line, self.column, "".to_string());
                }
                
                Some('\n') => {
                    let token = Token::new(TokenType::Newline, self.line, self.column, "\n".to_string());
                    self.advance();
                    self.at_line_start = true;
                    return token;
                }
                
                Some(' ') | Some('\t') | Some('\r') => {
                    self.skip_whitespace();
                    continue;
                }
                
                Some('\'') => {
                    let value = self.read_string('\'');
                    return Token::new(TokenType::InterpolatedString, self.line, self.column, value);
                }
                
                Some('"') => {
                    let value = self.read_string('"');
                    // Check if it contains interpolation patterns
                    if value.contains('{') && value.contains('}') {
                        return Token::new(TokenType::InterpolatedString, self.line, self.column, value);
                    } else {
                        return Token::new(TokenType::String, self.line, self.column, value);
                    }
                }
                
                Some('0') if self.peek() == Some('x') => {
                    let (token_type, lexeme) = self.read_hex_number();
                    return Token::new(token_type, self.line, self.column, lexeme);
                }
                
                Some('b') if self.peek().map_or(false, |c| c == '0' || c == '1') => {
                    let (token_type, lexeme) = self.read_binary_number();
                    return Token::new(token_type, self.line, self.column, lexeme);
                }
                
                Some(ch) if ch.is_ascii_digit() => {
                    let (token_type, lexeme) = self.read_number();
                    return Token::new(token_type, self.line, self.column, lexeme);
                }
                
                Some(ch) if ch.is_alphabetic() || ch == '_' => {
                    let identifier = self.read_identifier();
                    let token_type = self.keywords.get(&identifier)
                        .cloned()
                        .unwrap_or(TokenType::Identifier);
                    return Token::new(token_type, self.line, self.column, identifier);
                }
                
                Some('?') => {
                    self.advance();
                    return Token::new(TokenType::Question, self.line, self.column, "?".to_string());
                }
                
                Some('!') => {
                    if self.peek() == Some('=') {
                        self.advance(); // consume '!'
                        self.advance(); // consume '='
                        return Token::new(TokenType::BangEqual, self.line, self.column, "!=".to_string());
                    } else {
                        self.advance();
                        return Token::new(TokenType::Bang, self.line, self.column, "!".to_string());
                    }
                }
                
                Some('=') => {
                    if self.peek() == Some('=') {
                        self.advance(); // consume '='
                        self.advance(); // consume '='
                        return Token::new(TokenType::EqualEqual, self.line, self.column, "==".to_string());
                    } else {
                        self.advance();
                        return Token::new(TokenType::Equal, self.line, self.column, "=".to_string());
                    }
                }
                
                Some('<') => {
                    if self.peek() == Some('=') {
                        self.advance(); // consume '<'
                        self.advance(); // consume '='
                        return Token::new(TokenType::LessEqual, self.line, self.column, "<=".to_string());
                    } else if self.peek() == Some('<') {
                        self.advance(); // consume '<'
                        self.advance(); // consume '<'
                        return Token::new(TokenType::LeftShift, self.line, self.column, "<<".to_string());
                    } else {
                        self.advance();
                        return Token::new(TokenType::Less, self.line, self.column, "<".to_string());
                    }
                }
                
                Some('>') => {
                    if self.peek() == Some('=') {
                        self.advance(); // consume '>'
                        self.advance(); // consume '='
                        return Token::new(TokenType::GreaterEqual, self.line, self.column, ">=".to_string());
                    } else if self.peek() == Some('>') {
                        self.advance(); // consume '>'
                        self.advance(); // consume '>'
                        return Token::new(TokenType::RightShift, self.line, self.column, ">>".to_string());
                    } else {
                        self.advance();
                        return Token::new(TokenType::Greater, self.line, self.column, ">".to_string());
                    }
                }
                
                Some('-') => {
                    if self.peek() == Some('>') {
                        self.advance(); // consume '-'
                        self.advance(); // consume '>'
                        return Token::new(TokenType::Arrow, self.line, self.column, "->".to_string());
                    } else {
                        self.advance();
                        return Token::new(TokenType::Minus, self.line, self.column, "-".to_string());
                    }
                }
                
                Some('.') => {
                    self.advance();
                    return Token::new(TokenType::Dot, self.line, self.column, ".".to_string());
                }
                
                Some(',') => {
                    self.advance();
                    return Token::new(TokenType::Comma, self.line, self.column, ",".to_string());
                }
                
                Some(':') => {
                    self.advance();
                    return Token::new(TokenType::Colon, self.line, self.column, ":".to_string());
                }
                
                Some(';') => {
                    self.advance();
                    return Token::new(TokenType::Semicolon, self.line, self.column, ";".to_string());
                }
                
                Some('@') => {
                    self.advance();
                    return Token::new(TokenType::At, self.line, self.column, "@".to_string());
                }
                
                Some('&') => {
                    self.advance();
                    if self.current_char == Some('&') {
                        self.advance();
                        return Token::new(TokenType::LogicalAnd, self.line, self.column, "&&".to_string());
                    }
                    return Token::new(TokenType::Ampersand, self.line, self.column, "&".to_string());
                }
                
                Some('|') => {
                    self.advance();
                    if self.current_char == Some('|') {
                        self.advance();
                        return Token::new(TokenType::LogicalOr, self.line, self.column, "||".to_string());
                    }
                    return Token::new(TokenType::Pipe, self.line, self.column, "|".to_string());
                }
                
                Some('^') => {
                    self.advance();
                    return Token::new(TokenType::Caret, self.line, self.column, "^".to_string());
                }
                
                Some('~') => {
                    self.advance();
                    return Token::new(TokenType::Tilde, self.line, self.column, "~".to_string());
                }
                
                Some('$') => {
                    self.advance();
                    return Token::new(TokenType::Dollar, self.line, self.column, "$".to_string());
                }
                
                Some('+') => {
                    self.advance();
                    return Token::new(TokenType::Plus, self.line, self.column, "+".to_string());
                }
                
                Some('*') => {
                    self.advance();
                    return Token::new(TokenType::Star, self.line, self.column, "*".to_string());
                }
                
                Some('/') => {
                    self.advance();
                    return Token::new(TokenType::Slash, self.line, self.column, "/".to_string());
                }
                
                Some('%') => {
                    self.advance();
                    return Token::new(TokenType::Percent, self.line, self.column, "%".to_string());
                }
                
                Some('(') => {
                    self.advance();
                    return Token::new(TokenType::LeftParen, self.line, self.column, "(".to_string());
                }
                
                Some(')') => {
                    self.advance();
                    return Token::new(TokenType::RightParen, self.line, self.column, ")".to_string());
                }
                
                Some('{') => {
                    self.advance();
                    return Token::new(TokenType::LeftBrace, self.line, self.column, "{".to_string());
                }
                
                Some('}') => {
                    self.advance();
                    return Token::new(TokenType::RightBrace, self.line, self.column, "}".to_string());
                }
                
                Some('[') => {
                    self.advance();
                    return Token::new(TokenType::LeftBracket, self.line, self.column, "[".to_string());
                }
                
                Some(']') => {
                    self.advance();
                    return Token::new(TokenType::RightBracket, self.line, self.column, "]".to_string());
                }
                
                Some(ch) => {
                    // Return error token for unknown characters instead of skipping
                    let error_ch = ch.to_string();
                    self.advance();
                    return Token::new(TokenType::Error, self.line, self.column, error_ch);
                }
            }
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token();
            let is_eof = matches!(token.token_type, TokenType::Eof);
            tokens.push(token);
            
            if is_eof {
                break;
            }
        }
        
        Ok(tokens)
    }

}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::Integer => write!(f, "Integer"),
            TokenType::Float => write!(f, "Float"),
            TokenType::String => write!(f, "String"),
            TokenType::InterpolatedString => write!(f, "InterpolatedString"),
            TokenType::True => write!(f, "True"),
            TokenType::False => write!(f, "False"),
            TokenType::Identifier => write!(f, "Identifier"),
            TokenType::Let => write!(f, "Let"),
            TokenType::Fn => write!(f, "Fn"),
            TokenType::Is => write!(f, "Is"),
            TokenType::Object => write!(f, "Object"),
            TokenType::Store => write!(f, "Store"),
            TokenType::Actor => write!(f, "Actor"),
            TokenType::Use => write!(f, "Use"),
            TokenType::Mod => write!(f, "Mod"),
            TokenType::If => write!(f, "if"),
            TokenType::Then => write!(f, "then"),
            TokenType::Else => write!(f, "else"),
            TokenType::While => write!(f, "While"),
            TokenType::For => write!(f, "For"),
            TokenType::In => write!(f, "In"),
            TokenType::Until => write!(f, "Until"),
            TokenType::Unless => write!(f, "Unless"),
            TokenType::Iterate => write!(f, "Iterate"),
            TokenType::Across => write!(f, "Across"),
            TokenType::Return => write!(f, "Return"),
            TokenType::Break => write!(f, "Break"),
            TokenType::Continue => write!(f, "Continue"),
            TokenType::Import => write!(f, "Import"),
            TokenType::Err => write!(f, "Err"),
            TokenType::No => write!(f, "No"),
            TokenType::Yes => write!(f, "Yes"),
            TokenType::Empty => write!(f, "Empty"),
            TokenType::Now => write!(f, "Now"),
            TokenType::Plus => write!(f, "+"),
            TokenType::Minus => write!(f, "-"),
            TokenType::Star => write!(f, "*"),
            TokenType::Slash => write!(f, "/"),
            TokenType::Percent => write!(f, "%"),
            TokenType::Equal => write!(f, "="),
            TokenType::EqualEqual => write!(f, "=="),
            TokenType::BangEqual => write!(f, "!="),
            TokenType::Less => write!(f, "<"),
            TokenType::LessEqual => write!(f, "<="),
            TokenType::Greater => write!(f, ">"),
            TokenType::GreaterEqual => write!(f, ">="),
            TokenType::And => write!(f, "and"),
            TokenType::Or => write!(f, "or"),
            TokenType::LogicalAnd => write!(f, "&&"),
            TokenType::LogicalOr => write!(f, "||"),
            TokenType::Bang => write!(f, "!"),
            TokenType::Question => write!(f, "?"),
            TokenType::Dot => write!(f, "."),
            TokenType::Comma => write!(f, ","),
            TokenType::Colon => write!(f, ":"),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::Arrow => write!(f, "->"),
            TokenType::At => write!(f, "@"),
            TokenType::Ampersand => write!(f, "&"),
            TokenType::Dollar => write!(f, "$"),
            TokenType::Pipe => write!(f, "|"),
            TokenType::Caret => write!(f, "^"),
            TokenType::Tilde => write!(f, "~"),
            TokenType::LeftShift => write!(f, "<<"),
            TokenType::RightShift => write!(f, ">>"),
            TokenType::LeftParen => write!(f, "("),
            TokenType::RightParen => write!(f, ")"),
            TokenType::LeftBrace => write!(f, "{{"),
            TokenType::RightBrace => write!(f, "}}"),
            TokenType::LeftBracket => write!(f, "["),
            TokenType::RightBracket => write!(f, "]"),
            TokenType::Newline => write!(f, "Newline"),
            TokenType::Indent => write!(f, "Indent"),
            TokenType::Dedent => write!(f, "Dedent"),
            TokenType::Error => write!(f, "Error"),
            TokenType::Eof => write!(f, "EOF"),
            TokenType::PipeKeyword => write!(f, "pipe"),
            TokenType::IoKeyword => write!(f, "io"),
            TokenType::From => write!(f, "from"),
            TokenType::To => write!(f, "to"),
            TokenType::Nocopy => write!(f, "nocopy"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_tokens() {
        let mut lexer = Lexer::new("let x = 42".to_string(), "test.cor".to_string());
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Let);
        assert_eq!(tokens[1].token_type, TokenType::Identifier);
        assert_eq!(tokens[1].lexeme, "x");
        assert_eq!(tokens[2].token_type, TokenType::Equal);
        assert_eq!(tokens[3].token_type, TokenType::Integer);
        assert_eq!(tokens[3].lexeme, "42");
    }
    
    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 3.14 0xFF".to_string(), "test.cor".to_string());
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Integer);
        assert_eq!(tokens[0].lexeme, "42");
        assert_eq!(tokens[1].token_type, TokenType::Float);
        assert_eq!(tokens[1].lexeme, "3.14");
        assert_eq!(tokens[2].token_type, TokenType::Integer);
        assert_eq!(tokens[2].lexeme, "0xFF");
    }
    
    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("== != <= >= -> <<".to_string(), "test.cor".to_string());
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::EqualEqual);
        assert_eq!(tokens[1].token_type, TokenType::BangEqual);
        assert_eq!(tokens[2].token_type, TokenType::LessEqual);
        assert_eq!(tokens[3].token_type, TokenType::GreaterEqual);
        assert_eq!(tokens[4].token_type, TokenType::Arrow);
        assert_eq!(tokens[5].token_type, TokenType::LeftShift);
    }
}
