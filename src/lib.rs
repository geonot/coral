pub mod ast;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod resolver;
pub mod codegen;

pub use ast::*;
pub use lexer::*;
pub use parser::*;
pub use semantic::*;
pub use resolver::*;
pub use codegen::*;

/// Parse and analyze Coral source code
pub fn parse_and_analyze(input: &str) -> Result<(Program, Result<(), Vec<SemanticError>>), ParseError> {
    let mut lexer = Lexer::new(input.to_string(), "test.cor".to_string());
    let tokens = lexer.tokenize().map_err(|e| ParseError::InvalidSyntax { message: format!("Lexer error: {}", e), span: SourceSpan::default() })?;
    let mut parser = Parser::new(tokens, "test.cor".to_string());
    
    // Use the new integrated parse_and_resolve method
    let mut program = parser.parse_and_resolve()?;
    
    let mut analyzer = SemanticAnalyzer::new();
    let analysis_result = analyzer.analyze(&mut program);
    
    Ok((program, analysis_result))
}
