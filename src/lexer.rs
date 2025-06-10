use crate::token::{Token, TokenType};
use std::collections::HashMap;

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    read_position: usize,
    ch: char,
    line: usize,
    col: usize,
    start_byte: usize,
    keywords: HashMap<String, TokenType>,
    // Indentation tracking
    indent_stack: Vec<usize>,
    at_line_start: bool,
    pending_dedents: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let mut lexer = Lexer {
            input: chars,
            position: 0,
            read_position: 0,
            ch: '\0',
            line: 1,
            col: 0,
            start_byte: 0,
            keywords: Self::build_keywords(),
            indent_stack: vec![0], // Start with zero indentation
            at_line_start: true,
            pending_dedents: 0,
        };
        lexer.read_char();
        lexer
    }

    fn build_keywords() -> HashMap<String, TokenType> {
        let mut keywords = HashMap::new();
        
        // Core language keywords
        keywords.insert("fn".to_string(), TokenType::Fn);
        keywords.insert("object".to_string(), TokenType::Object);
        keywords.insert("store".to_string(), TokenType::Store);
        keywords.insert("actor".to_string(), TokenType::Actor);
        keywords.insert("is".to_string(), TokenType::Is);
        keywords.insert("as".to_string(), TokenType::As);
        keywords.insert("with".to_string(), TokenType::With);
        keywords.insert("use".to_string(), TokenType::Use);
        keywords.insert("make".to_string(), TokenType::Make);
        keywords.insert("return".to_string(), TokenType::Return);
        
        // Control flow
        keywords.insert("if".to_string(), TokenType::If);
        keywords.insert("else".to_string(), TokenType::Else);
        keywords.insert("unless".to_string(), TokenType::Unless);
        keywords.insert("while".to_string(), TokenType::While);
        keywords.insert("until".to_string(), TokenType::Until);
        keywords.insert("for".to_string(), TokenType::For);
        keywords.insert("in".to_string(), TokenType::In);
        keywords.insert("across".to_string(), TokenType::Across);
        keywords.insert("into".to_string(), TokenType::Into);
        keywords.insert("iterate".to_string(), TokenType::Iterate);
        
        // Comparison operators (Coral-style)
        keywords.insert("gt".to_string(), TokenType::Gt);
        keywords.insert("lt".to_string(), TokenType::Lt);
        keywords.insert("equals".to_string(), TokenType::Equals);
        keywords.insert("gte".to_string(), TokenType::Gte);
        keywords.insert("lte".to_string(), TokenType::Lte);
        
        // Boolean values
        keywords.insert("yes".to_string(), TokenType::Boolean(true));
        keywords.insert("no".to_string(), TokenType::Boolean(false));
        keywords.insert("true".to_string(), TokenType::Boolean(true));
        keywords.insert("false".to_string(), TokenType::Boolean(false));
        
        // Built-in functions and literals
        keywords.insert("log".to_string(), TokenType::Log);
        keywords.insert("empty".to_string(), TokenType::Empty);
        keywords.insert("now".to_string(), TokenType::Now);
        keywords.insert("at".to_string(), TokenType::At);
        keywords.insert("on".to_string(), TokenType::On);
        keywords.insert("err".to_string(), TokenType::Err);
        
        // Method chaining and logical operators
        keywords.insert("then".to_string(), TokenType::Then);
        keywords.insert("and".to_string(), TokenType::And);
        keywords.insert("or".to_string(), TokenType::Or);
        
        // Collection operations
        keywords.insert("push".to_string(), TokenType::Push);
        // "pop" is now treated as a regular identifier for method calls like object.pop
        
        keywords
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0';
        } else {
            self.ch = self.input[self.read_position];
        }
        self.position = self.read_position;
        self.read_position += 1;
        
        if self.ch == '\n' {
            self.line += 1;
            self.col = 0;
            self.at_line_start = true;
        } else {
            self.col += 1;
        }
    }

    fn peek_char(&self) -> char {
        if self.read_position >= self.input.len() {
            '\0'
        } else {
            self.input[self.read_position]
        }
    }

    fn peek_next_char(&self) -> char {
        if self.read_position + 1 >= self.input.len() {
            '\0'
        } else {
            self.input[self.read_position + 1]
        }
    }

    pub fn next_token(&mut self) -> Token {
        // Handle pending dedents first
        if self.pending_dedents > 0 {
            self.pending_dedents -= 1;
            return Token::new(
                TokenType::Dedent, 
                "".to_string(), 
                self.line, 
                self.col, 
                self.start_byte, 
                self.position
            );
        }

        // Handle indentation at line start
        if self.at_line_start {
            if self.ch == ' ' || self.ch == '\t' {
                return self.handle_indentation();
            } else if self.ch != '\n' && self.ch != '#' && self.ch != '\0' {
                // Non-whitespace at start of line - check for dedent
                self.at_line_start = false;
                let current_indent = *self.indent_stack.last().unwrap();
                if current_indent > 0 {
                    // We're at zero indentation but stack has indentation levels
                    let mut dedent_count = 0;
                    while let Some(&stack_indent) = self.indent_stack.last() {
                        if stack_indent == 0 {
                            break;
                        }
                        self.indent_stack.pop();
                        dedent_count += 1;
                    }
                    
                    if dedent_count > 1 {
                        self.pending_dedents = dedent_count - 1;
                    }
                    
                    if dedent_count > 0 {
                        return Token::new(TokenType::Dedent, "".to_string(), self.line, self.col, self.start_byte, self.position);
                    }
                }
            }
        }

        loop {
            self.skip_whitespace();
            self.start_byte = self.position;
            let start_line = self.line;
            let start_col = self.col;

            let token = match self.ch {
                '&' => {
                    // Check if this is a join table reference at start of line
                    if self.position == 0 || (self.position > 0 && self.input[self.position - 1] == '\n') {
                        Token::new(TokenType::AmpRef, "&".to_string(), self.line, self.col, self.position, self.position + 1)
                    } else {
                        Token::new(TokenType::Amp, "&".to_string(), self.line, self.col, self.position, self.position + 1)
                    }
                }
                '\0' => {
                    // Handle any remaining dedents at EOF
                    if self.indent_stack.len() > 1 {
                        self.indent_stack.pop();
                        self.pending_dedents = self.indent_stack.len().saturating_sub(1);
                        if self.pending_dedents > 0 {
                            self.pending_dedents -= 1;
                            return Token::new(TokenType::Dedent, "".to_string(), start_line, start_col, self.start_byte, self.position);
                        }
                    }
                    Token::eof(start_line, start_col, self.start_byte)
                }
                '\n' => {
                    let token = Token::new(TokenType::Newline, "\n".to_string(), start_line, start_col, self.start_byte, self.position + 1);
                    self.read_char();
                    token
                }
                '#' => {
                    // Skip comments entirely
                    if self.peek_char() == '#' {
                        self.read_char(); // consume second #
                        self.read_char(); // move past ##
                        self.read_doc_comment();
                    } else {
                        self.read_char(); // consume #
                        self.read_line_comment();
                    }
                    continue; // Skip comments, get next token
                }
                '\'' => self.read_string_literal('\''),
                '"' => self.read_string_literal('"'),
                '=' => {
                    if self.peek_char() == '=' {
                        let lexeme = "==".to_string();
                        self.read_char();
                        self.read_char();
                        Token::new(TokenType::EqEq, lexeme, start_line, start_col, self.start_byte, self.position)
                    } else {
                        let lexeme = "=".to_string();
                        self.read_char();
                        Token::new(TokenType::Eq, lexeme, start_line, start_col, self.start_byte, self.position)
                    }
                }
                '!' => {
                    if self.peek_char() == '=' {
                        let lexeme = "!=".to_string();
                        self.read_char();
                        self.read_char();
                        Token::new(TokenType::BangEq, lexeme, start_line, start_col, self.start_byte, self.position)
                    } else {
                        let lexeme = "!".to_string();
                        self.read_char();
                        Token::new(TokenType::Bang, lexeme, start_line, start_col, self.start_byte, self.position)
                    }
                }
                '+' => {
                    let lexeme = "+".to_string();
                    self.read_char();
                    Token::new(TokenType::Plus, lexeme, start_line, start_col, self.start_byte, self.position)
                }
                '-' => {
                    let lexeme = "-".to_string();
                    self.read_char();
                    Token::new(TokenType::Minus, lexeme, start_line, start_col, self.start_byte, self.position)
                }
                '*' => {
                    if self.peek_char() == '*' {
                        let lexeme = "**".to_string();
                        self.read_char();
                        self.read_char();
                        Token::new(TokenType::DoubleStar, lexeme, start_line, start_col, self.start_byte, self.position)
                    } else {
                        let lexeme = "*".to_string();
                        self.read_char();
                        Token::new(TokenType::Star, lexeme, start_line, start_col, self.start_byte, self.position)
                    }
                }
                '/' => {
                    let lexeme = "/".to_string();
                    self.read_char();
                    Token::new(TokenType::Slash, lexeme, start_line, start_col, self.start_byte, self.position)
                }
                '%' => {
                    let lexeme = "%".to_string();
                    self.read_char();
                    Token::new(TokenType::Percent, lexeme, start_line, start_col, self.start_byte, self.position)
                }
                '(' => {
                    let lexeme = "(".to_string();
                    self.read_char();
                    Token::new(TokenType::LParen, lexeme, start_line, start_col, self.start_byte, self.position)
                }
                ')' => {
                    let lexeme = ")".to_string();
                    self.read_char();
                    Token::new(TokenType::RParen, lexeme, start_line, start_col, self.start_byte, self.position)
                }
                '[' => {
                    let lexeme = "[".to_string();
                    self.read_char();
                    Token::new(TokenType::LBracket, lexeme, start_line, start_col, self.start_byte, self.position)
                }
                ']' => {
                    let lexeme = "]".to_string();
                    self.read_char();
                    Token::new(TokenType::RBracket, lexeme, start_line, start_col, self.start_byte, self.position)
                }
                '{' => {
                    let lexeme = "{".to_string();
                    self.read_char();
                    Token::new(TokenType::LBrace, lexeme, start_line, start_col, self.start_byte, self.position)
                }
                '}' => {
                    let lexeme = "}".to_string();
                    self.read_char();
                    Token::new(TokenType::RBrace, lexeme, start_line, start_col, self.start_byte, self.position)
                }
                ',' => {
                    let lexeme = ",".to_string();
                    self.read_char();
                    Token::new(TokenType::Comma, lexeme, start_line, start_col, self.start_byte, self.position)
                }
                '.' => {
                    if self.peek_char() == '.' {
                        if self.peek_next_char() == '=' {
                            let lexeme = "..=".to_string();
                            self.read_char();
                            self.read_char();
                            self.read_char();
                            Token::new(TokenType::DotDotEq, lexeme, start_line, start_col, self.start_byte, self.position)
                        } else {
                            let lexeme = "..".to_string();
                            self.read_char();
                            self.read_char();
                            Token::new(TokenType::DotDot, lexeme, start_line, start_col, self.start_byte, self.position)
                        }
                    } else {
                        let lexeme = ".".to_string();
                        self.read_char();
                        Token::new(TokenType::Dot, lexeme, start_line, start_col, self.start_byte, self.position)
                    }
                }
                ':' => {
                    let lexeme = ":".to_string();
                    self.read_char();
                    Token::new(TokenType::Colon, lexeme, start_line, start_col, self.start_byte, self.position)
                }
                '?' => {
                    if self.peek_char() == '.' {
                        let lexeme = "?.".to_string();
                        self.read_char();
                        self.read_char();
                        Token::new(TokenType::QuestionDot, lexeme, start_line, start_col, self.start_byte, self.position)
                    } else if self.peek_char() == '?' {
                        let lexeme = "??".to_string();
                        self.read_char();
                        self.read_char();
                        Token::new(TokenType::DoubleQuestion, lexeme, start_line, start_col, self.start_byte, self.position)
                    } else {
                        let lexeme = "?".to_string();
                        self.read_char();
                        Token::new(TokenType::Question, lexeme, start_line, start_col, self.start_byte, self.position)
                    }
                }
                '@' => {
                    let lexeme = "@".to_string();
                    self.read_char();
                    Token::new(TokenType::AnnotationMarker, lexeme, start_line, start_col, self.start_byte, self.position)
                }
                '&' => {
                    if self.peek_char() == '&' {
                        let lexeme = "&&".to_string();
                        self.read_char();
                        self.read_char();
                        Token::new(TokenType::AmpAmp, lexeme, start_line, start_col, self.start_byte, self.position)
                    } else {
                        let lexeme = "&".to_string();
                        self.read_char();
                        Token::new(TokenType::Amp, lexeme, start_line, start_col, self.start_byte, self.position)
                    }
                }
                '|' => {
                    if self.peek_char() == '|' {
                        let lexeme = "||".to_string();
                        self.read_char();
                        self.read_char();
                        Token::new(TokenType::PipePipe, lexeme, start_line, start_col, self.start_byte, self.position)
                    } else {
                        let lexeme = "|".to_string();
                        self.read_char();
                        Token::new(TokenType::Pipe, lexeme, start_line, start_col, self.start_byte, self.position)
                    }
                }
                '<' => {
                    if self.peek_char() == '=' {
                        let lexeme = "<=".to_string();
                        self.read_char();
                        self.read_char();
                        Token::new(TokenType::LtEq, lexeme, start_line, start_col, self.start_byte, self.position)
                    } else if self.peek_char() == '<' {
                        let lexeme = "<<".to_string();
                        self.read_char();
                        self.read_char();
                        Token::new(TokenType::LtLt, lexeme, start_line, start_col, self.start_byte, self.position)
                    } else {
                        let lexeme = "<".to_string();
                        self.read_char();
                        Token::new(TokenType::Illegal("<".to_string()), lexeme, start_line, start_col, self.start_byte, self.position)
                    }
                }
                '>' => {
                    if self.peek_char() == '=' {
                        let lexeme = ">=".to_string();
                        self.read_char();
                        self.read_char();
                        Token::new(TokenType::GtEq, lexeme, start_line, start_col, self.start_byte, self.position)
                    } else if self.peek_char() == '>' {
                        let lexeme = ">>".to_string();
                        self.read_char();
                        self.read_char();
                        Token::new(TokenType::GtGt, lexeme, start_line, start_col, self.start_byte, self.position)
                    } else {
                        let lexeme = ">".to_string();
                        self.read_char();
                        Token::new(TokenType::Illegal(">".to_string()), lexeme, start_line, start_col, self.start_byte, self.position)
                    }
                }
                '$' => {
                    self.read_char(); // consume $
                    let param_ref = self.read_parameter_reference();
                    if param_ref.is_empty() {
                        // Handle bare $ as placeholder
                        Token::new(TokenType::ParameterRef("$".to_string()), "$".to_string(), start_line, start_col, self.start_byte, self.position)
                    } else {
                        Token::new(TokenType::ParameterRef(param_ref.clone()), format!("${}", param_ref), start_line, start_col, self.start_byte, self.position)
                    }
                }
                _ => {
                    if self.ch.is_alphabetic() || self.ch == '_' {
                        let identifier = self.read_identifier();
                        if let Some(keyword_type) = self.keywords.get(&identifier) {
                            Token::new(keyword_type.clone(), identifier, start_line, start_col, self.start_byte, self.position)
                        } else {
                            Token::new(TokenType::Identifier(identifier.clone()), identifier, start_line, start_col, self.start_byte, self.position)
                        }
                    } else if self.ch.is_ascii_digit() {
                        self.read_number()
                    } else {
                        let lexeme = self.ch.to_string();
                        self.read_char();
                        Token::new(TokenType::Illegal(lexeme.clone()), lexeme, start_line, start_col, self.start_byte, self.position)
                    }
                }
            };

            return token;
        }
    }

    fn skip_whitespace(&mut self) {
        while self.ch.is_whitespace() && self.ch != '\n' {
            self.read_char();
        }
    }

    fn read_identifier(&mut self) -> String {
        let start_pos = self.position;
        while self.ch.is_alphanumeric() || self.ch == '_' {
            self.read_char();
        }
        self.input[start_pos..self.position].iter().collect()
    }

    fn read_parameter_reference(&mut self) -> String {
        let start_pos = self.position;
        if self.ch.is_ascii_digit() {
            // $0, $1, $2, etc.
            while self.ch.is_ascii_digit() {
                self.read_char();
            }
        } else if self.ch.is_alphabetic() || self.ch == '_' {
            // $identifier
            while self.ch.is_alphanumeric() || self.ch == '_' {
                self.read_char();
            }
        }
        // If no characters were consumed, return empty string for bare $
        self.input[start_pos..self.position].iter().collect()
    }

    fn read_number(&mut self) -> Token {
        let start_pos = self.position;
        let start_line = self.line;
        let start_col = self.col;
        let start_byte = self.start_byte;

        while self.ch.is_ascii_digit() {
            self.read_char();
        }

        if self.ch == '.' && self.peek_char().is_ascii_digit() {
            self.read_char(); // consume '.'
            while self.ch.is_ascii_digit() {
                self.read_char();
            }
            let float_str: String = self.input[start_pos..self.position].iter().collect();
            Token::new(TokenType::Float(float_str.clone()), float_str, start_line, start_col, start_byte, self.position)
        } else {
            let int_str: String = self.input[start_pos..self.position].iter().collect();
            Token::new(TokenType::Integer(int_str.clone()), int_str, start_line, start_col, start_byte, self.position)
        }
    }

    fn read_string_literal(&mut self, quote_char: char) -> Token {
        let start_line = self.line;
        let start_col = self.col;
        let start_byte = self.start_byte;
        
        self.read_char(); // consume opening quote
        let mut string_content = String::new();
        let mut has_interpolation = false;

        while self.ch != quote_char && self.ch != '\0' {
            if self.ch == '{' {
                // Check for string interpolation
                let mut brace_count = 1;
                let mut interpolation = String::new();
                self.read_char(); // consume '{'
                
                while brace_count > 0 && self.ch != '\0' {
                    if self.ch == '{' {
                        brace_count += 1;
                    } else if self.ch == '}' {
                        brace_count -= 1;
                    }
                    if brace_count > 0 {
                        interpolation.push(self.ch);
                    }
                    self.read_char();
                }
                
                has_interpolation = true;
                string_content.push_str(&format!("{{{}}}", interpolation));
            } else if self.ch == '\\' {
                self.read_char(); // consume backslash
                match self.ch {
                    'n' => string_content.push('\n'),
                    't' => string_content.push('\t'),
                    'r' => string_content.push('\r'),
                    '\\' => string_content.push('\\'),
                    '\'' => string_content.push('\''),
                    '"' => string_content.push('"'),
                    _ => {
                        string_content.push('\\');
                        string_content.push(self.ch);
                    }
                }
                self.read_char();
            } else {
                string_content.push(self.ch);
                self.read_char();
            }
        }

        if self.ch == quote_char {
            self.read_char(); // consume closing quote
        }

        let token_type = if has_interpolation {
            TokenType::InterpolatedString(string_content.clone())
        } else {
            TokenType::StringLiteral(string_content.clone())
        };

        Token::new(token_type, string_content, start_line, start_col, start_byte, self.position)
    }

    fn read_line_comment(&mut self) -> String {
        let start_pos = self.position;
        while self.ch != '\n' && self.ch != '\0' {
            self.read_char();
        }
        self.input[start_pos..self.position].iter().collect()
    }

    fn read_doc_comment(&mut self) -> String {
        let start_pos = self.position;
        while self.ch != '\n' && self.ch != '\0' {
            self.read_char();
        }
        self.input[start_pos..self.position].iter().collect()
    }

    fn handle_indentation(&mut self) -> Token {
        let start_line = self.line;
        let start_col = self.col;
        let start_byte = self.position;
        
        let mut indent_level = 0;
        
        // Count indentation
        while self.ch == ' ' || self.ch == '\t' {
            if self.ch == ' ' {
                indent_level += 1;
            } else if self.ch == '\t' {
                indent_level += 8; // Treat tab as 8 spaces
            }
            self.read_char();
        }
        
        // Skip empty lines and comments
        if self.ch == '\n' || self.ch == '#' || self.ch == '\0' {
            self.at_line_start = true;
            return self.next_token();
        }
        
        self.at_line_start = false;
        
        let current_indent = *self.indent_stack.last().unwrap();
        
        if indent_level > current_indent {
            // Check if this is a valid indentation increase
            // In Python-style indentation, we need consistent indentation levels
            if self.indent_stack.len() > 1 {
                // Find the smallest indentation step we've seen
                let mut min_step = usize::MAX;
                for i in 1..self.indent_stack.len() {
                    let step = self.indent_stack[i] - self.indent_stack[i-1];
                    if step < min_step {
                        min_step = step;
                    }
                }
                
                // Check if the new indentation follows the pattern
                let expected_indent = current_indent + min_step;
                if indent_level != expected_indent {
                    return Token::new(
                        TokenType::Illegal("Invalid indentation".to_string()),
                        "".to_string(),
                        start_line,
                        start_col,
                        start_byte,
                        self.position
                    );
                }
            }
            
            // Increased indentation - emit INDENT
            self.indent_stack.push(indent_level);
            Token::new(TokenType::Indent, "".to_string(), start_line, start_col, start_byte, self.position)
        } else if indent_level < current_indent {
            // Decreased indentation - emit DEDENT(s)
            let mut dedent_count = 0;
            while let Some(&stack_indent) = self.indent_stack.last() {
                if stack_indent <= indent_level {
                    break;
                }
                self.indent_stack.pop();
                dedent_count += 1;
            }
            
            // Check if we have a matching indentation level
            if let Some(&stack_level) = self.indent_stack.last() {
                if stack_level != indent_level {
                    return Token::new(
                        TokenType::Illegal("Invalid indentation".to_string()),
                        "".to_string(),
                        start_line,
                        start_col,
                        start_byte,
                        self.position
                    );
                }
            }
            
            if dedent_count > 1 {
                self.pending_dedents = dedent_count - 1;
            }
            
            Token::new(TokenType::Dedent, "".to_string(), start_line, start_col, start_byte, self.position)
        } else {
            // Same indentation - continue parsing
            self.next_token()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let input = "fn greet with name is 'hello'";
        let mut lexer = Lexer::new(input.to_string());
        
        let tokens = vec![
            TokenType::Fn,
            TokenType::Identifier("greet".to_string()),
            TokenType::With,
            TokenType::Identifier("name".to_string()),
            TokenType::Is,
            TokenType::StringLiteral("hello".to_string()),
            TokenType::Eof,
        ];

        for expected_type in tokens {
            let token = lexer.next_token();
            assert_eq!(token.kind, expected_type);
        }
    }

    #[test]
    fn test_parameter_reference() {
        let input = "$id $0 $description";
        let mut lexer = Lexer::new(input.to_string());
        
        assert_eq!(lexer.next_token().kind, TokenType::ParameterRef("id".to_string()));
        assert_eq!(lexer.next_token().kind, TokenType::ParameterRef("0".to_string()));
        assert_eq!(lexer.next_token().kind, TokenType::ParameterRef("description".to_string()));
    }

    #[test]
    fn test_string_interpolation() {
        let input = "'Hello {name}, welcome to {place}!'";
        let mut lexer = Lexer::new(input.to_string());
        
        let token = lexer.next_token();
        match token.kind {
            TokenType::InterpolatedString(_) => {},
            _ => panic!("Expected interpolated string"),
        }
    }

    #[test]
    fn test_coral_operators() {
        let input = "gt lt equals @ &";
        let mut lexer = Lexer::new(input.to_string());
        
        assert_eq!(lexer.next_token().kind, TokenType::Gt);
        assert_eq!(lexer.next_token().kind, TokenType::Lt);
        assert_eq!(lexer.next_token().kind, TokenType::Equals);
        assert_eq!(lexer.next_token().kind, TokenType::AnnotationMarker);
        assert_eq!(lexer.next_token().kind, TokenType::Amp);
    }

    #[test]
    fn test_indentation() {
        let input = "fn main\n    greet 'World'\nend";
        let mut lexer = Lexer::new(input.to_string());
        
        // We should get: fn, main, newline, indent, greet, 'World', newline, dedent, end, eof
        assert_eq!(lexer.next_token().kind, TokenType::Fn);
        assert_eq!(lexer.next_token().kind, TokenType::Identifier("main".to_string()));
        assert_eq!(lexer.next_token().kind, TokenType::Newline);
        assert_eq!(lexer.next_token().kind, TokenType::Indent);
        assert_eq!(lexer.next_token().kind, TokenType::Identifier("greet".to_string()));
        assert_eq!(lexer.next_token().kind, TokenType::StringLiteral("World".to_string()));
        assert_eq!(lexer.next_token().kind, TokenType::Newline);
        assert_eq!(lexer.next_token().kind, TokenType::Dedent);
        assert_eq!(lexer.next_token().kind, TokenType::Identifier("end".to_string()));
        assert_eq!(lexer.next_token().kind, TokenType::Eof);
    }

    #[test]
    fn test_invalid_indentation() {
        let input = "fn main\n  greet 'World'\n   bad_indent";  // 3 spaces - invalid
        let mut lexer = Lexer::new(input.to_string());
        
        // Skip to where invalid indentation occurs
        let mut found_error = false;
        for _ in 0..20 {  // Safety limit
            let token = lexer.next_token();
            match &token.kind {
                TokenType::Illegal(msg) if msg.contains("Invalid indentation") => {
                    found_error = true;
                    break;
                }
                TokenType::Eof => break,
                _ => continue,
            }
        }
        
        assert!(found_error, "Expected invalid indentation error");
    }

    #[test]
    fn test_invalid_indentation_debug() {
        let input = "fn main\n  greet 'World'\n   bad_indent";  // 3 spaces - invalid
        let mut lexer = Lexer::new(input.to_string());
        
        let mut tokens = Vec::new();
        for i in 0..25 {  // Safety limit
            let token = lexer.next_token();
            println!("Token {}: {:?} at line {} col {}", i, token.kind, token.line, token.col);
            let is_eof = matches!(token.kind, TokenType::Eof);
            tokens.push(token);
            if is_eof { break; }
        }
        
        // Find any illegal tokens
        let illegal_tokens: Vec<_> = tokens.iter().filter(|t| matches!(t.kind, TokenType::Illegal(_))).collect();
        println!("Found {} illegal tokens", illegal_tokens.len());
        for token in illegal_tokens {
            println!("Illegal token: {:?}", token);
        }
    }

    #[test]
    fn test_indentation_debug() {
        let input = "fn main\n    greet 'World'\nend";
        let mut lexer = Lexer::new(input.to_string());
        
        let mut tokens = Vec::new();
        loop {
            let token = lexer.next_token();
            let is_eof = matches!(token.kind, TokenType::Eof);
            tokens.push(token);
            if is_eof { break; }
        }
        
        for (i, token) in tokens.iter().enumerate() {
            println!("Token {}: {:?}", i, token.kind);
        }
        
        // Let's see what we actually get
        assert!(tokens.len() > 5);
    }
}