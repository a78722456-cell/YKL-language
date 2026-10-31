use apl_ast::Span;
use apl_lexer::{LexError, Lexer};
use apl_parser::{ParseError, Parser};
use apl_sema::{SemaError, SemanticAnalyzer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: Option<String>,
    pub message: String,
    pub span: Span,
    pub notes: Vec<String>,
}

impl Diagnostic {
    pub fn from_lex_error(err: LexError, _filename: &str) -> Self {
        let (msg, span) = match err {
            LexError::UnexpectedChar(c, s) => (format!("unexpected character '{}'", c), s),
            LexError::UnterminatedString(s) => ("unterminated string literal".to_string(), s),
            LexError::UnterminatedChar(s) => ("unterminated character literal".to_string(), s),
            LexError::InvalidNumber(n, s) => (format!("invalid number literal '{}'", n), s),
        };
        Self {
            severity: Severity::Error,
            code: Some("E0100".to_string()),
            message: msg,
            span,
            notes: vec![],
        }
    }

    pub fn from_parse_error(err: ParseError, _filename: &str) -> Self {
        let (msg, span) = match err {
            ParseError::UnexpectedToken(k, s, desc) => {
                (format!("unexpected token {:?}, expected {}", k, desc), s)
            }
            ParseError::UnexpectedEOF(desc) => (
                format!("unexpected EOF while parsing {}", desc),
                Span::dummy(),
            ),
        };
        Self {
            severity: Severity::Error,
            code: Some("E0200".to_string()),
            message: msg,
            span,
            notes: vec![],
        }
    }

    pub fn from_sema_error(err: SemaError, _filename: &str) -> Self {
        let (msg, span, code) = match err {
            SemaError::UndefinedSymbol(n, s) => (format!("undefined symbol '{}'", n), s, "E0301"),
            SemaError::TypeMismatch(exp, act, s) => (
                format!("type mismatch: expected '{:?}', got '{:?}'", exp, act),
                s,
                "E0302",
            ),
            SemaError::UseAfterMove(n, s) => (format!("use of moved value '{}'", n), s, "E0382"),
            SemaError::BorrowConflict(n, s) => (
                format!(
                    "cannot borrow '{}' mutably because it is already borrowed",
                    n
                ),
                s,
                "E0383",
            ),
            SemaError::ImmutableAssignment(n, s) => (
                format!("cannot assign to immutable variable '{}'", n),
                s,
                "E0384",
            ),
            SemaError::DuplicateSymbol(name, s) => {
                (format!("duplicate symbol '{}'", name), s, "E0400")
            }
            SemaError::MissingField(s_name, f_name, s) => (
                format!("missing field '{}' in struct '{}'", f_name, s_name),
                s,
                "E0401",
            ),
            SemaError::UnknownField(s_name, f_name, s) => (
                format!("unknown field '{}' in struct '{}'", f_name, s_name),
                s,
                "E0402",
            ),
            SemaError::BreakOutsideLoop(s) => {
                ("break or continue outside of loop".to_string(), s, "E0500")
            }
        };
        Self {
            severity: Severity::Error,
            code: Some(code.to_string()),
            message: msg,
            span,
            notes: vec![],
        }
    }
}

pub struct CompileOutput {
    pub diagnostics: Vec<Diagnostic>,
    pub success: bool,
}

pub fn compile_source(source: &str, filename: &str) -> CompileOutput {
    let mut output = CompileOutput {
        diagnostics: vec![],
        success: false,
    };

    let mut lexer = Lexer::new(source, 0);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            output
                .diagnostics
                .push(Diagnostic::from_lex_error(e, filename));
            return output;
        }
    };

    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program();
    for err in parser.errors {
        output
            .diagnostics
            .push(Diagnostic::from_parse_error(err, filename));
    }

    if !output.diagnostics.is_empty() {
        return output;
    }

    let mut sema = SemanticAnalyzer::new();
    if let Err(errors) = sema.analyze_program(&ast) {
        for err in errors {
            output
                .diagnostics
                .push(Diagnostic::from_sema_error(err, filename));
        }
    }

    output.success = output.diagnostics.is_empty();
    output
}

pub fn format_diagnostics(diagnostics: &[Diagnostic], filename: &str, source: &str) -> String {
    let mut res = String::new();
    let lines: Vec<&str> = source.lines().collect();

    for diag in diagnostics {
        let code_str = diag.code.as_deref().unwrap_or("E0000");
        res.push_str(&format!("error[{}]: {}\n", code_str, diag.message));
        res.push_str(&format!(
            " --> {}:{}:{}\n",
            filename, diag.span.line, diag.span.col
        ));

        let line_idx = diag.span.line.saturating_sub(1);
        if line_idx < lines.len() {
            let line_text = lines[line_idx];
            res.push_str(&format!("  |\n"));
            res.push_str(&format!("{} | {}\n", diag.span.line, line_text));
            res.push_str(&format!("  | "));
            for _ in 0..diag.span.col.saturating_sub(1) {
                res.push(' ');
            }
            res.push_str("^ error occurred here\n");
        }
        res.push_str("  |\n");
    }
    res
}
