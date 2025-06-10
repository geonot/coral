use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use tokio::sync::RwLock;
use std::collections::HashMap;
use ropey::Rope;

use coral::{parse_coral, lexer::Lexer, token::TokenType};

pub struct CoralLanguageServer {
    client: Client,
    documents: RwLock<HashMap<Url, Rope>>,
}

impl CoralLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: RwLock::new(HashMap::new()),
        }
    }

    async fn get_diagnostics(&self, _uri: &Url, text: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        
        match parse_coral(text) {
            Ok(_) => {
                // No syntax errors
            }
            Err(errors) => {
                for error in errors {
                    let range = Range {
                        start: Position {
                            line: error.line.saturating_sub(1) as u32,
                            character: error.col as u32,
                        },
                        end: Position {
                            line: error.line.saturating_sub(1) as u32,
                            character: (error.col + error.length.unwrap_or(1)) as u32,
                        },
                    };

                    diagnostics.push(Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: None,
                        code_description: None,
                        source: Some("coral".to_string()),
                        message: error.message.clone(),
                        related_information: None,
                        tags: None,
                        data: None,
                    });
                }
            }
        }

        diagnostics
    }

    async fn get_semantic_tokens(&self, text: &str) -> Vec<SemanticToken> {
        let mut tokens = Vec::new();
        let mut lexer = Lexer::new(text.to_string());
        let mut prev_line = 0;
        let mut prev_char = 0;

        loop {
            let token = lexer.next_token();
            
            if matches!(token.kind, TokenType::Eof) {
                break;
            }

            let (semantic_type, modifiers) = match &token.kind {
                TokenType::Is | TokenType::Fn | 
                TokenType::Object | TokenType::Store | 
                TokenType::With | TokenType::If | 
                TokenType::Else | TokenType::For | 
                TokenType::In | TokenType::While | 
                TokenType::Return | TokenType::Break |
                TokenType::Continue | TokenType::Unless |
                TokenType::Until => (0, 0), // KEYWORD
                
                TokenType::Boolean(_) => (0, 0), // KEYWORD for boolean literals
                
                TokenType::ParameterRef(_) => (6, 0), // PARAMETER
                
                TokenType::And | TokenType::Or | 
                TokenType::Bang | TokenType::Plus | 
                TokenType::Minus | TokenType::Star | 
                TokenType::Slash | TokenType::Percent |
                TokenType::Gt | TokenType::Lt | 
                TokenType::Equals | TokenType::Gte | 
                TokenType::Lte | TokenType::At |
                TokenType::Question => (5, 0), // OPERATOR
                
                TokenType::Identifier(_) => (3, 0), // VARIABLE
                
                TokenType::StringLiteral(_) | 
                TokenType::InterpolatedString(_) => (1, 0), // STRING
                
                TokenType::Integer(_) | 
                TokenType::Float(_) => (2, 0), // NUMBER
                
                _ => continue,
            };

            // Fix delta calculation - line and character are 1-based in tokens
            let token_line = token.line.saturating_sub(1);
            let token_char = token.col.saturating_sub(1);
            
            let delta_line = token_line.saturating_sub(prev_line) as u32;
            let delta_char = if delta_line > 0 {
                token_char as u32
            } else {
                token_char.saturating_sub(prev_char) as u32
            };

            tokens.push(SemanticToken {
                delta_line,
                delta_start: delta_char,
                length: token.lexeme.len() as u32,
                token_type: semantic_type,
                token_modifiers_bitset: modifiers,
            });

            prev_line = token_line;
            prev_char = token_char;
        }

        tokens
    }

    async fn get_completions(&self, _uri: &Url, _position: Position) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "fn".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Function definition".to_string()),
                documentation: Some(Documentation::String("Define a function with parameters".to_string())),
                insert_text: Some("fn ${1:name} with ${2:params}\n    ${0}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "object".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Object definition".to_string()),
                documentation: Some(Documentation::String("Define an object type".to_string())),
                insert_text: Some("object ${1:name}\n    ${0}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "store".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Store definition".to_string()),
                documentation: Some(Documentation::String("Define a data store".to_string())),
                insert_text: Some("store ${1:name}\n    ${0}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "if".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("If statement".to_string()),
                documentation: Some(Documentation::String("Conditional execution".to_string())),
                insert_text: Some("if ${1:condition}\n    ${0}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "for".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("For loop".to_string()),
                documentation: Some(Documentation::String("Iterate over a collection".to_string())),
                insert_text: Some("for ${1:item} in ${2:collection}\n    ${0}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "while".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("While loop".to_string()),
                documentation: Some(Documentation::String("Loop while condition is true".to_string())),
                insert_text: Some("while ${1:condition}\n    ${0}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "log".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Log output".to_string()),
                documentation: Some(Documentation::String("Output a value to the log".to_string())),
                insert_text: Some("log ${0}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "is".to_string(),
                kind: Some(CompletionItemKind::OPERATOR),
                detail: Some("Assignment operator".to_string()),
                documentation: Some(Documentation::String("Assign a value to a variable".to_string())),
                insert_text: Some("is ${0}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "make".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Create instance".to_string()),
                documentation: Some(Documentation::String("Create an instance of an object or store".to_string())),
                insert_text: Some("make ${0}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ]
    }

    async fn get_hover_info(&self, _uri: &Url, position: Position, text: &str) -> Option<Hover> {
        let lines: Vec<&str> = text.lines().collect();
        if let Some(line) = lines.get(position.line as usize) {
            let chars: Vec<char> = line.chars().collect();
            let pos = position.character as usize;
            
            if pos > chars.len() {
                return None;
            }
            
            // Find word boundaries - handle edge cases better
            let mut start = pos;
            let mut end = pos;
            
            // If we're not on a word character, try to find the nearest word
            if pos < chars.len() && !(chars[pos].is_alphanumeric() || chars[pos] == '_') {
                // Look backwards for a word character
                while start > 0 && !(chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
                    start -= 1;
                }
                if start > 0 {
                    start -= 1;
                    end = start;
                } else {
                    return None;
                }
            }
            
            // Expand backwards to find word start
            while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
                start -= 1;
            }
            
            // Expand forwards to find word end
            while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
                end += 1;
            }
            
            if start < end {
                let word: String = chars[start..end].iter().collect();
                
                let hover_text = match word.as_str() {
                    "fn" => "**fn** - Define a function\n\nSyntax: `fn name with param1, param2`",
                    "object" => "**object** - Define an object type\n\nSyntax: `object name`",
                    "store" => "**store** - Define a data store\n\nSyntax: `store name`",
                    "is" => "**is** - Assignment operator\n\nSyntax: `variable is value`",
                    "log" => "**log** - Output to console\n\nSyntax: `log expression`",
                    "if" => "**if** - Conditional statement\n\nSyntax: `if condition`",
                    "else" => "**else** - Alternative branch\n\nSyntax: `if condition ... else`",
                    "for" => "**for** - Loop over collection\n\nSyntax: `for item in collection`",
                    "while" => "**while** - Loop while condition\n\nSyntax: `while condition`",
                    "in" => "**in** - Membership operator\n\nUsed in for loops and expressions",
                    "and" => "**and** - Logical AND operator",
                    "or" => "**or** - Logical OR operator", 
                    "not" => "**not** - Logical NOT operator",
                    "gt" => "**gt** - Greater than comparison",
                    "lt" => "**lt** - Less than comparison",
                    "gte" => "**gte** - Greater than or equal comparison",
                    "lte" => "**lte** - Less than or equal comparison",
                    "equals" => "**equals** - Equality comparison",
                    "true" | "yes" => "**Boolean literal** - True value",
                    "false" | "no" => "**Boolean literal** - False value",
                    "make" => "**make** - Create instance\n\nSyntax: `make ObjectName`",
                    "with" => "**with** - Parameter delimiter\n\nUsed in function definitions",
                    "return" => "**return** - Return from function\n\nSyntax: `return value`",
                    "break" => "**break** - Exit loop\n\nBreaks out of current loop",
                    "continue" => "**continue** - Skip iteration\n\nSkips to next loop iteration",
                    "unless" => "**unless** - Conditional (inverted)\n\nSyntax: `unless condition`",
                    "until" => "**until** - Loop (inverted)\n\nSyntax: `until condition`",
                    _ => return None,
                };
                
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: hover_text.to_string(),
                    }),
                    range: Some(Range {
                        start: Position {
                            line: position.line,
                            character: start as u32,
                        },
                        end: Position {
                            line: position.line,
                            character: end as u32,
                        },
                    }),
                });
            }
        }
        
        None
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for CoralLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "coral-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![" ".to_string(), ".".to_string(), "\n".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
                        work_done_progress_options: Default::default(),
                        legend: SemanticTokensLegend {
                            token_types: vec![
                                SemanticTokenType::KEYWORD,     // 0
                                SemanticTokenType::STRING,      // 1  
                                SemanticTokenType::NUMBER,      // 2
                                SemanticTokenType::VARIABLE,    // 3
                                SemanticTokenType::FUNCTION,    // 4
                                SemanticTokenType::OPERATOR,    // 5
                                SemanticTokenType::COMMENT,     // 6
                            ],
                            token_modifiers: vec![],
                        },
                        range: Some(true),
                        full: Some(SemanticTokensFullOptions::Bool(true)),
                    }),
                ),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Coral LSP server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let rope = Rope::from_str(&params.text_document.text);
        self.documents.write().await.insert(params.text_document.uri.clone(), rope);
        
        let diagnostics = self.get_diagnostics(&params.text_document.uri, &params.text_document.text).await;
        self.client
            .publish_diagnostics(params.text_document.uri, diagnostics, Some(params.text_document.version))
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().next() {
            let rope = Rope::from_str(&change.text);
            self.documents.write().await.insert(params.text_document.uri.clone(), rope);
            
            let diagnostics = self.get_diagnostics(&params.text_document.uri, &change.text).await;
            self.client
                .publish_diagnostics(params.text_document.uri, diagnostics, Some(params.text_document.version))
                .await;
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let completions = self.get_completions(&params.text_document_position.text_document.uri, params.text_document_position.position).await;
        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn semantic_tokens_full(&self, params: SemanticTokensParams) -> Result<Option<SemanticTokensResult>> {
        let documents = self.documents.read().await;
        if let Some(rope) = documents.get(&params.text_document.uri) {
            let text = rope.to_string();
            let tokens = self.get_semantic_tokens(&text).await;
            
            Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: tokens,
            })))
        } else {
            Ok(None)
        }
    }

    async fn semantic_tokens_range(&self, params: SemanticTokensRangeParams) -> Result<Option<SemanticTokensRangeResult>> {
        let documents = self.documents.read().await;
        if let Some(rope) = documents.get(&params.text_document.uri) {
            let text = rope.to_string();
            let lines: Vec<&str> = text.lines().collect();
            
            let start_line = params.range.start.line as usize;
            let end_line = params.range.end.line as usize;
            
            if start_line < lines.len() && end_line < lines.len() {
                let range_text = lines[start_line..=end_line].join("\n");
                let tokens = self.get_semantic_tokens(&range_text).await;
                
                Ok(Some(SemanticTokensRangeResult::Tokens(SemanticTokens {
                    result_id: None,
                    data: tokens,
                })))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let documents = self.documents.read().await;
        if let Some(rope) = documents.get(&params.text_document_position_params.text_document.uri) {
            let text = rope.to_string();
            Ok(self.get_hover_info(&params.text_document_position_params.text_document.uri, params.text_document_position_params.position, &text).await)
        } else {
            Ok(None)
        }
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let documents = self.documents.read().await;
        if let Some(rope) = documents.get(&params.text_document.uri) {
            let text = rope.to_string();
            let formatted = self.format_coral_code(&text);
            
            let edit = TextEdit {
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { 
                        line: text.lines().count() as u32, 
                        character: 0 
                    },
                },
                new_text: formatted,
            };
            
            Ok(Some(vec![edit]))
        } else {
            Ok(None)
        }
    }
}

impl CoralLanguageServer {
    fn format_coral_code(&self, text: &str) -> String {
        // Basic Coral code formatting
        let lines: Vec<&str> = text.lines().collect();
        let mut formatted_lines = Vec::new();
        let mut indent_level: i32 = 0;
        
        for line in lines {
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                formatted_lines.push(String::new());
                continue;
            }
            
            // Decrease indent for certain keywords
            if trimmed.starts_with("else") {
                indent_level = indent_level.saturating_sub(1);
            }
            
            // Add current line with proper indentation
            let indent = "    ".repeat(indent_level as usize);
            formatted_lines.push(format!("{}{}", indent, trimmed));
            
            // Increase indent for block-starting constructs
            if trimmed.ends_with("with") || 
               trimmed.starts_with("if ") ||
               trimmed.starts_with("else") ||
               trimmed.starts_with("for ") ||
               trimmed.starts_with("while ") ||
               trimmed.starts_with("fn ") ||
               trimmed.starts_with("object ") ||
               trimmed.starts_with("store ") {
                indent_level += 1;
            }
        }
        
        formatted_lines.join("\n")
    }
}