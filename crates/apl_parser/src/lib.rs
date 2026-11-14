use apl_ast::*;
use apl_lexer::{Token, TokenKind};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum ParseError {
    #[error("❌ Синтаксическая ошибка: неожиданный токен {0:?} на {1:?}, ожидалось {2}\n💡 Подсказка: Проверьте закрывающие скобки или разделители выражений.")]
    UnexpectedToken(TokenKind, Span, &'static str),
    #[error("❌ Неожиданный конец файла при разборе {0}")]
    UnexpectedEOF(&'static str),
    #[error("❌ Ручной лайфтайм '{0}' запрещён в YKL!\n💡 Подсказка: Уберите '{0}' — компилятор YKL автоматически рассчитывает времена жизни без ручных аннотаций.")]
    ManualLifetimeForbidden(String, Span),
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    pub errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    fn current(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .unwrap_or(self.tokens.last().unwrap())
    }

    fn peek(&self, offset: usize) -> &Token {
        self.tokens
            .get(self.pos + offset)
            .unwrap_or(self.tokens.last().unwrap())
    }

    fn advance(&mut self) -> &Token {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        self.tokens.get(self.pos - 1).unwrap()
    }

    fn check(&self, kind: &TokenKind) -> bool {
        &self.current().kind == kind
    }

    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: &TokenKind, desc: &'static str) -> Result<Token, ParseError> {
        if &self.current().kind == kind {
            Ok(self.advance().clone())
        } else {
            let err =
                ParseError::UnexpectedToken(self.current().kind.clone(), self.current().span, desc);
            self.errors.push(err.clone());
            Err(err)
        }
    }

    pub fn parse_program(&mut self) -> Program {
        let mut items = Vec::new();
        while self.current().kind != TokenKind::EOF {
            match self.parse_item() {
                Ok(item) => items.push(item),
                Err(e) => {
                    if !self.errors.contains(&e) {
                        self.errors.push(e);
                    }
                    // Error recovery: skip until next semicolon or keyword fn/struct/enum/trait/impl/pub
                    while self.current().kind != TokenKind::EOF {
                        let k = &self.current().kind;
                        if matches!(
                            k,
                            TokenKind::Fn
                                | TokenKind::Struct
                                | TokenKind::Enum
                                | TokenKind::Trait
                                | TokenKind::Impl
                                | TokenKind::Pub
                                | TokenKind::Async
                        ) {
                            break;
                        }
                        self.advance();
                    }
                }
            }
        }
        Program { items }
    }

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        let mut is_pub = self.match_token(&TokenKind::Pub);
        let mut is_packed = false;
        if self.current().kind == TokenKind::At {
            self.advance();
            if let TokenKind::Ident(attr) = &self.current().kind {
                if attr == "packed" {
                    is_packed = true;
                    self.advance();
                }
            }
        }
        if !is_pub {
            is_pub = self.match_token(&TokenKind::Pub);
        }

        match &self.current().kind {
            TokenKind::Package => {
                let span_start = self.advance().span;
                let name = match self.advance().kind.clone() {
                    TokenKind::Ident(n) => n,
                    _ => {
                        return Err(ParseError::UnexpectedToken(
                            self.current().kind.clone(),
                            self.current().span,
                            "package name",
                        ));
                    }
                };
                self.match_token(&TokenKind::Semicolon);
                Ok(Item::Package(name, span_start))
            }
            TokenKind::Import => {
                let span_start = self.advance().span;
                let mut path = String::new();
                
                if let TokenKind::Literal(apl_ast::Literal::String(s)) = self.current().kind.clone() {
                    self.advance();
                    path = s;
                } else if let TokenKind::Ident(n) = self.advance().kind.clone() {
                    path.push_str(&n);
                    while self.match_token(&TokenKind::ColonColon) || self.match_token(&TokenKind::Dot) || self.match_token(&TokenKind::Slash) {
                        path.push('/');
                        if let TokenKind::Ident(next_seg) = self.advance().kind.clone() {
                            path.push_str(&next_seg);
                        }
                    }
                    if !path.ends_with(".yk") && !path.contains("::") {
                        path.push_str(".yk");
                    }
                }
                self.match_token(&TokenKind::Semicolon);
                Ok(Item::Import(path, span_start))
            }
            TokenKind::Async | TokenKind::Fn => {
                let func = self.parse_function(is_pub)?;
                Ok(Item::Function(func))
            }
            TokenKind::Struct => {
                let st = self.parse_struct(is_pub, is_packed)?;
                Ok(Item::Struct(st))
            }
            TokenKind::Enum => {
                let en = self.parse_enum(is_pub)?;
                Ok(Item::Enum(en))
            }
            TokenKind::Trait => {
                let tr = self.parse_trait(is_pub)?;
                Ok(Item::Trait(tr))
            }
            TokenKind::Impl => {
                let im = self.parse_impl()?;
                Ok(Item::Impl(im))
            }
            TokenKind::Extern => {
                let block = self.parse_extern_block()?;
                Ok(block)
            }
            _ => {
                let err = ParseError::UnexpectedToken(
                    self.current().kind.clone(),
                    self.current().span,
                    "top-level item",
                );
                self.errors.push(err.clone());
                Err(err)
            }
        }
    }

    fn parse_extern_block(&mut self) -> Result<Item, ParseError> {
        let span_start = self.current().span;
        self.expect(&TokenKind::Extern, "extern")?;
        self.expect(&TokenKind::LBrace, "{")?;
        let mut decls = Vec::new();
        while self.current().kind != TokenKind::RBrace && self.current().kind != TokenKind::EOF {
            let decl_span = self.current().span;
            self.expect(&TokenKind::Fn, "fn")?;
            let name = match self.advance().kind.clone() {
                TokenKind::Ident(n) => n,
                _ => return Err(ParseError::UnexpectedToken(self.current().kind.clone(), self.current().span, "function name")),
            };
            self.expect(&TokenKind::LParen, "(")?;
            let mut params = Vec::new();
            while self.current().kind != TokenKind::RParen && self.current().kind != TokenKind::EOF {
                let p_name = match self.advance().kind.clone() {
                    TokenKind::Ident(n) => n,
                    _ => return Err(ParseError::UnexpectedToken(self.current().kind.clone(), self.current().span, "param name")),
                };
                self.expect(&TokenKind::Colon, ":")?;
                let p_ty = self.parse_type()?;
                params.push((p_name, p_ty));
                if !self.match_token(&TokenKind::Comma) { break; }
            }
            self.expect(&TokenKind::RParen, ")")?;
            let ret_ty = if self.match_token(&TokenKind::Arrow) {
                self.parse_type()?
            } else {
                Type::Void
            };
            self.expect(&TokenKind::Semicolon, ";")?;
            decls.push(ExternDecl {
                name,
                params,
                return_type: ret_ty,
                span: decl_span,
            });
        }
        let span_end = self.expect(&TokenKind::RBrace, "}")?.span;
        Ok(Item::ExternBlock(decls, Span::new(span_start.file_id, span_start.start, span_end.end, span_start.line, span_end.col)))
    }

    fn parse_generics(&mut self) -> Result<Vec<String>, ParseError> {
        let mut generics = Vec::new();
        if self.match_token(&TokenKind::Lt) {
            while self.current().kind != TokenKind::Gt && self.current().kind != TokenKind::EOF {
                if let TokenKind::Lifetime(lt) = &self.current().kind {
                    return Err(ParseError::ManualLifetimeForbidden(lt.clone(), self.current().span));
                }
                if let TokenKind::Ident(name) = self.advance().kind.clone() {
                    generics.push(name);
                }
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(&TokenKind::Gt, ">")?;
        }
        Ok(generics)
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        if let TokenKind::Lifetime(lt) = &self.current().kind {
            return Err(ParseError::ManualLifetimeForbidden(lt.clone(), self.current().span));
        }
        if self.match_token(&TokenKind::Ampersand) {
            if let TokenKind::Lifetime(lt) = &self.current().kind {
                return Err(ParseError::ManualLifetimeForbidden(lt.clone(), self.current().span));
            }
            if self.match_token(&TokenKind::Mut) {
                let inner = self.parse_type()?;
                return Ok(Type::RefMut(Box::new(inner)));
            } else {
                let inner = self.parse_type()?;
                return Ok(Type::Ref(Box::new(inner)));
            }
        }
        if self.match_token(&TokenKind::Question) {
            let inner = self.parse_type()?;
            return Ok(Type::Nullable(Box::new(inner)));
        }
        if self.match_token(&TokenKind::LBracket) {
            let inner = self.parse_type()?;
            if self.match_token(&TokenKind::Semicolon) {
                // Fixed size array: [T; N]
                let tok = self.advance().clone();
                let size = match tok.kind {
                    TokenKind::Literal(Literal::Int(n)) => n as usize,
                    _ => {
                        return Err(ParseError::UnexpectedToken(
                            tok.kind,
                            tok.span,
                            "array size integer",
                        ));
                    }
                };
                self.expect(&TokenKind::RBracket, "]")?;
                return Ok(Type::Array(Box::new(inner), size));
            } else {
                // Slice: [T]
                self.expect(&TokenKind::RBracket, "]")?;
                return Ok(Type::Slice(Box::new(inner)));
            }
        }

        let tok = self.advance().clone();
        match tok.kind {
            TokenKind::Ident(name) => match name.as_str() {
                // Интуитивные базовые типы (Идея 5):
                "int" => Ok(Type::I32),
                "byte" => Ok(Type::U8),
                "float" => Ok(Type::F64),
                "string" | "String" | "str" => Ok(Type::String),
                "bool" => Ok(Type::Bool),
                // Точные системные типы:
                "i8" => Ok(Type::I8),
                "i16" => Ok(Type::I16),
                "i32" => Ok(Type::I32),
                "i64" => Ok(Type::I64),
                "isize" => Ok(Type::ISize),
                "u8" => Ok(Type::U8),
                "u16" => Ok(Type::U16),
                "u32" => Ok(Type::U32),
                "u64" => Ok(Type::U64),
                "usize" => Ok(Type::USize),
                "f32" => Ok(Type::F32),
                "f64" => Ok(Type::F64),
                "char" => Ok(Type::Char),
                "Ptr" => Ok(Type::Ptr),
                "void" => Ok(Type::Void),
                _ => {
                    let mut type_args = Vec::new();
                    if self.check(&TokenKind::Lt) {
                        self.advance();
                        while self.current().kind != TokenKind::Gt
                            && self.current().kind != TokenKind::EOF
                        {
                            type_args.push(self.parse_type()?);
                            if !self.match_token(&TokenKind::Comma) {
                                break;
                            }
                        }
                        self.expect(&TokenKind::Gt, ">")?;
                    }
                    if name == "Receiver" || name == "Sender" || name == "Channel" {
                        if !type_args.is_empty() {
                            return Ok(Type::Channel(Box::new(type_args[0].clone())));
                        }
                    }
                    Ok(Type::Named(name, type_args))
                }
            },
            _ => Err(ParseError::UnexpectedToken(tok.kind, tok.span, "type name")),
        }
    }

    fn parse_function(&mut self, is_pub: bool) -> Result<Function, ParseError> {
        let start_span = self.current().span;
        let is_async = self.match_token(&TokenKind::Async);
        self.expect(&TokenKind::Fn, "fn")?;

        let name = match self.advance().kind.clone() {
            TokenKind::Ident(n) => n,
            _ => {
                return Err(ParseError::UnexpectedToken(
                    self.current().kind.clone(),
                    self.current().span,
                    "function name",
                ));
            }
        };

        let generics = self.parse_generics()?;
        self.expect(&TokenKind::LParen, "(")?;

        let mut params = Vec::new();
        while self.current().kind != TokenKind::RParen && self.current().kind != TokenKind::EOF {
            let param_span = self.current().span;
            let mut is_ref = false;
            let mut is_mut_ref = false;

            if self.match_token(&TokenKind::Ampersand) {
                if self.match_token(&TokenKind::Mut) {
                    is_mut_ref = true;
                } else {
                    is_ref = true;
                }
            } else {
                let _ = self.match_token(&TokenKind::Mut);
            }

            let param_name = if let TokenKind::Ident(n) = self.advance().kind.clone() {
                n
            } else {
                return Err(ParseError::UnexpectedToken(
                    self.current().kind.clone(),
                    self.current().span,
                    "param name",
                ));
            };

            let ty = if self.match_token(&TokenKind::Colon) {
                self.parse_type()?
            } else {
                let base_self = Type::Named("Self".to_string(), vec![]);
                if is_mut_ref {
                    Type::RefMut(Box::new(base_self))
                } else if is_ref {
                    Type::Ref(Box::new(base_self))
                } else {
                    base_self
                }
            };

            params.push(Param {
                name: param_name,
                ty,
                span: param_span,
            });
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        self.expect(&TokenKind::RParen, ")")?;

        let return_type = if self.match_token(&TokenKind::Arrow) {
            self.parse_type()?
        } else {
            Type::Void
        };

        let body = if self.check(&TokenKind::LBrace) {
            self.parse_block_expr()?
        } else {
            self.match_token(&TokenKind::Semicolon);
            Expr {
                kind: ExprKind::Block(vec![], None),
                span: start_span,
            }
        };

        Ok(Function {
            name,
            generics,
            params,
            return_type,
            is_async,
            is_pub,
            body,
            span: start_span,
        })
    }

    fn parse_struct(&mut self, is_pub: bool, is_packed: bool) -> Result<StructDecl, ParseError> {
        let start_span = self.current().span;
        self.expect(&TokenKind::Struct, "struct")?;
        let name = match self.advance().kind.clone() {
            TokenKind::Ident(n) => n,
            _ => {
                return Err(ParseError::UnexpectedToken(
                    self.current().kind.clone(),
                    self.current().span,
                    "struct name",
                ));
            }
        };
        let generics = self.parse_generics()?;
        self.expect(&TokenKind::LBrace, "{")?;

        let mut fields = Vec::new();
        let mut methods = Vec::new();
        while self.current().kind != TokenKind::RBrace && self.current().kind != TokenKind::EOF {
            let member_pub = self.match_token(&TokenKind::Pub);

            // Идея 1: Если внутри struct встретили `fn`, парсим метод прямо внутри структуры!
            if self.current().kind == TokenKind::Fn {
                let mut func = self.parse_function(member_pub)?;
                let has_self = func.params.first().map(|p| p.name == "self").unwrap_or(false);
                if !has_self {
                    func.params.insert(
                        0,
                        Param {
                            name: "self".to_string(),
                            ty: Type::RefMut(Box::new(Type::Named(name.clone(), vec![]))),
                            span: func.span,
                        },
                    );
                }
                methods.push(func);
                let _ = self.match_token(&TokenKind::Semicolon);
                continue;
            }

            let f_name = match self.advance().kind.clone() {
                TokenKind::Ident(n) => n,
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        self.current().kind.clone(),
                        self.current().span,
                        "field name or method",
                    ));
                }
            };
            self.expect(&TokenKind::Colon, ":")?;
            let ty = self.parse_type()?;
            fields.push(StructField {
                name: f_name,
                ty,
                is_pub: member_pub,
            });
            let _ = self.match_token(&TokenKind::Comma);
            let _ = self.match_token(&TokenKind::Semicolon);
        }
        self.expect(&TokenKind::RBrace, "}")?;

        Ok(StructDecl {
            name,
            generics,
            fields,
            methods,
            is_pub,
            is_packed,
            span: start_span,
        })
    }

    fn parse_enum(&mut self, is_pub: bool) -> Result<EnumDecl, ParseError> {
        let start_span = self.current().span;
        self.expect(&TokenKind::Enum, "enum")?;
        let name = match self.advance().kind.clone() {
            TokenKind::Ident(n) => n,
            _ => {
                return Err(ParseError::UnexpectedToken(
                    self.current().kind.clone(),
                    self.current().span,
                    "enum name",
                ));
            }
        };
        let generics = self.parse_generics()?;
        self.expect(&TokenKind::LBrace, "{")?;

        let mut variants = Vec::new();
        while self.current().kind != TokenKind::RBrace && self.current().kind != TokenKind::EOF {
            let v_name = match self.advance().kind.clone() {
                TokenKind::Ident(n) => n,
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        self.current().kind.clone(),
                        self.current().span,
                        "variant name",
                    ));
                }
            };
            let mut types = Vec::new();
            if self.match_token(&TokenKind::LParen) {
                while self.current().kind != TokenKind::RParen
                    && self.current().kind != TokenKind::EOF
                {
                    types.push(self.parse_type()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(&TokenKind::RParen, ")")?;
            }
            variants.push(EnumVariantDecl {
                name: v_name,
                types,
            });
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        self.expect(&TokenKind::RBrace, "}")?;

        Ok(EnumDecl {
            name,
            generics,
            variants,
            is_pub,
            span: start_span,
        })
    }

    fn parse_trait(&mut self, is_pub: bool) -> Result<TraitDecl, ParseError> {
        let start_span = self.current().span;
        self.expect(&TokenKind::Trait, "trait")?;
        let name = match self.advance().kind.clone() {
            TokenKind::Ident(n) => n,
            _ => {
                return Err(ParseError::UnexpectedToken(
                    self.current().kind.clone(),
                    self.current().span,
                    "trait name",
                ));
            }
        };
        self.expect(&TokenKind::LBrace, "{")?;

        let mut methods = Vec::new();
        while self.current().kind != TokenKind::RBrace && self.current().kind != TokenKind::EOF {
            methods.push(self.parse_function(true)?);
        }
        self.expect(&TokenKind::RBrace, "}")?;

        Ok(TraitDecl {
            name,
            methods,
            is_pub,
            span: start_span,
        })
    }

    fn parse_impl(&mut self) -> Result<ImplDecl, ParseError> {
        let start_span = self.current().span;
        self.expect(&TokenKind::Impl, "impl")?;
        let first = match self.advance().kind.clone() {
            TokenKind::Ident(n) => n,
            _ => {
                return Err(ParseError::UnexpectedToken(
                    self.current().kind.clone(),
                    self.current().span,
                    "impl target/trait",
                ));
            }
        };

        let (trait_name, target_type) = if self.match_token(&TokenKind::For) {
            let target = match self.advance().kind.clone() {
                TokenKind::Ident(n) => n,
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        self.current().kind.clone(),
                        self.current().span,
                        "target type",
                    ));
                }
            };
            (Some(first), target)
        } else {
            (None, first)
        };

        self.expect(&TokenKind::LBrace, "{")?;
        let mut methods = Vec::new();
        while self.current().kind != TokenKind::RBrace && self.current().kind != TokenKind::EOF {
            let is_pub = self.match_token(&TokenKind::Pub);
            methods.push(self.parse_function(is_pub)?);
        }
        self.expect(&TokenKind::RBrace, "}")?;

        Ok(ImplDecl {
            target_type,
            trait_name,
            methods,
            span: start_span,
        })
    }

    fn parse_block_expr(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current().span;
        self.expect(&TokenKind::LBrace, "{")?;
        let mut stmts = Vec::new();
        let mut trailing_expr = None;

        while self.current().kind != TokenKind::RBrace && self.current().kind != TokenKind::EOF {
            if self.match_token(&TokenKind::Let) {
                let mut_flag = self.match_token(&TokenKind::Mut);
                let name = match self.advance().kind.clone() {
                    TokenKind::Ident(n) => n,
                    _ => {
                        return Err(ParseError::UnexpectedToken(
                            self.current().kind.clone(),
                            self.current().span,
                            "variable name",
                        ));
                    }
                };
                let ty = if self.match_token(&TokenKind::Colon) {
                    Some(self.parse_type()?)
                } else {
                    None
                };
                self.expect(&TokenKind::Assign, "=")?;
                let init = self.parse_expr()?;
                self.match_token(&TokenKind::Semicolon);
                stmts.push(Stmt::Let {
                    name,
                    mutability: mut_flag,
                    ty,
                    init,
                    span: start_span,
                });
            } else if let TokenKind::Ident(var_name) = self.current().kind.clone() {
                if self.peek(1).kind == TokenKind::ColonAssign {
                    self.advance(); // consume identifier
                    self.advance(); // consume :=
                    let init = self.parse_expr()?;
                    self.match_token(&TokenKind::Semicolon);
                    stmts.push(Stmt::Let {
                        name: var_name,
                        mutability: true,
                        ty: None,
                        init,
                        span: start_span,
                    });
                } else {
                    let expr = self.parse_expr()?;
                    if self.match_token(&TokenKind::Semicolon) {
                        stmts.push(Stmt::Expr(expr));
                    } else if self.check(&TokenKind::RBrace) {
                        trailing_expr = Some(Box::new(expr));
                    } else {
                        stmts.push(Stmt::Expr(expr));
                    }
                }
            } else {
                let expr = self.parse_expr()?;
                if self.match_token(&TokenKind::Semicolon) {
                    stmts.push(Stmt::Expr(expr));
                } else if self.check(&TokenKind::RBrace) {
                    trailing_expr = Some(Box::new(expr));
                } else {
                    stmts.push(Stmt::Expr(expr));
                }
            }
        }
        self.expect(&TokenKind::RBrace, "}")?;

        Ok(Expr {
            kind: ExprKind::Block(stmts, trailing_expr),
            span: start_span,
        })
    }

    pub fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_logical_or()?;
        if self.match_token(&TokenKind::Assign) {
            let right = self.parse_assignment()?;
            let span = Span::new(expr.span.file_id, expr.span.start, right.span.end,
                expr.span.line,
                right.span.col,);
            return Ok(Expr {
                kind: ExprKind::Assign(Box::new(expr), Box::new(right)),
                span,
            });
        }
        Ok(expr)
    }

    fn parse_logical_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_logical_and()?;
        while self.match_token(&TokenKind::Or) {
            let right = self.parse_logical_and()?;
            let span = Span::new(left.span.file_id, left.span.start, right.span.end,
                left.span.line,
                right.span.col,);
            left = Expr {
                kind: ExprKind::Binary(BinaryOp::Or, Box::new(left), Box::new(right)),
                span,
            };
        }
        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_bitwise_or()?;
        while self.match_token(&TokenKind::And) {
            let right = self.parse_bitwise_or()?;
            let span = Span::new(left.span.file_id, left.span.start, right.span.end,
                left.span.line,
                right.span.col,);
            left = Expr {
                kind: ExprKind::Binary(BinaryOp::And, Box::new(left), Box::new(right)),
                span,
            };
        }
        Ok(left)
    }

    fn parse_bitwise_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_bitwise_xor()?;
        while self.match_token(&TokenKind::Pipe) {
            let right = self.parse_bitwise_xor()?;
            let span = Span::new(left.span.file_id, left.span.start, right.span.end,
                left.span.line,
                right.span.col,);
            left = Expr {
                kind: ExprKind::Binary(BinaryOp::BitOr, Box::new(left), Box::new(right)),
                span,
            };
        }
        Ok(left)
    }

    fn parse_bitwise_xor(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_bitwise_and()?;
        while self.match_token(&TokenKind::Caret) {
            let right = self.parse_bitwise_and()?;
            let span = Span::new(left.span.file_id, left.span.start, right.span.end,
                left.span.line,
                right.span.col,);
            left = Expr {
                kind: ExprKind::Binary(BinaryOp::BitXor, Box::new(left), Box::new(right)),
                span,
            };
        }
        Ok(left)
    }

    fn parse_bitwise_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_equality()?;
        while self.match_token(&TokenKind::Ampersand) {
            let right = self.parse_equality()?;
            let span = Span::new(left.span.file_id, left.span.start, right.span.end,
                left.span.line,
                right.span.col,);
            left = Expr {
                kind: ExprKind::Binary(BinaryOp::BitAnd, Box::new(left), Box::new(right)),
                span,
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_comparison()?;
        while let Some(op) = match self.current().kind {
            TokenKind::Eq => {
                self.advance();
                Some(BinaryOp::Eq)
            }
            TokenKind::Neq => {
                self.advance();
                Some(BinaryOp::Neq)
            }
            _ => None,
        } {
            let right = self.parse_comparison()?;
            let span = Span::new(left.span.file_id, left.span.start, right.span.end,
                left.span.line,
                right.span.col,);
            left = Expr {
                kind: ExprKind::Binary(op, Box::new(left), Box::new(right)),
                span,
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_shift()?;
        while let Some(op) = match self.current().kind {
            TokenKind::Lt => {
                self.advance();
                Some(BinaryOp::Lt)
            }
            TokenKind::Lte => {
                self.advance();
                Some(BinaryOp::Lte)
            }
            TokenKind::Gt => {
                self.advance();
                Some(BinaryOp::Gt)
            }
            TokenKind::Gte => {
                self.advance();
                Some(BinaryOp::Gte)
            }
            _ => None,
        } {
            let right = self.parse_shift()?;
            let span = Span::new(left.span.file_id, left.span.start, right.span.end,
                left.span.line,
                right.span.col,);
            left = Expr {
                kind: ExprKind::Binary(op, Box::new(left), Box::new(right)),
                span,
            };
        }
        Ok(left)
    }

    fn parse_shift(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_term()?;
        while let Some(op) = match self.current().kind {
            TokenKind::Shl => {
                self.advance();
                Some(BinaryOp::Shl)
            }
            TokenKind::Shr => {
                self.advance();
                Some(BinaryOp::Shr)
            }
            _ => None,
        } {
            let right = self.parse_term()?;
            let span = Span::new(left.span.file_id, left.span.start, right.span.end,
                left.span.line,
                right.span.col,);
            left = Expr {
                kind: ExprKind::Binary(op, Box::new(left), Box::new(right)),
                span,
            };
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_factor()?;
        while let Some(op) = match self.current().kind {
            TokenKind::Plus => {
                self.advance();
                Some(BinaryOp::Add)
            }
            TokenKind::Minus => {
                self.advance();
                Some(BinaryOp::Sub)
            }
            _ => None,
        } {
            let right = self.parse_factor()?;
            let span = Span::new(left.span.file_id, left.span.start, right.span.end,
                left.span.line,
                right.span.col,);
            left = Expr {
                kind: ExprKind::Binary(op, Box::new(left), Box::new(right)),
                span,
            };
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        while let Some(op) = match self.current().kind {
            TokenKind::Star => {
                self.advance();
                Some(BinaryOp::Mul)
            }
            TokenKind::Slash => {
                self.advance();
                Some(BinaryOp::Div)
            }
            TokenKind::Percent => {
                self.advance();
                Some(BinaryOp::Mod)
            }
            _ => None,
        } {
            let right = self.parse_unary()?;
            let span = Span::new(left.span.file_id, left.span.start, right.span.end,
                left.span.line,
                right.span.col,);
            left = Expr {
                kind: ExprKind::Binary(op, Box::new(left), Box::new(right)),
                span,
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current().span;
        if self.match_token(&TokenKind::Minus) {
            let operand = self.parse_unary()?;
            let span = Span::new(start_span.file_id, start_span.start, operand.span.end,
                start_span.line,
                operand.span.col,);
            return Ok(Expr {
                kind: ExprKind::Unary(UnaryOp::Neg, Box::new(operand)),
                span,
            });
        }
        if self.match_token(&TokenKind::Not) {
            let operand = self.parse_unary()?;
            let span = Span::new(start_span.file_id, start_span.start, operand.span.end,
                start_span.line,
                operand.span.col,);
            return Ok(Expr {
                kind: ExprKind::Unary(UnaryOp::Not, Box::new(operand)),
                span,
            });
        }
        if self.match_token(&TokenKind::Ampersand) {
            let op = if self.match_token(&TokenKind::Mut) {
                UnaryOp::RefMut
            } else {
                UnaryOp::Ref
            };
            let operand = self.parse_unary()?;
            let span = Span::new(start_span.file_id, start_span.start, operand.span.end,
                start_span.line,
                operand.span.col,);
            return Ok(Expr {
                kind: ExprKind::Unary(op, Box::new(operand)),
                span,
            });
        }
        if self.match_token(&TokenKind::Star) {
            let operand = self.parse_unary()?;
            let span = Span::new(start_span.file_id, start_span.start, operand.span.end,
                start_span.line,
                operand.span.col,);
            return Ok(Expr {
                kind: ExprKind::Unary(UnaryOp::Deref, Box::new(operand)),
                span,
            });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.match_token(&TokenKind::Dot) {
                if let TokenKind::Ident(method_or_field) = self.advance().kind.clone() {
                    if self.match_token(&TokenKind::LParen) {
                        let mut args = Vec::new();
                        while self.current().kind != TokenKind::RParen
                            && self.current().kind != TokenKind::EOF
                        {
                            args.push(self.parse_expr()?);
                            if !self.match_token(&TokenKind::Comma) {
                                break;
                            }
                        }
                        let end_span = self.expect(&TokenKind::RParen, ")")?.span;
                        let span =
                            Span::new(expr.span.file_id, expr.span.start, end_span.end, expr.span.line, end_span.col);
                        expr = Expr {
                            kind: ExprKind::MethodCall(Box::new(expr), method_or_field, args),
                            span,
                        };
                    } else {
                        let span = Span::new(expr.span.file_id, expr.span.start, expr.span.end + method_or_field.len(),
                            expr.span.line,
                            expr.span.col,
                        );
                        expr = Expr {
                            kind: ExprKind::FieldAccess(Box::new(expr), method_or_field),
                            span,
                        };
                    }
                }
            } else if self.match_token(&TokenKind::LParen) {
                // Function call
                let mut args = Vec::new();
                while self.current().kind != TokenKind::RParen
                    && self.current().kind != TokenKind::EOF
                {
                    args.push(self.parse_expr()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                let end_span = self.expect(&TokenKind::RParen, ")")?.span;
                let span = Span::new(expr.span.file_id, expr.span.start, end_span.end, expr.span.line, end_span.col);
                expr = Expr {
                    kind: ExprKind::Call(Box::new(expr), args),
                    span,
                };
            } else if self.match_token(&TokenKind::LBracket) {
                let index = self.parse_expr()?;
                let end_span = self.expect(&TokenKind::RBracket, "]")?.span;
                let span = Span::new(expr.span.file_id, expr.span.start, end_span.end, expr.span.line, end_span.col);
                expr = Expr {
                    kind: ExprKind::Index(Box::new(expr), Box::new(index)),
                    span,
                };
            } else if self.check(&TokenKind::Dot) && self.peek(1).kind == TokenKind::Await {
                self.advance(); // consume .
                let end_span = self.advance().span; // consume await
                let span = Span::new(expr.span.file_id, expr.span.start, end_span.end, expr.span.line, end_span.col);
                expr = Expr {
                    kind: ExprKind::Await(Box::new(expr)),
                    span,
                };
            } else if self.match_token(&TokenKind::Question) {
                let span = expr.span;
                expr = Expr {
                    kind: ExprKind::Unary(UnaryOp::TryUnwrap, Box::new(expr)),
                    span,
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let tok = self.advance().clone();
        match tok.kind {
            TokenKind::Literal(lit) => Ok(Expr {
                kind: ExprKind::Literal(lit),
                span: tok.span,
            }),
            TokenKind::Ident(mut name) => {
                while self.match_token(&TokenKind::ColonColon) {
                    name.push_str("::");
                    if let TokenKind::Ident(n) = self.advance().kind.clone() {
                        name.push_str(&n);
                    } else {
                        return Err(ParseError::UnexpectedToken(
                            self.current().kind.clone(),
                            self.current().span,
                            "path segment",
                        ));
                    }
                }

                // Now `name` is a full path, e.g. "Result::Ok" or "std::fs::open" or "x"

                // Check if struct init: Ident { field: val, ... }
                if self.check(&TokenKind::LBrace) && self.peek(1).kind != TokenKind::RBrace {
                    // Let's verify if next is Ident followed by colon
                    if let TokenKind::Ident(_) = self.peek(1).kind {
                        if self.peek(2).kind == TokenKind::Colon {
                            self.advance(); // consume {
                            let mut fields = Vec::new();
                            while self.current().kind != TokenKind::RBrace
                                && self.current().kind != TokenKind::EOF
                            {
                                let f_name = match self.advance().kind.clone() {
                                    TokenKind::Ident(n) => n,
                                    _ => {
                                        return Err(ParseError::UnexpectedToken(
                                            self.current().kind.clone(),
                                            self.current().span,
                                            "field name",
                                        ));
                                    }
                                };
                                self.expect(&TokenKind::Colon, ":")?;
                                let val = self.parse_expr()?;
                                fields.push((f_name, val));
                                if !self.match_token(&TokenKind::Comma) {
                                    break;
                                }
                            }
                            let end_span = self.expect(&TokenKind::RBrace, "}")?.span;
                            let span = Span::new(tok.span.file_id, tok.span.start, end_span.end,
                                tok.span.line,
                                end_span.col,);
                            return Ok(Expr {
                                kind: ExprKind::StructInit(name, fields),
                                span,
                            });
                        }
                    }
                }
                Ok(Expr {
                    kind: ExprKind::Ident(name),
                    span: tok.span,
                })
            }
            TokenKind::LParen => {
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::RParen, ")")?;
                Ok(expr)
            }
            TokenKind::LBrace => {
                self.pos -= 1; // re-consume {
                self.parse_block_expr()
            }
            TokenKind::If => {
                let cond = self.parse_expr()?;
                let then_branch = self.parse_block_expr()?;
                let else_branch = if self.match_token(&TokenKind::Else) {
                    if self.check(&TokenKind::If) {
                        Some(Box::new(self.parse_expr()?))
                    } else {
                        Some(Box::new(self.parse_block_expr()?))
                    }
                } else {
                    None
                };
                let end_span = else_branch
                    .as_ref()
                    .map(|e| e.span)
                    .unwrap_or(then_branch.span);
                let span = Span::new(tok.span.file_id, tok.span.start, end_span.end, tok.span.line, end_span.col);
                Ok(Expr {
                    kind: ExprKind::If(Box::new(cond), Box::new(then_branch), else_branch),
                    span,
                })
            }
            TokenKind::Match => {
                let target = self.parse_expr()?;
                self.expect(&TokenKind::LBrace, "{")?;
                let mut arms = Vec::new();
                while self.current().kind != TokenKind::RBrace
                    && self.current().kind != TokenKind::EOF
                {
                    let pattern = self.parse_pattern()?;
                    let guard = if self.match_token(&TokenKind::If) {
                        Some(Box::new(self.parse_expr()?))
                    } else {
                        None
                    };
                    self.expect(&TokenKind::FatArrow, "=>")?;
                    let body = self.parse_expr()?;
                    self.match_token(&TokenKind::Comma);
                    arms.push(MatchArm {
                        pattern,
                        guard,
                        body,
                    });
                }
                let end_span = self.expect(&TokenKind::RBrace, "}")?.span;
                let span = Span::new(tok.span.file_id, tok.span.start, end_span.end, tok.span.line, end_span.col);
                Ok(Expr {
                    kind: ExprKind::Match(Box::new(target), arms),
                    span,
                })
            }
            TokenKind::Return => {
                let val = if self.check(&TokenKind::Semicolon) || self.check(&TokenKind::RBrace) {
                    None
                } else {
                    Some(Box::new(self.parse_expr()?))
                };
                Ok(Expr {
                    kind: ExprKind::Return(val),
                    span: tok.span,
                })
            }
            TokenKind::Break => Ok(Expr {
                kind: ExprKind::Break,
                span: tok.span,
            }),
            TokenKind::Continue => Ok(Expr {
                kind: ExprKind::Continue,
                span: tok.span,
            }),
            TokenKind::Spawn => {
                let expr = self.parse_expr()?;
                let span = Span::new(tok.span.file_id, tok.span.start, expr.span.end, tok.span.line, expr.span.col);
                Ok(Expr {
                    kind: ExprKind::Spawn(Box::new(expr)),
                    span,
                })
            }
            TokenKind::Loop => {
                let body = self.parse_block_expr()?;
                let span = Span::new(tok.span.file_id, tok.span.start, body.span.end, tok.span.line, body.span.col);
                Ok(Expr {
                    kind: ExprKind::Loop(Box::new(body)),
                    span,
                })
            }
            TokenKind::While => {
                let cond = self.parse_expr()?;
                let body = self.parse_block_expr()?;
                let span = Span::new(tok.span.file_id, tok.span.start, body.span.end, tok.span.line, body.span.col);
                Ok(Expr {
                    kind: ExprKind::While(Box::new(cond), Box::new(body)),
                    span,
                })
            }
            TokenKind::LBracket => {
                let mut items = Vec::new();
                while self.current().kind != TokenKind::RBracket
                    && self.current().kind != TokenKind::EOF
                {
                    items.push(self.parse_expr()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                let end_span = self.expect(&TokenKind::RBracket, "]")?.span;
                let span = Span::new(tok.span.file_id, tok.span.start, end_span.end, tok.span.line, end_span.col);
                Ok(Expr {
                    kind: ExprKind::ArrayInit(items),
                    span,
                })
            }
            _ => Err(ParseError::UnexpectedToken(
                tok.kind,
                tok.span,
                "expression",
            )),
        }
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        let tok = self.advance().clone();
        match tok.kind {
            TokenKind::Ident(name) => {
                if name == "_" {
                    Ok(Pattern::Wildcard)
                } else if self.match_token(&TokenKind::ColonColon) {
                    let var_name = match self.advance().kind.clone() {
                        TokenKind::Ident(n) => n,
                        _ => {
                            return Err(ParseError::UnexpectedToken(
                                self.current().kind.clone(),
                                self.current().span,
                                "variant name",
                            ));
                        }
                    };
                    let mut sub = Vec::new();
                    if self.match_token(&TokenKind::LParen) {
                        while self.current().kind != TokenKind::RParen
                            && self.current().kind != TokenKind::EOF
                        {
                            sub.push(self.parse_pattern()?);
                            if !self.match_token(&TokenKind::Comma) {
                                break;
                            }
                        }
                        self.expect(&TokenKind::RParen, ")")?;
                    }
                    Ok(Pattern::EnumVariant(name, var_name, sub))
                } else {
                    Ok(Pattern::Ident(name))
                }
            }
            TokenKind::Literal(lit) => Ok(Pattern::Literal(lit)),
            _ => Err(ParseError::UnexpectedToken(tok.kind, tok.span, "pattern")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apl_lexer::Lexer;

    #[test]
    fn test_parser_basic() {
        let input =
            "fn factorial(n: i32) -> i32 { if n <= 1 { return 1; } return n * factorial(n - 1); }";
        let mut lexer = Lexer::new(input, 0);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let prog = parser.parse_program();
        assert!(parser.errors.is_empty());
        assert_eq!(prog.items.len(), 1);
    }
}
