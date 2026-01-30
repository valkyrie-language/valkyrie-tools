use crate::ast::{
    lexer::{Lexer, Token, TokenKind},
    BinaryOperator, Expression, LiteralValue, MatchPattern, Position, ProgramNode, Span, Statement, TypeExpression,
    UnaryOperator,
};

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEOF,
    UnexpectedToken(Token),
}

impl From<ParseError> for String {
    fn from(e: ParseError) -> Self {
        format!("{:?}", e)
    }
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let current = lexer.next_token();
        Self { lexer, current }
    }

    fn bump(&mut self) {
        self.current = self.lexer.next_token();
    }

    fn peek(&self) -> Token {
        let mut lexer = self.lexer.clone();
        lexer.next_token()
    }

    fn expect_eof(&self) -> Result<(), ParseError> {
        match self.current.kind {
            TokenKind::EOF => Ok(()),
            _ => Err(ParseError::UnexpectedToken(self.current.clone())),
        }
    }

    fn parse_generic_params(&mut self) -> Result<Vec<String>, ParseError> {
        let mut params = Vec::new();
        if self.current.kind == TokenKind::Less {
            self.bump();
            loop {
                if let TokenKind::Identifier(name) = &self.current.kind {
                     params.push(name.clone());
                     self.bump();
                } else {
                     return Err(ParseError::UnexpectedToken(self.current.clone()));
                }

                if self.current.kind == TokenKind::Comma {
                    self.bump();
                } else {
                    break;
                }
            }
            if self.current.kind != TokenKind::Greater {
                return Err(ParseError::UnexpectedToken(self.current.clone()));
            }
            self.bump();
        }
        Ok(params)
    }

    pub fn parse_program(&mut self) -> Result<ProgramNode, ParseError> {
        let start = self.current.span.start;
        let mut statements = Vec::new();
        while self.current.kind != TokenKind::EOF {
            statements.push(self.parse_statement()?);
        }
        let end = self.current.span.end; // This will be EOF span
        Ok(ProgramNode { statements, span: Span { start, end } })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.current.kind {
            TokenKind::Namespace => self.parse_namespace(),
            TokenKind::Using => self.parse_using(),
            TokenKind::Class => self.parse_class(),
            TokenKind::Trait => self.parse_trait(),
            TokenKind::Imply => self.parse_imply(),
            TokenKind::Let => self.parse_let(),
            TokenKind::Return => self.parse_return(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::Micro => self.parse_function(false),
            TokenKind::Async => {
                let start = self.current.span.start;
                self.bump();
                if self.current.kind == TokenKind::Micro {
                    let mut func = self.parse_function(true)?;
                    // Adjust span start to include 'async'
                    if let Statement::Function { span, .. } = &mut func {
                        span.start = start;
                    }
                    Ok(func)
                }
                else {
                    Err(ParseError::UnexpectedToken(self.current.clone()))
                }
            }
            TokenKind::Yield => self.parse_yield(),
            TokenKind::Macro => self.parse_macro(),
            TokenKind::BraceL => self.parse_block(),
            TokenKind::At => {
                let start = self.current.span.start;
                self.bump(); // consume '@'

                if self.current.kind == TokenKind::Dot {
                    self.parse_annotation(start, true)
                }
                else {
                    // It must be a macro call expression statement
                    let expr = self.parse_macro_call_inner(start)?;
                    let mut end = expr.span().end;
                    if self.current.kind == TokenKind::SemiColon {
                        end = self.current.span.end;
                        self.bump();
                    }
                    Ok(Statement::Expression { expression: expr, span: Span { start, end } })
                }
            }
            TokenKind::ZigZag => {
                let start = self.current.span.start;
                self.parse_annotation(start, false)
            }
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_namespace(&mut self) -> Result<Statement, ParseError> {
        let start = self.current.span.start;
        self.bump();

        let (path, _) = self.parse_path()?;

        if self.current.kind != TokenKind::SemiColon {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        let end = self.current.span.end;
        self.bump();

        Ok(Statement::Namespace { path, span: Span { start, end } })
    }

    fn parse_using(&mut self) -> Result<Statement, ParseError> {
        let start = self.current.span.start;
        self.bump();

        let (path, _) = self.parse_path()?;

        if self.current.kind != TokenKind::SemiColon {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        let end = self.current.span.end;
        self.bump();

        Ok(Statement::Using { path, span: Span { start, end } })
    }

    fn parse_path(&mut self) -> Result<(Vec<String>, Span), ParseError> {
        let mut segments = Vec::new();
        let start = self.current.span.start;
        let first = match &self.current.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => return Err(ParseError::UnexpectedToken(self.current.clone())),
        };
        segments.push(first);
        let mut end = self.current.span.end;
        self.bump();

        while self.current.kind == TokenKind::DoubleColon {
            // Check if this is 0-based indexing: path::[index]
            if self.peek().kind == TokenKind::BracketL {
                break;
            }

            self.bump();

            let seg = match &self.current.kind {
                TokenKind::Identifier(n) => n.clone(),
                _ => return Err(ParseError::UnexpectedToken(self.current.clone())),
            };
            segments.push(seg);
            end = self.current.span.end;
            self.bump();
        }

        // HACK: Bootstrap compiler treats 'package' as 'std' in paths
        // if let Some(first) = segments.first_mut() {
        //     if first == "package" {
        //         *first = "std".to_string();
        //     }
        // }

        Ok((segments, Span { start, end }))
    }

    fn parse_block(&mut self) -> Result<Statement, ParseError> {
        let start = self.current.span.start;
        if self.current.kind != TokenKind::BraceL {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        self.bump();

        let mut statements = Vec::new();
        while self.current.kind != TokenKind::BraceR && self.current.kind != TokenKind::EOF {
            statements.push(self.parse_statement()?);
        }

        let end = self.current.span.end;
        if self.current.kind != TokenKind::BraceR {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        self.bump();

        Ok(Statement::Block { statements, span: Span { start, end } })
    }

    fn parse_let(&mut self) -> Result<Statement, ParseError> {
        let start = self.current.span.start;
        self.bump(); // consume 'let'

        let name = match &self.current.kind {
            TokenKind::Identifier(n) => n.clone(),
            TokenKind::New => "new".to_string(),
            _ => return Err(ParseError::UnexpectedToken(self.current.clone())),
        };
        self.bump();

        // Optional Type Hint
        let mut type_hint = None;
        if let TokenKind::Colon = self.current.kind {
            self.bump();
            type_hint = Some(self.parse_type()?);
        }

        if self.current.kind != TokenKind::Equal {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        self.bump(); // consume '='

        let value = self.parse_expression(0)?;

        if self.current.kind != TokenKind::SemiColon {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        let end = self.current.span.end;
        self.bump(); // consume ';'

        Ok(Statement::Let { name, type_hint, value, span: Span { start, end } })
    }

    fn parse_return(&mut self) -> Result<Statement, ParseError> {
        let start = self.current.span.start;
        self.bump(); // consume 'return'

        let value = if self.current.kind != TokenKind::SemiColon { Some(self.parse_expression(0)?) } else { None };

        if self.current.kind != TokenKind::SemiColon {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        let end = self.current.span.end;
        self.bump(); // consume ';'

        Ok(Statement::Return { value, span: Span { start, end } })
    }

    fn parse_yield(&mut self) -> Result<Statement, ParseError> {
        let start = self.current.span.start;
        self.bump(); // consume 'yield'

        let value = if self.current.kind != TokenKind::SemiColon { Some(self.parse_expression(0)?) } else { None };

        if self.current.kind != TokenKind::SemiColon {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        let end = self.current.span.end;
        self.bump(); // consume ';'

        Ok(Statement::Yield { value, span: Span { start, end } })
    }

    fn parse_if(&mut self) -> Result<Statement, ParseError> {
        let start = self.current.span.start;
        self.bump(); // consume 'if'

        let condition = self.parse_expression(0)?;
        let then_branch = self.parse_block()?;
        let mut else_branch = None;
        let mut end = then_branch.span().end;

        if let TokenKind::Else = self.current.kind {
            self.bump();
            let else_node = if let TokenKind::If = self.current.kind {
                self.parse_if()? // else if ...
            }
            else {
                self.parse_block()?
            };
            end = else_node.span().end;
            else_branch = Some(Box::new(else_node));
        }

        Ok(Statement::If { condition, then_branch: Box::new(then_branch), else_branch, span: Span { start, end } })
    }

    fn parse_while(&mut self) -> Result<Statement, ParseError> {
        let start = self.current.span.start;
        self.bump(); // consume 'while'

        let condition = self.parse_expression(0)?;
        let body = self.parse_block()?;
        let end = body.span().end;

        Ok(Statement::While { condition, body: Box::new(body), span: Span { start, end } })
    }

    fn parse_class(&mut self) -> Result<Statement, ParseError> {
        let start = self.current.span.start;
        self.bump(); // consume 'class'

        let name = match &self.current.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => return Err(ParseError::UnexpectedToken(self.current.clone())),
        };
        self.bump();

        let generics = self.parse_generic_params()?;

        // Optional inheritance: (Base1, alias: Base2)
        let mut parents = Vec::new();
        if self.current.kind == TokenKind::ParenL {
            self.bump();
            while self.current.kind != TokenKind::ParenR && self.current.kind != TokenKind::EOF {
                let mut alias = None;
                let type_expr = self.parse_type()?;

                // Check if it was actually an alias: alias: Type
                if self.current.kind == TokenKind::Colon {
                    if let TypeExpression::Name { name: n, .. } = type_expr {
                        alias = Some(n);
                        self.bump(); // consume ':'
                                     // type_expr is consumed, parse actual type
                        let real_type = self.parse_type()?;
                        parents.push((alias, real_type));
                    }
                    else {
                        return Err(ParseError::UnexpectedToken(self.current.clone()));
                    }
                }
                else {
                    parents.push((alias, type_expr));
                }

                if self.current.kind == TokenKind::Comma {
                    self.bump();
                }
                else {
                    break;
                }
            }
            if self.current.kind != TokenKind::ParenR {
                return Err(ParseError::UnexpectedToken(self.current.clone()));
            }
            self.bump();
        }

        // Optional traits: : Trait1 + Trait2
        let mut traits = Vec::new();
        if self.current.kind == TokenKind::Colon {
            self.bump();
            loop {
                traits.push(self.parse_type()?);
                if self.current.kind == TokenKind::Plus {
                    self.bump();
                }
                else {
                    break;
                }
            }
        }

        if self.current.kind != TokenKind::BraceL {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        self.bump();

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while self.current.kind != TokenKind::BraceR && self.current.kind != TokenKind::EOF {
            if self.current.kind == TokenKind::Micro {
                methods.push(self.parse_function(false)?);
            }
            else if self.current.kind == TokenKind::Async {
                self.bump();
                if self.current.kind == TokenKind::Micro {
                    methods.push(self.parse_function(true)?);
                }
                else {
                    return Err(ParseError::UnexpectedToken(self.current.clone()));
                }
            }
            else if self.current.kind == TokenKind::Override {
                // Handle manual trait resolution: override trait_method() { ... }
                self.bump();
                let is_async = if self.current.kind == TokenKind::Async {
                    self.bump();
                    true
                }
                else {
                    false
                };
                methods.push(self.parse_function(is_async)?);
            }
            else if let TokenKind::Identifier(fname) = &self.current.kind {
                let fname = fname.clone();
                self.bump();
                if self.current.kind != TokenKind::Colon {
                    return Err(ParseError::UnexpectedToken(self.current.clone()));
                }
                self.bump();
                let ftype = self.parse_type()?;
                fields.push((fname, ftype));

                if self.current.kind == TokenKind::Comma {
                    self.bump();
                }
            }
            else {
                return Err(ParseError::UnexpectedToken(self.current.clone()));
            }
        }

        let end = self.current.span.end;
        if self.current.kind != TokenKind::BraceR {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        self.bump();

        Ok(Statement::Class { name, generics, parents, traits, fields, methods, span: Span { start, end } })
    }

    fn parse_trait(&mut self) -> Result<Statement, ParseError> {
        let start = self.current.span.start;
        self.bump(); // consume 'trait'

        let name = match &self.current.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => return Err(ParseError::UnexpectedToken(self.current.clone())),
        };
        self.bump();

        let generics = self.parse_generic_params()?;

        // Optional parents: (Parent1, Parent2) or : Parent1, Parent2
        let mut parents = Vec::new();
        if self.current.kind == TokenKind::ParenL {
            self.bump();
            while self.current.kind != TokenKind::ParenR && self.current.kind != TokenKind::EOF {
                parents.push(self.parse_type()?);
                if self.current.kind == TokenKind::Comma {
                    self.bump();
                } else {
                    break;
                }
            }
            if self.current.kind != TokenKind::ParenR {
                return Err(ParseError::UnexpectedToken(self.current.clone()));
            }
            self.bump();
        }

        if self.current.kind == TokenKind::Colon {
            self.bump();
            loop {
                parents.push(self.parse_type()?);
                if self.current.kind == TokenKind::Plus {
                    self.bump();
                }
                else {
                    break;
                }
            }
        }

        if self.current.kind != TokenKind::BraceL {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        self.bump();

        let mut methods = Vec::new();

        while self.current.kind != TokenKind::BraceR && self.current.kind != TokenKind::EOF {
            if self.current.kind == TokenKind::Micro {
                methods.push(self.parse_function(false)?);
            }
            else if self.current.kind == TokenKind::Async {
                self.bump();
                if self.current.kind == TokenKind::Micro {
                    methods.push(self.parse_function(true)?);
                }
                else {
                    return Err(ParseError::UnexpectedToken(self.current.clone()));
                }
            }
            else {
                return Err(ParseError::UnexpectedToken(self.current.clone()));
            }
        }

        let end = self.current.span.end;
        if self.current.kind != TokenKind::BraceR {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        self.bump();

        Ok(Statement::Trait { name, generics, parents, methods, span: Span { start, end } })
    }

    fn parse_imply(&mut self) -> Result<Statement, ParseError> {
        let start = self.current.span.start;
        self.bump(); // consume 'imply'

        let generics = self.parse_generic_params()?;

        let target = self.parse_type()?;

        let mut trait_target = None;
        // Check for 'for' or ':' or just implies methods on type?
        // Assuming `imply Type { ... }` adds methods to Type.
        // `imply Type : Trait { ... }` implements Trait for Type.
        if self.current.kind == TokenKind::Colon {
            self.bump();
            trait_target = Some(self.parse_type()?);
        }

        if self.current.kind != TokenKind::BraceL {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        self.bump();

        let mut methods = Vec::new();
        while self.current.kind != TokenKind::BraceR && self.current.kind != TokenKind::EOF {
            if self.current.kind == TokenKind::Micro {
                methods.push(self.parse_function(false)?);
            }
            else if self.current.kind == TokenKind::Async {
                self.bump();
                if self.current.kind == TokenKind::Micro {
                    methods.push(self.parse_function(true)?);
                }
                else {
                    return Err(ParseError::UnexpectedToken(self.current.clone()));
                }
            }
            else {
                return Err(ParseError::UnexpectedToken(self.current.clone()));
            }
        }

        let end = self.current.span.end;
        if self.current.kind != TokenKind::BraceR {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        self.bump();

        Ok(Statement::Imply { target, generics, trait_target, methods, span: Span { start, end } })
    }

    fn parse_function(&mut self, is_async: bool) -> Result<Statement, ParseError> {
        let start = self.current.span.start;
        // consume 'micro' or assume it was consumed if called from override
        if self.current.kind == TokenKind::Micro {
            self.bump();
        }

        let name = match &self.current.kind {
            TokenKind::Identifier(n) => n.clone(),
            TokenKind::New => "new".to_string(),
            _ => return Err(ParseError::UnexpectedToken(self.current.clone())),
        };
        self.bump();

        let generics = self.parse_generic_params()?;

        if self.current.kind != TokenKind::ParenL {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        self.bump();

        let mut params = Vec::new();
        while self.current.kind != TokenKind::ParenR {
            let p_name = match &self.current.kind {
                TokenKind::Identifier(n) => n.clone(),
                _ => return Err(ParseError::UnexpectedToken(self.current.clone())),
            };
            self.bump();

            let mut p_type = None;
            if let TokenKind::Colon = self.current.kind {
                self.bump();
                p_type = Some(self.parse_type()?);
            }

            params.push((p_name, p_type));

            if let TokenKind::Comma = self.current.kind {
                self.bump();
            }
            else {
                break;
            }
        }

        if self.current.kind != TokenKind::ParenR {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        self.bump();

        let mut return_type = None;
        if let TokenKind::Arrow = self.current.kind {
            self.bump();
            return_type = Some(self.parse_type()?);
        }

        let body;
        let end;
        if self.current.kind == TokenKind::BraceL {
            let b = self.parse_block()?;
            end = b.span().end;
            body = Some(Box::new(b));
        }
        else if self.current.kind == TokenKind::SemiColon {
            end = self.current.span.end;
            self.bump();
            body = None;
        }
        else {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }

        let is_generator = if let Some(b) = &body { has_yield(b) } else { false };

        Ok(Statement::Function { name, generics, params, return_type, body, is_async, is_generator, span: Span { start, end } })
    }

    fn parse_macro(&mut self) -> Result<Statement, ParseError> {
        let start = self.current.span.start;
        if self.current.kind == TokenKind::Macro {
            self.bump();
        }

        let name = match &self.current.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => return Err(ParseError::UnexpectedToken(self.current.clone())),
        };
        self.bump();

        if self.current.kind != TokenKind::ParenL {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        self.bump();

        let mut params = Vec::new();
        while self.current.kind != TokenKind::ParenR {
            let p_name = match &self.current.kind {
                TokenKind::Identifier(n) => n.clone(),
                _ => return Err(ParseError::UnexpectedToken(self.current.clone())),
            };
            self.bump();

            let mut p_type = None;
            if let TokenKind::Colon = self.current.kind {
                self.bump();
                p_type = Some(self.parse_type()?);
            }

            params.push((p_name, p_type));

            if let TokenKind::Comma = self.current.kind {
                self.bump();
            }
            else {
                break;
            }
        }

        if self.current.kind != TokenKind::ParenR {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        self.bump();

        let mut return_type = None;
        if let TokenKind::Arrow = self.current.kind {
            self.bump();
            return_type = Some(self.parse_type()?);
        }

        let body;
        let end;
        if self.current.kind == TokenKind::BraceL {
            let b = self.parse_block()?;
            end = b.span().end;
            body = Box::new(b);
        }
        else {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }

        Ok(Statement::Macro { name, params, return_type, body, span: Span { start, end } })
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.parse_expression(0)?;
        let start = expr.span().start;
        let mut end = expr.span().end;

        if let TokenKind::SemiColon = self.current.kind {
            end = self.current.span.end;
            self.bump();
            Ok(Statement::Expression { expression: expr, span: Span { start, end } })
        }
        else {
             // Treat expression without semicolon as a potential return value if it's the last statement in a block
             // But parse_statement doesn't know if it's the last.
             // We return it as Expression statement. 
             // If it's the last statement in a block, the block parser/compiler should handle it.
             // Wait, if we consume it here without semicolon, `parse_block` needs to know.
             // Currently parse_block calls parse_statement repeatedly.
             // If we don't consume a semicolon, we might run into the next statement or closing brace.
             // If next token is '}', it's the last expression.
             
             // If the next token is NOT '}' and NOT ';', then it's likely a parse error (missing semicolon),
             // unless we support implicit concatenation which we don't.
             // But `parse_expression` consumes the expression.
             
             // So if current is '}', we are good.
             // If current is something else that starts a statement, we missed a semicolon.
             
             if self.current.kind == TokenKind::BraceR || self.current.kind == TokenKind::EOF {
                 Ok(Statement::Return { value: Some(expr), span: Span { start, end } })
             } else {
                 // Missing semicolon
                 // But for now, let's allow it and see if we can just treat it as an expression statement.
                 // The compiler (MIR gen) needs to know not to pop it if it's the last one.
                 // Actually, if we convert it to Statement::Return here, it solves the problem for blocks!
                 // Because Statement::Return generates OpCode::Return (or leaves value on stack).
                 // But wait, Statement::Return usually means explicit return.
                 // Implicit return in Rust: the block evaluates to the expression.
                 // If we convert it to Return, it returns from the *function*.
                 // That is correct for the last expression of a function body.
                 // But what about `if` expressions? `let x = if c { 1 } else { 0 };`
                 // `if` block returns 1.
                 // If we emit `Return`, it returns from the function, which is WRONG for `if`.
                 
                 // So we cannot just convert to `Statement::Return`.
                 // We should keep it as `Statement::Expression`.
                 // And the `Block` compiler should handle "if last statement is Expression, don't pop".
                 
                 // However, we need to allow parsing it without semicolon.
                 Ok(Statement::Expression { expression: expr, span: Span { start, end } })
             }
        }
    }

    fn parse_type(&mut self) -> Result<TypeExpression, ParseError> {
        match &self.current.kind {
            TokenKind::Identifier(_) => {
                let (segments, mut span) = self.parse_path()?;
                let name = segments.join("::");
                let mut type_expr = TypeExpression::Name { name, span };

                if self.current.kind == TokenKind::Less {
                    self.bump(); // consume '<'
                    let mut params = Vec::new();
                    while self.current.kind != TokenKind::Greater && self.current.kind != TokenKind::EOF {
                        params.push(self.parse_type()?);
                        if self.current.kind == TokenKind::Comma {
                            self.bump();
                        } else {
                            break;
                        }
                    }
                    if self.current.kind != TokenKind::Greater {
                        return Err(ParseError::UnexpectedToken(self.current.clone()));
                    }
                    span.end = self.current.span.end;
                    self.bump(); // consume '>'
                    type_expr = TypeExpression::Generic {
                        base: Box::new(type_expr),
                        params,
                        span
                    };
                }

                Ok(type_expr)
            }
            _ => Err(ParseError::UnexpectedToken(self.current.clone())),
        }
    }

    // --- Pratt Parser for Expressions ---

    fn get_precedence(kind: &TokenKind) -> u8 {
        match kind {
            TokenKind::ParenL | TokenKind::Dot | TokenKind::BracketL | TokenKind::DoubleColon => 10,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => 8,
            TokenKind::Plus | TokenKind::Minus => 7,
            TokenKind::EqualEqual
            | TokenKind::NotEqual
            | TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual => 6,
            TokenKind::And => 5,
            TokenKind::Pipe => 4,
            TokenKind::AndAnd => 3,
            TokenKind::PipePipe => 2,
            TokenKind::Equal => 1,
            _ => 0,
        }
    }

    fn parse_expression(&mut self, min_prec: u8) -> Result<Expression, ParseError> {
        let mut left = self.parse_prefix()?;

        while self.current.kind != TokenKind::EOF {
            let prec = Self::get_precedence(&self.current.kind);
            if prec < min_prec {
                break;
            }

            // Handle infix operators and postfix (like function calls)
            if let TokenKind::ParenL = self.current.kind {
                // Call expression (postfix)
                left = self.parse_call(left)?;
            }
            else if let TokenKind::Dot = self.current.kind {
                // Field access or Method call (if followed by parens, but we parse get first)
                self.bump(); // consume '.'

                if let TokenKind::Await = self.current.kind {
                    let end = self.current.span.end;
                    self.bump();
                    let span = Span { start: left.span().start, end };
                    left = Expression::Await { future: Box::new(left), span };
                    continue;
                }

                let name = match &self.current.kind {
                    TokenKind::Identifier(n) => n.clone(),
                    _ => return Err(ParseError::UnexpectedToken(self.current.clone())),
                };
                let end = self.current.span.end;
                self.bump();
                let span = Span { start: left.span().start, end };
                left = Expression::Get { object: Box::new(left), name, span };
            }
            else if let TokenKind::BracketL = self.current.kind {
                // Indexing: left[index]
                self.bump();
                let index = self.parse_expression(0)?;
                if self.current.kind != TokenKind::BracketR {
                    return Err(ParseError::UnexpectedToken(self.current.clone()));
                }
                let end = self.current.span.end;
                self.bump();
                let span = Span { start: left.span().start, end };
                left = Expression::Index { target: Box::new(left), index: Box::new(index), is_zero_based: true, span };
            }
            else if let TokenKind::DoubleColon = self.current.kind {
                // 0-based indexing: left::[index]
                let start = left.span().start;
                self.bump(); // consume ::

                if let TokenKind::BracketL = self.current.kind {
                    self.bump();
                    let index = self.parse_expression(0)?;
                    if self.current.kind != TokenKind::BracketR {
                        return Err(ParseError::UnexpectedToken(self.current.clone()));
                    }
                    let end = self.current.span.end;
                    self.bump();
                    let span = Span { start, end };
                    left = Expression::Index { target: Box::new(left), index: Box::new(index), is_zero_based: true, span };
                }
                else {
                    return Err(ParseError::UnexpectedToken(self.current.clone()));
                }
            }
            else if let Some(op) = Self::token_to_binop(&self.current.kind) {
                // Binary operator
                let _op_token = self.current.clone();
                self.bump();
                let right = self.parse_expression(prec + 1)?;
                let span = Span { start: left.span().start, end: right.span().end };
                left = Expression::BinaryOp { left: Box::new(left), op, right: Box::new(right), span };
            }
            else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expression, ParseError> {
        match &self.current.kind {
            TokenKind::New => self.parse_new_expression(),
            TokenKind::Micro => self.parse_lambda_expression(false),
            TokenKind::Async => {
                self.bump();
                if self.current.kind == TokenKind::Micro {
                    self.parse_lambda_expression(true)
                }
                else {
                    Err(ParseError::UnexpectedToken(self.current.clone()))
                }
            }
            TokenKind::Match => self.parse_match_expression(),
            TokenKind::Integer(value) => {
                let span = self.current.span;
                let val = *value;
                self.bump();
                Ok(Expression::Literal { value: LiteralValue::Int(val), span })
            }
            TokenKind::Float(value) => {
                let span = self.current.span;
                let val = *value; // f64
                                  // Convert f64 to u64 bits for LiteralValue::Float
                self.bump();
                Ok(Expression::Literal { value: LiteralValue::Float(val.to_bits()), span })
            }
            TokenKind::String(value) => {
                let span = self.current.span;
                let val = value.clone();
                self.bump();
                Ok(Expression::Literal { value: LiteralValue::String(val), span })
            }
            TokenKind::True => {
                let span = self.current.span;
                self.bump();
                Ok(Expression::Literal { value: LiteralValue::Bool(true), span })
            }
            TokenKind::False => {
                let span = self.current.span;
                self.bump();
                Ok(Expression::Literal { value: LiteralValue::Bool(false), span })
            }
            TokenKind::Identifier(_) => {
                let (segments, span) = self.parse_path()?;
                let name = segments.join("::");
                Ok(Expression::Identifier { name, span })
            }
            TokenKind::ParenL => {
                self.bump();
                let expr = self.parse_expression(0)?;
                if self.current.kind != TokenKind::ParenR {
                    return Err(ParseError::UnexpectedToken(self.current.clone()));
                }
                self.bump();
                Ok(expr)
            }
            TokenKind::At => self.parse_macro_call(),
            TokenKind::BraceL => {
                let block = self.parse_block()?;
                let span = block.span();
                Ok(Expression::Block { body: Box::new(block), span })
            }
            TokenKind::BracketL => {
                // List literal: [expr, expr, ...]
                let start = self.current.span.start;
                self.bump();
                let mut elements = Vec::new();
                while self.current.kind != TokenKind::BracketR && self.current.kind != TokenKind::EOF {
                    elements.push(self.parse_expression(0)?);
                    if self.current.kind == TokenKind::Comma {
                        self.bump();
                    }
                    else {
                        break;
                    }
                }
                if self.current.kind != TokenKind::BracketR {
                    return Err(ParseError::UnexpectedToken(self.current.clone()));
                }
                let end = self.current.span.end;
                self.bump();
                Ok(Expression::List { elements, span: Span { start, end } })
            }
            TokenKind::Bang => {
                let start = self.current.span.start;
                self.bump();
                let operand = self.parse_expression(7)?; // High precedence (above * /)
                let end = operand.span().end;
                Ok(Expression::UnaryOp { op: UnaryOperator::Not, operand: Box::new(operand), span: Span { start, end } })
            }
            TokenKind::Minus => {
                let start = self.current.span.start;
                self.bump();
                let operand = self.parse_expression(7)?;
                let end = operand.span().end;
                Ok(Expression::UnaryOp { op: UnaryOperator::Neg, operand: Box::new(operand), span: Span { start, end } })
            }
            TokenKind::Pipe => self.parse_lambda_pipe(),
            _ => Err(ParseError::UnexpectedToken(self.current.clone())),
        }
    }

    fn parse_lambda_pipe(&mut self) -> Result<Expression, ParseError> {
        let start = self.current.span.start;
        self.bump(); // consume '|'

        let mut params = Vec::new();
        while self.current.kind != TokenKind::Pipe && self.current.kind != TokenKind::EOF {
            let p_name = match &self.current.kind {
                TokenKind::Identifier(n) => n.clone(),
                _ => return Err(ParseError::UnexpectedToken(self.current.clone())),
            };
            self.bump();

            let mut p_type = None;
            if let TokenKind::Colon = self.current.kind {
                self.bump();
                p_type = Some(self.parse_type()?);
            }
            params.push((p_name, p_type));

            if let TokenKind::Comma = self.current.kind {
                self.bump();
            }
            else {
                break;
            }
        }

        if self.current.kind != TokenKind::Pipe {
             return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        self.bump(); // consume '|'

        // Body can be Block or Expression
        let body_stmt;
        let end;
        if self.current.kind == TokenKind::BraceL {
            let b = self.parse_block()?;
            end = b.span().end;
            body_stmt = b;
        } else {
            let expr = self.parse_expression(0)?;
            end = expr.span().end;
            let start_expr = expr.span().start;
            // Wrap expression in a return statement block?
            // Or just ExpressionStatement.
            // If it's a lambda, it should return the value.
            // So `return expr;` implicit?
            // Yes, `|x| x+1` means `return x+1`.
            body_stmt = Statement::Return { value: Some(expr), span: Span { start: start_expr, end } };
            // Actually wrap in a Block?
            // Statement::Function body is Option<Box<Statement>> (usually Block).
            // But Expression::Lambda body is Box<Statement>.
        }
        
        // Wrap single statement in Block if not already?
        let final_body = if let Statement::Block { .. } = body_stmt {
            Box::new(body_stmt)
        } else {
             let start_body = body_stmt.span().start;
             Box::new(Statement::Block { statements: vec![body_stmt], span: Span { start: start_body, end } })
        };

        let is_generator = has_yield(&final_body);

        Ok(Expression::Lambda { params, body: final_body, is_async: false, is_generator, span: Span { start, end } })
    }

    fn parse_macro_call(&mut self) -> Result<Expression, ParseError> {
        let start = self.current.span.start;
        self.bump(); // consume '@'
        self.parse_macro_call_inner(start)
    }

    fn parse_macro_call_inner(&mut self, start: Position) -> Result<Expression, ParseError> {
        let name = match &self.current.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => return Err(ParseError::UnexpectedToken(self.current.clone())),
        };
        self.bump();

        // Handle @location.line_number
        if name == "location" && self.current.kind == TokenKind::Dot {
            self.bump(); // consume '.'
            let prop = match &self.current.kind {
                TokenKind::Identifier(n) => n.clone(),
                _ => return Err(ParseError::UnexpectedToken(self.current.clone())),
            };
            let end = self.current.span.end;
            self.bump();

            if prop == "line_number" {
                // Expand to literal integer of current line
                return Ok(Expression::Literal { value: LiteralValue::Int(start.line as i64), span: Span { start, end } });
            }
            else {
                return Err(ParseError::UnexpectedToken(self.current.clone()));
            }
        }

        if self.current.kind != TokenKind::ParenL {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        self.bump();

        let mut args = Vec::new();
        while self.current.kind != TokenKind::ParenR && self.current.kind != TokenKind::EOF {
            args.push(self.parse_expression(0)?);
            if let TokenKind::Comma = self.current.kind {
                self.bump();
            }
            else {
                break;
            }
        }

        if self.current.kind != TokenKind::ParenR {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        let end = self.current.span.end;
        self.bump();

        Ok(Expression::MacroCall { name, args, span: Span { start, end } })
    }

    fn parse_annotation(&mut self, start: Position, is_at_style: bool) -> Result<Statement, ParseError> {
        if is_at_style {
            self.bump(); // consume '.'
        } else {
            self.bump(); // consume '↯'
        }

        // Support list syntax: ↯[anno1, anno2]
        if self.current.kind == TokenKind::BracketL {
            self.bump();
            let mut annos = Vec::new();
            while self.current.kind != TokenKind::BracketR && self.current.kind != TokenKind::EOF {
                let a_start = self.current.span.start;
                let name = match &self.current.kind {
                    TokenKind::Identifier(n) => n.clone(),
                    _ => return Err(ParseError::UnexpectedToken(self.current.clone())),
                };
                self.bump();

                let mut args = Vec::new();
                if self.current.kind == TokenKind::ParenL {
                    self.bump();
                    while self.current.kind != TokenKind::ParenR && self.current.kind != TokenKind::EOF {
                        args.push(self.parse_expression(0)?);
                        if self.current.kind == TokenKind::Comma {
                            self.bump();
                        } else {
                            break;
                        }
                    }
                    if self.current.kind != TokenKind::ParenR {
                        return Err(ParseError::UnexpectedToken(self.current.clone()));
                    }
                    self.bump();
                }
                annos.push((a_start, name, args));

                if self.current.kind == TokenKind::Comma {
                    self.bump();
                } else {
                    break;
                }
            }

            if self.current.kind != TokenKind::BracketR {
                return Err(ParseError::UnexpectedToken(self.current.clone()));
            }
            self.bump();

            let mut target = self.parse_statement()?;
            // Wrap in reverse order to maintain nesting: ↯[a, b] target -> a(b(target))
            for (a_start, name, args) in annos.into_iter().rev() {
                let end = target.span().end;
                target = Statement::Annotation { name, args, target: Box::new(target), span: Span { start: a_start, end } };
            }
            return Ok(target);
        }

        let name = match &self.current.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => return Err(ParseError::UnexpectedToken(self.current.clone())),
        };
        self.bump();

        let mut args = Vec::new();
        if self.current.kind == TokenKind::ParenL {
            self.bump();
            while self.current.kind != TokenKind::ParenR && self.current.kind != TokenKind::EOF {
                args.push(self.parse_expression(0)?);
                if self.current.kind == TokenKind::Comma {
                    self.bump();
                }
                else {
                    break;
                }
            }
            if self.current.kind != TokenKind::ParenR {
                return Err(ParseError::UnexpectedToken(self.current.clone()));
            }
            self.bump();
        }

        let target = self.parse_statement()?;
        let end = target.span().end;

        Ok(Statement::Annotation { name, args, target: Box::new(target), span: Span { start, end } })
    }

    fn parse_new_expression(&mut self) -> Result<Expression, ParseError> {
        let start = self.current.span.start;
        self.bump(); // consume 'new'

        let class = self.parse_type()?;

        if self.current.kind != TokenKind::ParenL {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        self.bump();

        let mut args = Vec::new();
        while self.current.kind != TokenKind::ParenR && self.current.kind != TokenKind::EOF {
            args.push(self.parse_expression(0)?);
            if let TokenKind::Comma = self.current.kind {
                self.bump();
            }
            else {
                break;
            }
        }

        if self.current.kind != TokenKind::ParenR {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        self.bump();

        let mut closure = None;
        let mut end = self.current.span.end;

        if self.current.kind == TokenKind::BraceL {
            let body = self.parse_block()?;
            end = body.span().end;
            closure = Some(Box::new(body));
        }

        Ok(Expression::New { class, args, closure, span: Span { start, end } })
    }

    fn parse_lambda_expression(&mut self, is_async: bool) -> Result<Expression, ParseError> {
        let start = self.current.span.start;
        self.bump(); // consume 'micro'

        let mut params = Vec::new();
        if self.current.kind == TokenKind::ParenL {
            self.bump();
            while self.current.kind != TokenKind::ParenR {
                let p_name = match &self.current.kind {
                    TokenKind::Identifier(n) => n.clone(),
                    _ => return Err(ParseError::UnexpectedToken(self.current.clone())),
                };
                self.bump();

                let mut p_type = None;
                if let TokenKind::Colon = self.current.kind {
                    self.bump();
                    p_type = Some(self.parse_type()?);
                }
                params.push((p_name, p_type));

                if let TokenKind::Comma = self.current.kind {
                    self.bump();
                }
                else {
                    break;
                }
            }
            if self.current.kind != TokenKind::ParenR {
                return Err(ParseError::UnexpectedToken(self.current.clone()));
            }
            self.bump();
        }

        if self.current.kind != TokenKind::BraceL {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        let body = self.parse_block()?;
        let end = body.span().end;

        let is_generator = has_yield(&body);

        Ok(Expression::Lambda { params, body: Box::new(body), is_async, is_generator, span: Span { start, end } })
    }

    fn parse_match_expression(&mut self) -> Result<Expression, ParseError> {
        let start = self.current.span.start;
        self.bump();

        let scrutinee = self.parse_expression(0)?;

        if self.current.kind != TokenKind::BraceL {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        self.bump();

        let mut arms: Vec<(MatchPattern, Expression)> = Vec::new();
        while self.current.kind == TokenKind::Case {
            self.bump();
            let pat = self.parse_match_pattern()?;

            if self.current.kind != TokenKind::Colon {
                return Err(ParseError::UnexpectedToken(self.current.clone()));
            }
            self.bump();

            let expr = self.parse_expression(0)?;
            if self.current.kind == TokenKind::SemiColon {
                self.bump();
            }
            arms.push((pat, expr));
        }

        let mut else_arm = None;
        if self.current.kind == TokenKind::Else {
            self.bump();
            if self.current.kind != TokenKind::Colon {
                return Err(ParseError::UnexpectedToken(self.current.clone()));
            }
            self.bump();

            let expr = self.parse_expression(0)?;
            if self.current.kind == TokenKind::SemiColon {
                self.bump();
            }
            else_arm = Some(Box::new(expr));
        }

        let end = self.current.span.end;
        if self.current.kind != TokenKind::BraceR {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        self.bump();

        Ok(Expression::Match { scrutinee: Box::new(scrutinee), arms, else_arm, span: Span { start, end } })
    }

    fn parse_match_pattern(&mut self) -> Result<MatchPattern, ParseError> {
        match &self.current.kind {
            TokenKind::Identifier(name) if name == "_" => {
                self.bump();
                Ok(MatchPattern::Wildcard)
            }
            TokenKind::Integer(v) => {
                let val = *v;
                self.bump();
                Ok(MatchPattern::Literal(LiteralValue::Int(val)))
            }
            TokenKind::Float(v) => {
                let val = v.to_bits();
                self.bump();
                Ok(MatchPattern::Literal(LiteralValue::Float(val)))
            }
            TokenKind::String(v) => {
                let s = v.clone();
                self.bump();
                Ok(MatchPattern::Literal(LiteralValue::String(s)))
            }
            TokenKind::True => {
                self.bump();
                Ok(MatchPattern::Literal(LiteralValue::Bool(true)))
            }
            TokenKind::False => {
                self.bump();
                Ok(MatchPattern::Literal(LiteralValue::Bool(false)))
            }
            _ => Err(ParseError::UnexpectedToken(self.current.clone())),
        }
    }

    fn parse_call(&mut self, callee: Expression) -> Result<Expression, ParseError> {
        // Expect ParenL (already checked in parse_expression loop)
        self.bump();

        let mut args = Vec::new();
        while self.current.kind != TokenKind::ParenR && self.current.kind != TokenKind::EOF {
            args.push(self.parse_expression(0)?);
            if let TokenKind::Comma = self.current.kind {
                self.bump();
            }
            else {
                break;
            }
        }

        let end = self.current.span.end;
        if self.current.kind != TokenKind::ParenR {
            return Err(ParseError::UnexpectedToken(self.current.clone()));
        }
        self.bump();

        let span = Span { start: callee.span().start, end };
        Ok(Expression::Call { callee: Box::new(callee), args, span })
    }

    fn token_to_binop(kind: &TokenKind) -> Option<BinaryOperator> {
        match kind {
            TokenKind::Plus => Some(BinaryOperator::Add),
            TokenKind::Minus => Some(BinaryOperator::Sub),
            TokenKind::Star => Some(BinaryOperator::Mul),
            TokenKind::Slash => Some(BinaryOperator::Div),
            TokenKind::Percent => Some(BinaryOperator::Rem),
            TokenKind::EqualEqual => Some(BinaryOperator::Equal),
            TokenKind::NotEqual => Some(BinaryOperator::NotEqual),
            TokenKind::Less => Some(BinaryOperator::Less),
            TokenKind::LessEqual => Some(BinaryOperator::LessEqual),
            TokenKind::Greater => Some(BinaryOperator::Greater),
            TokenKind::GreaterEqual => Some(BinaryOperator::GreaterEqual),
            TokenKind::Equal => Some(BinaryOperator::Assign),
            TokenKind::AndAnd => Some(BinaryOperator::And),
            TokenKind::PipePipe => Some(BinaryOperator::Or),
            TokenKind::And => Some(BinaryOperator::BitAnd),
            TokenKind::Pipe => Some(BinaryOperator::BitOr),
            _ => None,
        }
    }
}

fn has_yield(stmt: &Statement) -> bool {
    match stmt {
        Statement::Yield { .. } => true,
        Statement::Block { statements, .. } => statements.iter().any(has_yield),
        Statement::If { then_branch, else_branch, .. } => {
            has_yield(then_branch) || else_branch.as_ref().map(|b| has_yield(&**b)).unwrap_or(false)
        }
        _ => false,
    }
}
