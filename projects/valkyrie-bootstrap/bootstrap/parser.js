// Valkyrie 语言语法分析器
import { TokenType } from './lexer.js';
import * as AST from './ast.js';

export class Parser {
    constructor(tokens) {
        this.tokens = tokens.filter(token => 
            token.type !== TokenType.COMMENT && 
            token.type !== TokenType.NEWLINE
        );
        this.position = 0;
    }
    
    current() {
        if (this.position >= this.tokens.length) {
            return this.tokens[this.tokens.length - 1]; // EOF token
        }
        return this.tokens[this.position];
    }
    
    peek(offset = 1) {
        const pos = this.position + offset;
        if (pos >= this.tokens.length) {
            return this.tokens[this.tokens.length - 1]; // EOF token
        }
        return this.tokens[pos];
    }
    
    advance() {
        if (this.position < this.tokens.length - 1) {
            this.position++;
        }
        return this.current();
    }
    
    match(...types) {
        return types.includes(this.current().type);
    }
    
    check(type) {
        return this.current().type === type;
    }
    
    checkNext(type) {
        return this.peek().type === type;
    }
    
    consume(type, message) {
        if (this.current().type === type) {
            const token = this.current();
            this.advance();
            return token;
        }
        throw new Error(`${message}. Expected ${type}, got ${this.current().type} at line ${this.current().line}`);
    }
    
    // 解析程序
    parseProgram() {
        const statements = [];
        const firstToken = this.current();
        
        while (!this.match(TokenType.EOF)) {
            const stmt = this.parseStatement();
            if (stmt) {
                statements.push(stmt);
            }
        }
        
        return new AST.Program(statements, firstToken.line, firstToken.column);
    }
    
    // 解析语句
    parseStatement() {
        if (this.match(TokenType.LET)) {
            return this.parseVariableDeclaration();
        }
        
        if (this.match(TokenType.MICRO)) {
            return this.parseFunctionDeclaration();
        }
        
        if (this.match(TokenType.IF)) {
            return this.parseIfStatement();
        }
        
        if (this.match(TokenType.LBRACE)) {
            return this.parseBlockStatement();
        }
        
        // 表达式语句（包括赋值语句）
        const expr = this.parseExpression();
        return new AST.ExpressionStatement(expr, expr.line, expr.column);
    }
    
    // 检查是否是赋值语句
    isAssignmentStatement() {
        // 简单标识符赋值: identifier = value
        if (this.check(TokenType.IDENTIFIER) && this.checkNext(TokenType.ASSIGN)) {
            return true;
        }
        
        // 成员表达式赋值: identifier.property = value 或 identifier[index] = value
        if (this.check(TokenType.IDENTIFIER)) {
            let pos = 1;
            let foundMemberAccess = false;
            
            // 跳过可能的成员访问链
            while (this.peek(pos) && 
                   (this.peek(pos).type === TokenType.DOT || this.peek(pos).type === TokenType.LBRACKET)) {
                foundMemberAccess = true;
                
                if (this.peek(pos).type === TokenType.DOT) {
                    pos++; // 跳过 DOT
                    if (this.peek(pos) && this.peek(pos).type === TokenType.IDENTIFIER) {
                        pos++; // 跳过属性名
                    } else {
                        return false;
                    }
                } else if (this.peek(pos).type === TokenType.LBRACKET) {
                    pos++; // 跳过 LBRACKET
                    // 跳过索引表达式直到找到 RBRACKET
                    let bracketCount = 1;
                    while (this.peek(pos) && bracketCount > 0) {
                        if (this.peek(pos).type === TokenType.LBRACKET) {
                            bracketCount++;
                        } else if (this.peek(pos).type === TokenType.RBRACKET) {
                            bracketCount--;
                        }
                        pos++;
                    }
                }
            }
            
            // 如果找到成员访问且以赋值符号结尾，则是赋值语句
            if (foundMemberAccess && this.peek(pos) && this.peek(pos).type === TokenType.ASSIGN) {
                return true;
            }
        }
        
        return false;
    }
    
    // 解析变量声明
    parseVariableDeclaration() {
        const letToken = this.consume(TokenType.LET, "Expected 'let'");
        
        let mutable = false;
        if (this.match(TokenType.MUT)) {
            mutable = true;
            this.advance();
        }
        
        const nameToken = this.consume(TokenType.IDENTIFIER, "Expected variable name");
        
        let typeAnnotation = null;
        if (this.match(TokenType.COLON)) {
            this.advance();
            const typeToken = this.consume(TokenType.IDENTIFIER, "Expected type name");
            typeAnnotation = new AST.TypeAnnotation(typeToken.value, typeToken.line, typeToken.column);
        }
        
        this.consume(TokenType.ASSIGN, "Expected '=' in variable declaration");
        const value = this.parseExpression();
        
        return new AST.VariableDeclaration(
            nameToken.value,
            value,
            mutable,
            typeAnnotation,
            letToken.line,
            letToken.column
        );
    }
    
    // 解析函数声明
    parseFunctionDeclaration() {
        const microToken = this.consume(TokenType.MICRO, "Expected 'micro'");
        const nameToken = this.consume(TokenType.IDENTIFIER, "Expected function name");
        
        this.consume(TokenType.LPAREN, "Expected '(' after function name");
        
        const parameters = [];
        if (!this.match(TokenType.RPAREN)) {
            do {
                const paramName = this.consume(TokenType.IDENTIFIER, "Expected parameter name");
                
                let paramType = null;
                if (this.match(TokenType.COLON)) {
                    this.advance();
                    const typeToken = this.consume(TokenType.IDENTIFIER, "Expected parameter type");
                    paramType = new AST.TypeAnnotation(typeToken.value, typeToken.line, typeToken.column);
                }
                
                parameters.push(new AST.Parameter(
                    paramName.value,
                    paramType,
                    null,
                    paramName.line,
                    paramName.column
                ));
                
                if (this.match(TokenType.COMMA)) {
                    this.advance();
                } else {
                    break;
                }
            } while (!this.match(TokenType.RPAREN));
        }
        
        this.consume(TokenType.RPAREN, "Expected ')' after parameters");
        
        let returnType = null;
        if (this.match(TokenType.ARROW)) {
            this.advance();
            const typeToken = this.consume(TokenType.IDENTIFIER, "Expected return type");
            returnType = new AST.TypeAnnotation(typeToken.value, typeToken.line, typeToken.column);
        }
        
        const body = this.parseBlockStatement();
        
        return new AST.FunctionDeclaration(
            nameToken.value,
            parameters,
            body,
            returnType,
            microToken.line,
            microToken.column
        );
    }
    
    // 解析匿名函数
    parseAnonymousFunction() {
        const microToken = this.consume(TokenType.MICRO, "Expected 'micro'");
        this.consume(TokenType.LPAREN, "Expected '(' after 'micro'");
        
        const parameters = [];
        if (!this.match(TokenType.RPAREN)) {
            do {
                const paramName = this.consume(TokenType.IDENTIFIER, "Expected parameter name");
                
                let paramType = null;
                if (this.match(TokenType.COLON)) {
                    this.advance();
                    const typeToken = this.consume(TokenType.IDENTIFIER, "Expected parameter type");
                    paramType = new AST.TypeAnnotation(typeToken.value, typeToken.line, typeToken.column);
                }
                
                parameters.push(new AST.Parameter(
                    paramName.value,
                    paramType,
                    null,
                    paramName.line,
                    paramName.column
                ));
                
                if (this.match(TokenType.COMMA)) {
                    this.advance();
                } else {
                    break;
                }
            } while (!this.match(TokenType.RPAREN));
        }
        
        this.consume(TokenType.RPAREN, "Expected ')' after parameters");
        
        let returnType = null;
        if (this.match(TokenType.ARROW)) {
            this.advance();
            const typeToken = this.consume(TokenType.IDENTIFIER, "Expected return type");
            returnType = new AST.TypeAnnotation(typeToken.value, typeToken.line, typeToken.column);
        }
        
        const body = this.parseBlockStatement();
        
        return new AST.AnonymousFunction(
            parameters,
            body,
            returnType,
            microToken.line,
            microToken.column
        );
    }
    
    // 解析块语句
    parseBlockStatement() {
        const lbraceToken = this.consume(TokenType.LBRACE, "Expected '{'");
        const statements = [];
        
        while (!this.match(TokenType.RBRACE) && !this.match(TokenType.EOF)) {
            const stmt = this.parseStatement();
            if (stmt) {
                statements.push(stmt);
            }
        }
        
        this.consume(TokenType.RBRACE, "Expected '}'");
        
        return new AST.BlockStatement(statements, lbraceToken.line, lbraceToken.column);
    }
    
    // 解析 If 语句
    parseIfStatement() {
        const ifToken = this.consume(TokenType.IF, "Expected 'if'");
        const condition = this.parseExpression();
        
        // then分支必须是块语句
        const thenBranch = this.parseBlockStatement();
        
        let elseBranch = null;
        if (this.match(TokenType.ELSE)) {
            this.advance();
            if (this.match(TokenType.IF)) {
                // else if 情况
                elseBranch = this.parseIfStatement();
            } else {
                // else 分支必须是块语句
                elseBranch = this.parseBlockStatement();
            }
        }
        
        return new AST.IfStatement(condition, thenBranch, elseBranch, ifToken.line, ifToken.column);
    }
    
    // 解析表达式
    parseExpression() {
        return this.parseAssignment();
    }
    
    // 解析赋值表达式
    parseAssignment() {
        let expr = this.parseLogicalOr();
        
        if (this.match(TokenType.ASSIGN)) {
            console.log(`[DEBUG] Found ASSIGN token at position ${this.position}, current token:`, this.current());
            console.log(`[DEBUG] Left expression:`, expr);
            this.advance(); // 消耗 ASSIGN token
            const right = this.parseAssignment();
            console.log(`[DEBUG] Right expression:`, right);
            return new AST.AssignmentExpression(expr, right, expr.line, expr.column);
        }
        
        return expr;
    }
    
    // 解析逻辑或表达式
    parseLogicalOr() {
        let expr = this.parseLogicalAnd();
        
        while (this.match(TokenType.OR)) {
            const operator = this.current();
            this.advance();
            const right = this.parseLogicalAnd();
            expr = new AST.BinaryExpression(expr, operator.value, right, operator.line, operator.column);
        }
        
        return expr;
    }
    
    // 解析逻辑与表达式
    parseLogicalAnd() {
        let expr = this.parseEquality();
        
        while (this.match(TokenType.AND)) {
            const operator = this.current();
            this.advance();
            const right = this.parseEquality();
            expr = new AST.BinaryExpression(expr, operator.value, right, operator.line, operator.column);
        }
        
        return expr;
    }
    
    // 解析相等性表达式
    parseEquality() {
        let expr = this.parseComparison();
        
        while (this.match(TokenType.EQUAL, TokenType.NOT_EQUAL)) {
            const operator = this.current();
            this.advance();
            const right = this.parseComparison();
            expr = new AST.BinaryExpression(expr, operator.value, right, operator.line, operator.column);
        }
        
        return expr;
    }
    
    // 解析比较表达式
    parseComparison() {
        let expr = this.parseAddition();
        
        while (this.match(TokenType.LESS, TokenType.GREATER, TokenType.LESS_EQUAL, TokenType.GREATER_EQUAL)) {
            const operator = this.current();
            this.advance();
            const right = this.parseAddition();
            expr = new AST.BinaryExpression(expr, operator.value, right, operator.line, operator.column);
        }
        
        return expr;
    }
    
    // 解析加减表达式
    parseAddition() {
        let expr = this.parseMultiplication();
        
        while (this.match(TokenType.PLUS, TokenType.MINUS)) {
            const operator = this.current();
            this.advance();
            const right = this.parseMultiplication();
            expr = new AST.BinaryExpression(expr, operator.value, right, operator.line, operator.column);
        }
        
        return expr;
    }
    
    // 解析乘除表达式
    parseMultiplication() {
        let expr = this.parseUnary();
        
        while (this.match(TokenType.MULTIPLY, TokenType.DIVIDE)) {
            const operator = this.current();
            this.advance();
            const right = this.parseUnary();
            expr = new AST.BinaryExpression(expr, operator.value, right, operator.line, operator.column);
        }
        
        return expr;
    }
    
    // 解析一元表达式
    parseUnary() {
        if (this.match(TokenType.MINUS, TokenType.NOT)) {
            const operator = this.current();
            this.advance();
            const operand = this.parseUnary();
            return new AST.UnaryExpression(operator.value, operand, operator.line, operator.column);
        }
        
        return this.parseCall();
    }
    
    // 解析函数调用
    parseCall() {
        let expr = this.parsePrimary();
        
        while (this.match(TokenType.LPAREN, TokenType.DOT, TokenType.LBRACKET)) {
            if (this.match(TokenType.LPAREN)) {
                // 函数调用
                const lparen = this.current();
                this.advance();
                
                const args = [];
                if (!this.match(TokenType.RPAREN)) {
                    do {
                        args.push(this.parseExpression());
                        if (this.match(TokenType.COMMA)) {
                            this.advance();
                        } else {
                            break;
                        }
                    } while (!this.match(TokenType.RPAREN));
                }
                
                this.consume(TokenType.RPAREN, "Expected ')' after arguments");
                expr = new AST.CallExpression(expr, args, lparen.line, lparen.column);
            } else if (this.match(TokenType.DOT)) {
                // 点号访问 obj.prop
                const dot = this.current();
                this.advance();
                const property = this.consume(TokenType.IDENTIFIER, "Expected property name after '.'");
                expr = new AST.MemberExpression(expr, new AST.Identifier(property.value, property.line, property.column), false, dot.line, dot.column);
            } else if (this.match(TokenType.LBRACKET)) {
                // 数组访问 obj[index]
                console.log(`[DEBUG] Parsing array access, current position: ${this.position}, tokens around:`, 
                    this.tokens.slice(Math.max(0, this.position - 2), this.position + 3).map(t => `${t.type}:${t.value}`));
                const lbracket = this.current();
                this.advance();
                const index = this.parseExpression();
                this.consume(TokenType.RBRACKET, "Expected ']' after array index");
                expr = new AST.MemberExpression(expr, index, true, lbracket.line, lbracket.column);
                console.log(`[DEBUG] Created MemberExpression:`, expr);
            }
        }
        
        return expr;
    }
    
    // 解析基本表达式
    parsePrimary() {
        if (this.match(TokenType.NUMBER)) {
            const token = this.current();
            this.advance();
            return new AST.NumberLiteral(token.value, token.line, token.column);
        }
        
        if (this.match(TokenType.STRING)) {
            const token = this.current();
            this.advance();
            return new AST.StringLiteral(token.value, token.line, token.column);
        }
        
        if (this.match(TokenType.BOOLEAN)) {
            const token = this.current();
            this.advance();
            return new AST.BooleanLiteral(token.value, token.line, token.column);
        }
        
        if (this.match(TokenType.IDENTIFIER)) {
            const token = this.current();
            this.advance();
            return new AST.Identifier(token.value, token.line, token.column);
        }
        
        if (this.match(TokenType.MICRO)) {
            return this.parseAnonymousFunction();
        }
        
        // if表达式
        if (this.match(TokenType.IF)) {
            return this.parseIfExpression();
        }
        
       // 数组字面量
        if (this.match(TokenType.LBRACKET)) {
            return this.parseArrayLiteral();
        }
        
        // 对象字面量
        if (this.match(TokenType.LBRACE)) {
            return this.parseObjectLiteral();
        }
        
        if (this.match(TokenType.LPAREN)) {
            this.advance();
            const expr = this.parseExpression();
            this.consume(TokenType.RPAREN, "Expected ')' after expression");
            return expr;
        }
        
        throw new Error(`Unexpected token ${this.current().type} at line ${this.current().line}`);
    }
    
    // 解析if表达式
    parseIfExpression() {
        const ifToken = this.consume(TokenType.IF, "Expected 'if'");
        const condition = this.parseExpression();
        this.consume(TokenType.LBRACE, "Expected '{' after if condition");
        const thenExpr = this.parseExpression();
        this.consume(TokenType.RBRACE, "Expected '}' after then expression");
        
        this.consume(TokenType.ELSE, "Expected 'else' in if expression");
        this.consume(TokenType.LBRACE, "Expected '{' after else");
        const elseExpr = this.parseExpression();
        this.consume(TokenType.RBRACE, "Expected '}' after else expression");
        
        return new AST.IfExpression(condition, thenExpr, elseExpr, ifToken.line, ifToken.column);
    }

    // 解析数组字面量
    parseArrayLiteral() {
        const startToken = this.current();
        this.consume(TokenType.LBRACKET, "Expected '['");
        
        const elements = [];
        
        if (!this.match(TokenType.RBRACKET)) {
            do {
                elements.push(this.parseExpression());
            } while (this.match(TokenType.COMMA) && this.advance());
        }
        
        this.consume(TokenType.RBRACKET, "Expected ']' after array elements");
        
        return new AST.ArrayLiteral(elements, startToken.line, startToken.column);
    }
    
    // 解析对象字面量
    parseObjectLiteral() {
        const startToken = this.current();
        this.advance(); // 消耗 '{'
        
        const properties = [];
        
        while (!this.match(TokenType.RBRACE) && !this.match(TokenType.EOF)) {
            // 解析属性键
            let key;
            if (this.match(TokenType.IDENTIFIER)) {
                const keyToken = this.current();
                this.advance();
                key = new AST.Identifier(keyToken.value, keyToken.line, keyToken.column);
            } else if (this.match(TokenType.STRING)) {
                const keyToken = this.current();
                this.advance();
                key = new AST.StringLiteral(keyToken.value, keyToken.line, keyToken.column);
            } else {
                throw new Error(`Expected property key at line ${this.current().line}`);
            }
            
            this.consume(TokenType.ASSIGN, "Expected '=' after property key");
            
            // 解析属性值
            const value = this.parseExpression();
            
            properties.push(new AST.Property(key, value, key.line, key.column));
            
            if (this.match(TokenType.COMMA)) {
                this.advance();
            } else {
                break;
            }
        }
        
        this.consume(TokenType.RBRACE, "Expected '}' after object literal");
        
        return new AST.ObjectLiteral(properties, startToken.line, startToken.column);
    }
}