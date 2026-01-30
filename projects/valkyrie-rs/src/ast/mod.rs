pub mod expansion;
pub mod lexer;
pub mod lowering;
pub mod parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveType {
    U32,
    F32,
    String,
    Bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralValue {
    Int(i64),
    Bool(bool),
    Float(u64), // Store as bits for Eq
    String(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Equal,    // ==
    NotEqual, // !=
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Assign, // =
    And,    // &&
    Or,     // ||
    BitAnd, // &
    BitOr,  // |
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Not, // !
    Neg, // -
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProgramNode {
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Namespace {
        path: Vec<String>,
        span: Span,
    },
    Using {
        path: Vec<String>,
        span: Span,
    },
    Block {
        statements: Vec<Statement>,
        span: Span,
    },
    Let {
        is_mutable: bool,
        name: String,
        type_hint: Option<TypeExpression>,
        value: Expression,
        span: Span,
    },
    If {
        condition: Expression,
        then_branch: Box<Statement>, // Usually a Block
        else_branch: Option<Box<Statement>>,
        span: Span,
    },
    Return {
        value: Option<Expression>,
        span: Span,
    },
    Class {
        name: String,
        generics: Vec<String>,
        parents: Vec<(Option<String>, TypeExpression)>, // (alias_name, parent_type)
        traits: Vec<TypeExpression>,                    // Implemented traits
        fields: Vec<(String, TypeExpression)>,
        methods: Vec<Statement>,
        span: Span,
    },
    Trait {
        name: String,
        generics: Vec<String>,
        parents: Vec<TypeExpression>,
        methods: Vec<Statement>,
        span: Span,
    },
    Imply {
        target: TypeExpression,
        generics: Vec<String>,
        trait_target: Option<TypeExpression>,
        methods: Vec<Statement>,
        span: Span,
    },
    Function {
        name: String,
        generics: Vec<String>,
        params: Vec<(String, Option<TypeExpression>)>,
        return_type: Option<TypeExpression>,
        body: Option<Box<Statement>>, // Optional body for trait methods
        is_async: bool,
        is_generator: bool,
        span: Span,
    },
    Yield {
        value: Option<Expression>,
        span: Span,
    },
    While {
        condition: Expression,
        body: Box<Statement>,
        span: Span,
    },
    Macro {
        name: String,
        params: Vec<(String, Option<TypeExpression>)>,
        return_type: Option<TypeExpression>,
        body: Box<Statement>,
        span: Span,
    },
    Annotation {
        name: String,
        args: Vec<Expression>,
        target: Box<Statement>,
        span: Span,
    },
    Expression {
        expression: Expression,
        span: Span,
    },
}

impl Statement {
    pub fn span(&self) -> Span {
        match self {
            Statement::Namespace { span, .. } => *span,
            Statement::Using { span, .. } => *span,
            Statement::Block { span, .. } => *span,
            Statement::Let { span, .. } => *span,
            Statement::If { span, .. } => *span,
            Statement::Return { span, .. } => *span,
            Statement::Class { span, .. } => *span,
            Statement::Trait { span, .. } => *span,
            Statement::Imply { span, .. } => *span,
            Statement::Function { span, .. } => *span,
            Statement::Yield { span, .. } => *span,
            Statement::While { span, .. } => *span,
            Statement::Macro { span, .. } => *span,
            Statement::Annotation { span, .. } => *span,
            Statement::Expression { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier {
        name: String,
        span: Span,
    },
    Literal {
        value: LiteralValue,
        span: Span,
    },
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
        span: Span,
    },
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
        span: Span,
    },
    Call {
        callee: Box<Expression>,
        args: Vec<Expression>,
        span: Span,
    },
    Get {
        object: Box<Expression>,
        name: String,
        span: Span,
    },
    New {
        class: TypeExpression,
        args: Vec<Expression>,
        closure: Option<Box<Statement>>,
        span: Span,
    },
    MacroCall {
        name: String,
        args: Vec<Expression>,
        span: Span,
    },
    Match {
        scrutinee: Box<Expression>,
        arms: Vec<(MatchPattern, Expression)>,
        else_arm: Option<Box<Expression>>,
        span: Span,
    },
    Lambda {
        params: Vec<(String, Option<TypeExpression>)>,
        body: Box<Statement>,
        is_async: bool,
        is_generator: bool,
        span: Span,
    },
    Await {
        future: Box<Expression>,
        span: Span,
    },
    List {
        elements: Vec<Expression>,
        span: Span,
    },
    Index {
        target: Box<Expression>,
        index: Box<Expression>,
        is_zero_based: bool,
        span: Span,
    },
    Block {
        body: Box<Statement>,
        span: Span,
    },
}

impl Expression {
    pub fn span(&self) -> Span {
        match self {
            Expression::Literal { span, .. } => *span,
            Expression::Identifier { span, .. } => *span,
            Expression::BinaryOp { span, .. } => *span,
            Expression::UnaryOp { span, .. } => *span,
            Expression::Call { span, .. } => *span,
            Expression::Get { span, .. } => *span,
            Expression::New { span, .. } => *span,
            Expression::Match { span, .. } => *span,
            Expression::Lambda { span, .. } => *span,
            Expression::MacroCall { span, .. } => *span,
            Expression::Await { span, .. } => *span,
            Expression::List { span, .. } => *span,
            Expression::Index { span, .. } => *span,
            Expression::Block { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchPattern {
    Wildcard,
    Literal(LiteralValue),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpression {
    Name {
        name: String,
        span: Span,
    },
    Generic {
        base: Box<TypeExpression>,
        params: Vec<TypeExpression>,
        span: Span,
    },
}
