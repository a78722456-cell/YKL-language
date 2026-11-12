use apl_lexer::Lexer;
use apl_parser::Parser;
use apl_sema::SemanticAnalyzer;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use apl_ast::{Span, Type};

struct Backend {
    client: Client,
    // Store type_map per file URI
    document_types: Arc<RwLock<HashMap<String, HashMap<Span, Type>>>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![":".to_string(), ".".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(
                MessageType::INFO,
                "Young Kazah Language Server (apl-lsp) initialized!",
            )
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.analyze_document(params.text_document.uri, &params.text_document.text)
            .await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.pop() {
            self.analyze_document(params.text_document.uri, &change.text)
                .await;
        }
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        let keywords = vec![
            "fn", "let", "mut", "if", "else", "match", "return", "struct", "enum", "trait", "impl",
            "async", "await", "spawn", "channel", "loop", "while", "for", "in", "pub", "import",
            "package", "true", "false", "None", "println",
        ];

        let items: Vec<CompletionItem> = keywords
            .into_iter()
            .map(|kw| CompletionItem {
                label: kw.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("YKL keyword/builtin".to_string()),
                ..Default::default()
            })
            .collect();

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let position = params.text_document_position_params.position;
        let line = (position.line + 1) as usize; // YKL spans are 1-indexed
        let col = (position.character + 1) as usize;

        let docs = self.document_types.read().await;
        if let Some(type_map) = docs.get(&uri) {
            // Find the narrowest span containing the cursor
            let mut best_match: Option<(&Span, &Type)> = None;

            for (span, ty) in type_map {
                if span.line == line && col >= span.col && col <= span.col + (span.end - span.start) {
                    if let Some((best_span, _)) = best_match {
                        // Pick the more specific (shorter) span
                        if (span.end - span.start) < (best_span.end - best_span.start) {
                            best_match = Some((span, ty));
                        }
                    } else {
                        best_match = Some((span, ty));
                    }
                }
            }

            if let Some((_, ty)) = best_match {
                let ty_str = format!("{}", ty);
                return Ok(Some(Hover {
                    contents: HoverContents::Scalar(MarkedString::String(format!(
                        "```ykl\n{}\n```",
                        ty_str
                    ))),
                    range: None,
                }));
            }
        }

        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(
                "**YKL Language Server**\nNo type information found.".to_string(),
            )),
            range: None,
        }))
    }
}

impl Backend {
    async fn analyze_document(&self, uri: Url, text: &str) {
        let mut diagnostics = Vec::new();

        let mut lexer = Lexer::new(text, 0);
        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(e) => {
                let span = match e {
                    apl_lexer::LexError::UnexpectedChar(_, s)
                    | apl_lexer::LexError::UnterminatedString(s)
                    | apl_lexer::LexError::UnterminatedChar(s)
                    | apl_lexer::LexError::InvalidNumber(_, s) => s,
                };
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: (span.line.saturating_sub(1)) as u32,
                            character: (span.col.saturating_sub(1)) as u32,
                        },
                        end: Position {
                            line: (span.line.saturating_sub(1)) as u32,
                            character: (span.col + 1) as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    source: Some("apl-lexer".to_string()),
                    message: format!("{}", e),
                    ..Default::default()
                });
                self.client
                    .publish_diagnostics(uri, diagnostics, None)
                    .await;
                return;
            }
        };

        let mut parser = Parser::new(tokens);
        let prog = parser.parse_program();
        for pe in &parser.errors {
            let span = match pe {
                apl_parser::ParseError::UnexpectedToken(_, s, _) => *s,
                apl_parser::ParseError::UnexpectedEOF(_) => apl_ast::Span::dummy(),
                apl_parser::ParseError::ManualLifetimeForbidden(_, s) => *s,
            };
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        line: (span.line.saturating_sub(1)) as u32,
                        character: (span.col.saturating_sub(1)) as u32,
                    },
                    end: Position {
                        line: (span.line.saturating_sub(1)) as u32,
                        character: (span.col + 5) as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("apl-parser".to_string()),
                message: format!("{}", pe),
                ..Default::default()
            });
        }

        if diagnostics.is_empty() {
            let mut sema = SemanticAnalyzer::new();
            if let Err(errs) = sema.analyze_program(&prog) {
                for se in errs {
                    let span = match &se {
                        apl_sema::SemaError::UndefinedSymbol(_, s)
                        | apl_sema::SemaError::TypeMismatch(_, _, s)
                        | apl_sema::SemaError::UseAfterMove(_, s)
                        | apl_sema::SemaError::BorrowConflict(_, s)
                        | apl_sema::SemaError::ImmutableAssignment(_, s)
                        | apl_sema::SemaError::DuplicateSymbol(_, s)
                        | apl_sema::SemaError::MissingField(_, _, s)
                        | apl_sema::SemaError::UnknownField(_, _, s)
                        | apl_sema::SemaError::BreakOutsideLoop(s) => *s,
                    };
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: (span.line.saturating_sub(1)) as u32,
                                character: (span.col.saturating_sub(1)) as u32,
                            },
                            end: Position {
                                line: (span.line.saturating_sub(1)) as u32,
                                character: (span.col + 5) as u32,
                            },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        source: Some("apl-borrow-checker".to_string()),
                        message: format!("{}", se),
                        ..Default::default()
                    });
                }
            }

            for sw in sema.warnings {
                let span = match &sw {
                    apl_sema::SemaWarning::UnusedVariable(_, s) => *s,
                    apl_sema::SemaWarning::UnusedFunction(_, s) => *s,
                    apl_sema::SemaWarning::NamingConvention(_, _, s) => *s,
                };
                let msg = match &sw {
                    apl_sema::SemaWarning::UnusedVariable(n, _) => format!("Unused variable: `{}`", n),
                    apl_sema::SemaWarning::UnusedFunction(n, _) => format!("Unused function: `{}`", n),
                    apl_sema::SemaWarning::NamingConvention(n, e, _) => format!("Invalid naming convention: `{}` should be `{}`", n, e),
                };
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: (span.line.saturating_sub(1)) as u32,
                            character: (span.col.saturating_sub(1)) as u32,
                        },
                        end: Position {
                            line: (span.line.saturating_sub(1)) as u32,
                            character: (span.col + 5) as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::WARNING),
                    source: Some("apl-linter".to_string()),
                    message: msg,
                    ..Default::default()
                });
            }
            
            // Store type map for hover support
            let mut docs = self.document_types.write().await;
            docs.insert(uri.to_string(), sema.type_map.clone());
        }

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { 
        client,
        document_types: Arc::new(RwLock::new(HashMap::new())),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
