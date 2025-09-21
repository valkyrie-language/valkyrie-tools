// Valkyrie 语言词法分析器
export class Token {
    constructor(type, value, line, column) {
        this.type = type;
        this.value = value;
        this.line = line;
        this.column = column;
    }
}

export const TokenType = {
    // 关键字
    LET: 'LET',
    MUT: 'MUT',
    MICRO: 'MICRO',
    IF: 'IF',
    ELSE: 'ELSE',
    
    // 标识符和字面量
    IDENTIFIER: 'IDENTIFIER',
    NUMBER: 'NUMBER',
    STRING: 'STRING',
    BOOLEAN: 'BOOLEAN',
    
    // 操作符
    ASSIGN: 'ASSIGN',        // =
    PLUS: 'PLUS',            // +
    MINUS: 'MINUS',          // -
    MULTIPLY: 'MULTIPLY',    // *
    DIVIDE: 'DIVIDE',        // /
    EQUAL: 'EQUAL',          // ==
    NOT_EQUAL: 'NOT_EQUAL',  // !=
    LESS: 'LESS',            // <
    GREATER: 'GREATER',      // >
    NOT: 'NOT',              // !
    OR: 'OR',                // ||
    AND: 'AND',              // &&
    DOT: 'DOT',              // .
    
    // 符号
    LPAREN: 'LPAREN',        // (
    RPAREN: 'RPAREN',        // )
    LBRACE: 'LBRACE',        // {
    RBRACE: 'RBRACE',        // }
    LBRACKET: 'LBRACKET',    // [
    RBRACKET: 'RBRACKET',    // ]
    COMMA: 'COMMA',          // ,
    SEMICOLON: 'SEMICOLON',  // ;
    COLON: 'COLON',          // :
    ARROW: 'ARROW',          // ->
    
    // 特殊
    EOF: 'EOF',
    COMMENT: 'COMMENT'
};

export class Lexer {
    constructor(source) {
        this.source = source;
        this.position = 0;
        this.line = 1;
        this.column = 1;
        this.tokens = [];
        
        this.keywords = {
            'let': TokenType.LET,
            'mut': TokenType.MUT,
            'micro': TokenType.MICRO,
            'if': TokenType.IF,
            'else': TokenType.ELSE,
            'true': TokenType.BOOLEAN,
            'false': TokenType.BOOLEAN
        };
    }
    
    current() {
        if (this.position >= this.source.length) {
            return null;
        }
        return this.source[this.position];
    }
    
    peek(offset = 1) {
        const pos = this.position + offset;
        if (pos >= this.source.length) {
            return null;
        }
        return this.source[pos];
    }
    
    advance() {
        if (this.position < this.source.length) {
            if (this.source[this.position] === '\n') {
                this.line++;
                this.column = 1;
            } else {
                this.column++;
            }
            this.position++;
        }
    }
    
    skipWhitespace() {
        while (this.current() && /\s/.test(this.current())) {
            this.advance();
        }
    }
    
    readString() {
        const startLine = this.line;
        const startColumn = this.column;
        this.advance(); // 跳过开始的引号
        
        let value = '';
        while (this.current() && this.current() !== '"') {
            if (this.current() === '\\') {
                this.advance();
                const escaped = this.current();
                switch (escaped) {
                    case 'n': value += '\n'; break;
                    case 't': value += '\t'; break;
                    case 'r': value += '\r'; break;
                    case '\\': value += '\\'; break;
                    case '"': value += '"'; break;
                    default: value += escaped; break;
                }
            } else {
                value += this.current();
            }
            this.advance();
        }
        
        if (this.current() === '"') {
            this.advance(); // 跳过结束的引号
        }
        
        return new Token(TokenType.STRING, value, startLine, startColumn);
    }
    
    readNumber() {
        const startLine = this.line;
        const startColumn = this.column;
        let value = '';
        
        while (this.current() && /\d/.test(this.current())) {
            value += this.current();
            this.advance();
        }
        
        if (this.current() === '.' && this.peek() && /\d/.test(this.peek())) {
            value += this.current();
            this.advance();
            while (this.current() && /\d/.test(this.current())) {
                value += this.current();
                this.advance();
            }
        }
        
        return new Token(TokenType.NUMBER, parseFloat(value), startLine, startColumn);
    }
    
    readIdentifier() {
        const startLine = this.line;
        const startColumn = this.column;
        let value = '';
        
        while (this.current() && /[a-zA-Z0-9_]/.test(this.current())) {
            value += this.current();
            this.advance();
        }
        
        const tokenType = this.keywords[value] || TokenType.IDENTIFIER;
        const tokenValue = tokenType === TokenType.BOOLEAN ? (value === 'true') : value;
        
        return new Token(tokenType, tokenValue, startLine, startColumn);
    }
    
    readComment() {
        const startLine = this.line;
        const startColumn = this.column;
        this.advance(); // 跳过 #
        
        let value = '';
        while (this.current() && this.current() !== '\n') {
            value += this.current();
            this.advance();
        }
        
        return new Token(TokenType.COMMENT, value, startLine, startColumn);
    }
    
    tokenize() {
        while (this.position < this.source.length) {
            this.skipWhitespace();
            
            const char = this.current();
            if (!char) break;
            
            const line = this.line;
            const column = this.column;
            
            // 注释
            if (char === '#') {
                const comment = this.readComment();
                this.tokens.push(comment);
                continue;
            }
            
            // 字符串
            if (char === '"') {
                this.tokens.push(this.readString());
                continue;
            }
            
            // 数字
            if (/\d/.test(char)) {
                this.tokens.push(this.readNumber());
                continue;
            }
            
            // 标识符和关键字
            if (/[a-zA-Z_]/.test(char)) {
                this.tokens.push(this.readIdentifier());
                continue;
            }
            
            // 双字符操作符
            if (char === '=' && this.peek() === '=') {
                this.tokens.push(new Token(TokenType.EQUAL, '==', line, column));
                this.advance();
                this.advance();
                continue;
            }
            
            if (char === '!' && this.peek() === '=') {
                this.tokens.push(new Token(TokenType.NOT_EQUAL, '!=', line, column));
                this.advance();
                this.advance();
                continue;
            }
            
            if (char === '-' && this.peek() === '>') {
                this.tokens.push(new Token(TokenType.ARROW, '->', line, column));
                this.advance();
                this.advance();
                continue;
            }
            
            if (char === '|' && this.peek() === '|') {
                this.tokens.push(new Token(TokenType.OR, '||', line, column));
                this.advance();
                this.advance();
                continue;
            }
            
            if (char === '&' && this.peek() === '&') {
                this.tokens.push(new Token(TokenType.AND, '&&', line, column));
                this.advance();
                this.advance();
                continue;
            }
            
            // 单字符操作符和分隔符
            switch (char) {
                case '=': this.tokens.push(new Token(TokenType.ASSIGN, '=', line, column)); break;
                case '+': this.tokens.push(new Token(TokenType.PLUS, '+', line, column)); break;
                case '-': this.tokens.push(new Token(TokenType.MINUS, '-', line, column)); break;
                case '*': this.tokens.push(new Token(TokenType.MULTIPLY, '*', line, column)); break;
                case '/': this.tokens.push(new Token(TokenType.DIVIDE, '/', line, column)); break;
                case '<': this.tokens.push(new Token(TokenType.LESS, '<', line, column)); break;
                case '>': this.tokens.push(new Token(TokenType.GREATER, '>', line, column)); break;
                case '!': this.tokens.push(new Token(TokenType.NOT, '!', line, column)); break;
                case '.': this.tokens.push(new Token(TokenType.DOT, '.', line, column)); break;
                case '(': this.tokens.push(new Token(TokenType.LPAREN, '(', line, column)); break;
                case ')': this.tokens.push(new Token(TokenType.RPAREN, ')', line, column)); break;
                case '{': this.tokens.push(new Token(TokenType.LBRACE, '{', line, column)); break;
                case '}': this.tokens.push(new Token(TokenType.RBRACE, '}', line, column)); break;
                case '[': this.tokens.push(new Token(TokenType.LBRACKET, '[', line, column)); break;
                case ']': this.tokens.push(new Token(TokenType.RBRACKET, ']', line, column)); break;
                case ',': this.tokens.push(new Token(TokenType.COMMA, ',', line, column)); break;
                case ';': this.tokens.push(new Token(TokenType.SEMICOLON, ';', line, column)); break;
                case ':': this.tokens.push(new Token(TokenType.COLON, ':', line, column)); break;
                default:
                    throw new Error(`Unexpected character '${char}' at line ${line}, column ${column}`);
            }
            
            this.advance();
        }
        
        this.tokens.push(new Token(TokenType.EOF, null, this.line, this.column));
        return this.tokens;
    }
}