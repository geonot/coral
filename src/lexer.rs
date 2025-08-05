use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    // Literals
    Integer,
    Float,
    String,
    InterpolatedString,
    True,
    False,
    Identifier,
    
    // Keywords
    Let, Fn, Is, Object, Store, Actor, Use, Mod, If, Then, Else, While,
    For, In, Until, Unless, Iterate, Across, Return, Break, Continue,
    Import, From, To, Nocopy, Err, No, Yes, Empty, Now, As, With, Into, Make,
    
    // Operators
    Plus, Minus, Star, Slash, Percent, Equal, EqualEqual, BangEqual,
    Less, LessEqual, Greater, GreaterEqual, And, Or, Equals, Gt, Gte,
    Lt, Lte, LogicalAnd, LogicalOr, Bang, Question, Dot, Comma, Colon,
    Semicolon, Arrow, At, Ampersand, Dollar, Pipe, Caret, Tilde,
    LeftShift, RightShift,
    
    // Delimiters
    LeftParen, RightParen, LeftBrace, RightBrace, LeftBracket, RightBracket,
    
    // Special
    Newline,
    Indent,
    Dedent,
    Error,
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
        Self { token_type, lexeme, line, column }
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
    brace_level: usize,
}

impl Lexer {
    pub fn new(input: String, _file_name: String) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.get(0).copied();
        
        let mut keywords = HashMap::new();
        keywords.insert("let".to_string(), TokenType::Let);
        keywords.insert("fn".to_string(), TokenType::Fn);
        keywords.insert("def".to_string(), TokenType::Fn);
        keywords.insert("is".to_string(), TokenType::Is);
        keywords.insert("object".to_string(), TokenType::Object);
        keywords.insert("store".to_string(), TokenType::Store);
        keywords.insert("actor".to_string(), TokenType::Actor);
        keywords.insert("if".to_string(), TokenType::If);
        keywords.insert("else".to_string(), TokenType::Else);
        keywords.insert("while".to_string(), TokenType::While);
        keywords.insert("return".to_string(), TokenType::Return);
        keywords.insert("true".to_string(), TokenType::True);
        keywords.insert("false".to_string(), TokenType::False);
        keywords.insert("and".to_string(), TokenType::And);
        keywords.insert("or".to_string(), TokenType::Or);
        keywords.insert("iterate".to_string(), TokenType::Iterate);
        keywords.insert("as".to_string(), TokenType::As);
        keywords.insert("for".to_string(), TokenType::For);
        keywords.insert("with".to_string(), TokenType::With);
        keywords.insert("into".to_string(), TokenType::Into);
        keywords.insert("make".to_string(), TokenType::Make);
        keywords.insert("equals".to_string(), TokenType::Equals);
        keywords.insert("gt".to_string(), TokenType::Gt);
        keywords.insert("gte".to_string(), TokenType::Gte);
        keywords.insert("lt".to_string(), TokenType::Lt);
        keywords.insert("lte".to_string(), TokenType::Lte);
        keywords.insert("unless".to_string(), TokenType::Unless);
        keywords.insert("until".to_string(), TokenType::Until);
        keywords.insert("across".to_string(), TokenType::Across);
        
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
            brace_level: 0,
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
        self.advance();
        while let Some(ch) = self.current_char {
            if ch == quote_char {
                self.advance();
                break;
            }
            value.push(ch);
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
            } else if ch == '.' && !is_float && self.peek().map_or(false, |c| c.is_ascii_digit()) {
                is_float = true;
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        if is_float { (TokenType::Float, value) } else { (TokenType::Integer, value) }
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
    
    fn handle_indentation(&mut self) {
        if self.brace_level > 0 {
            self.at_line_start = false;
            return;
        }

        let mut indent_level = 0;
        while let Some(ch) = self.current_char {
            if ch == ' ' {
                indent_level += 1;
                self.advance();
            } else if ch == '\t' {
                indent_level += 4;
                self.advance();
            } else {
                break;
            }
        }
        
        let current_indent = *self.indent_stack.last().unwrap();
        
        if indent_level > current_indent {
            self.indent_stack.push(indent_level);
            self.token_buffer.push(Token::new(TokenType::Indent, self.line, 1, " ".repeat(indent_level)));
        } else if indent_level < current_indent {
            while indent_level < *self.indent_stack.last().unwrap() {
                self.indent_stack.pop();
                self.token_buffer.push(Token::new(TokenType::Dedent, self.line, 1, "".to_string()));
            }
        }
        self.at_line_start = false;
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        while self.current_char.is_some() {
            if self.at_line_start {
                self.handle_indentation();
            }
            
            while let Some(buffered) = self.token_buffer.pop() {
                tokens.push(buffered);
            }

            self.skip_whitespace();

            if let Some(ch) = self.current_char {
                let start_line = self.line;
                let start_col = self.column;
                let token = match ch {
                    '\n' => {
                        self.at_line_start = true;
                        let tok = Token::new(TokenType::Newline, start_line, start_col, "\n".to_string());
                        self.advance();
                        tok
                    }
                    '{' => { self.brace_level += 1; self.advance(); Token::new(TokenType::LeftBrace, start_line, start_col, "{".to_string()) }
                    '}' => { if self.brace_level > 0 { self.brace_level -= 1; } self.advance(); Token::new(TokenType::RightBrace, start_line, start_col, "}".to_string()) }
                    '(' => { self.advance(); Token::new(TokenType::LeftParen, start_line, start_col, "(".to_string()) }
                    ')' => { self.advance(); Token::new(TokenType::RightParen, start_line, start_col, ")".to_string()) }
                    '[' => { self.advance(); Token::new(TokenType::LeftBracket, start_line, start_col, "[".to_string()) }
                    ']' => { self.advance(); Token::new(TokenType::RightBracket, start_line, start_col, "]".to_string()) }
                    ',' => { self.advance(); Token::new(TokenType::Comma, start_line, start_col, ",".to_string()) }
                    '.' => { self.advance(); Token::new(TokenType::Dot, start_line, start_col, ".".to_string()) }
                    ':' => { self.advance(); Token::new(TokenType::Colon, start_line, start_col, ":".to_string()) }
                    ';' => { self.advance(); Token::new(TokenType::Semicolon, start_line, start_col, ";".to_string()) }
                    '+' => { self.advance(); Token::new(TokenType::Plus, start_line, start_col, "+".to_string()) }
                    '-' => {
                        self.advance();
                        if self.current_char == Some('>') {
                            self.advance();
                            Token::new(TokenType::Arrow, start_line, start_col, "->".to_string())
                        } else {
                            Token::new(TokenType::Minus, start_line, start_col, "-".to_string())
                        }
                    }
                    '*' => { self.advance(); Token::new(TokenType::Star, start_line, start_col, "*".to_string()) }
                    '/' => { self.advance(); Token::new(TokenType::Slash, start_line, start_col, "/".to_string()) }
                    '=' => {
                        self.advance();
                        if self.current_char == Some('=') {
                            self.advance();
                            Token::new(TokenType::EqualEqual, start_line, start_col, "==".to_string())
                        } else {
                            Token::new(TokenType::Equal, start_line, start_col, "=".to_string())
                        }
                    }
                    '!' => {
                        self.advance();
                        if self.current_char == Some('=') {
                            self.advance();
                            Token::new(TokenType::BangEqual, start_line, start_col, "!=".to_string())
                        } else {
                            Token::new(TokenType::Bang, start_line, start_col, "!".to_string())
                        }
                    }
                    '<' => {
                        self.advance();
                        if self.current_char == Some('=') {
                            self.advance();
                            Token::new(TokenType::LessEqual, start_line, start_col, "<=".to_string())
                        } else {
                            Token::new(TokenType::Less, start_line, start_col, "<".to_string())
                        }
                    }
                    '>' => {
                        self.advance();
                        if self.current_char == Some('=') {
                            self.advance();
                            Token::new(TokenType::GreaterEqual, start_line, start_col, ">=".to_string())
                        } else {
                            Token::new(TokenType::Greater, start_line, start_col, ">".to_string())
                        }
                    }
                    '?' => { self.advance(); Token::new(TokenType::Question, start_line, start_col, "?".to_string()) }
                    '$' => { self.advance(); Token::new(TokenType::Dollar, start_line, start_col, "$".to_string()) }
                    '"' | '\'' => {
                        let content = self.read_string(ch);
                        let tt = if ch == '\'' { TokenType::InterpolatedString } else { TokenType::String };
                        Token::new(tt, start_line, start_col, content)
                    }
                    _ if ch.is_ascii_digit() => {
                        let (tt, lexeme) = self.read_number();
                        Token::new(tt, start_line, start_col, lexeme)
                    }
                    _ if ch.is_alphabetic() || ch == '_' => {
                        let lexeme = self.read_identifier();
                        let tt = self.keywords.get(&lexeme).copied().unwrap_or(TokenType::Identifier);
                        Token::new(tt, start_line, start_col, lexeme)
                    }
                    _ => {
                        let lexeme = ch.to_string();
                        self.advance();
                        Token::new(TokenType::Error, start_line, start_col, lexeme)
                    }
                };
                tokens.push(token);
            }
        }

        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            tokens.push(Token::new(TokenType::Dedent, self.line, 1, "".to_string()));
        }
        tokens.push(Token::new(TokenType::Eof, self.line, 1, "".to_string()));
        Ok(tokens)
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}