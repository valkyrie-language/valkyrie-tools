// Valkyrie Runtime Support
const ValkyrieRuntime = {
  // 命名空间支持
  namespaces: new Map(),
  
  // 创建命名空间
  createNamespace: function(name) {
    if (!this.namespaces.has(name)) {
      this.namespaces.set(name, {});
    }
    return this.namespaces.get(name);
  },
  
  // 获取命名空间
  getNamespace: function(name) {
    return this.namespaces.get(name) || {};
  },
  
  // 类继承支持
  extend: function(child, parent) {
    child.prototype = Object.create(parent.prototype);
    child.prototype.constructor = child;
    child.super = parent;
  },
  
  // 枚举支持
  createEnum: function(values) {
    const enumObj = {};
    values.forEach((value, index) => {
      if (typeof value === "string") {
        enumObj[value] = index;
      } else {
        enumObj[value.name] = value.value;
      }
    });
    return Object.freeze(enumObj);
  }
};

let TokenType = { IDENTIFIER: "IDENTIFIER", INTEGER: "INTEGER", STRING: "STRING", LET: "let", FUNCTION: "function", IF: "if", ELSE: "else", WHILE: "while", FOR: "for", RETURN: "return", TRUE: "true", FALSE: "false", ASSIGN: "=", PLUS: "+", MINUS: "-", MULTIPLY: "*", DIVIDE: "/", EQUAL: "==", NOT_EQUAL: "!=", LESS_THAN: "<", GREATER_THAN: ">", SEMICOLON: ";", COMMA: ",", DOT: ".", LPAREN: "(", RPAREN: ")", LBRACE: "{", RBRACE: "}", EOF: "EOF" };
let token = {  };
return token;
// Function: Lexer
Lexer = function(source) {
  let lexer = {  };
  lexer.source = source;
  lexer.current = 0;
  lexer.line = 1;
  lexer.column = 1;
  return (lexer.current >= lexer.source.length);
  return "";
};

let char = lexer.source[lexer.current];
lexer.current = (lexer.current + 1);
return char;
return "";
return lexer.source[lexer.current];
return ((char >= "0") && (char <= "9"));
return ((((char >= "a") && (char <= "z")) || ((char >= "A") && (char <= "Z"))) || (char == "_"));
return (lexer.isAlpha(char) || lexer.isDigit(char));
return (((char == " ") || (char == "	")) || (char == ""));
let value = "";
lexer.advance();
return Token("ERROR", "Unterminated string", lexer.line, lexer.column);
return Token(TokenType.STRING, value, lexer.line, lexer.column);
let value = "";
return Token(TokenType.INTEGER, value, lexer.line, lexer.column);
let value = "";
return Token(TokenType.LET, value, lexer.line, lexer.column);
return Token(TokenType.FUNCTION, value, lexer.line, lexer.column);
return Token(TokenType.IF, value, lexer.line, lexer.column);
return Token(TokenType.ELSE, value, lexer.line, lexer.column);
return Token(TokenType.WHILE, value, lexer.line, lexer.column);
return Token(TokenType.FOR, value, lexer.line, lexer.column);
return Token(TokenType.RETURN, value, lexer.line, lexer.column);
return Token(TokenType.TRUE, value, lexer.line, lexer.column);
return Token(TokenType.FALSE, value, lexer.line, lexer.column);
return Token(TokenType.IDENTIFIER, value, lexer.line, lexer.column);
return Token(TokenType.EOF, "", lexer.line, lexer.column);
let char = lexer.peek();
return lexer.nextToken();
return lexer.nextToken();
return lexer.scanString();
return lexer.scanNumber();
return lexer.scanIdentifier();
return Token(TokenType.EQUAL, "==", lexer.line, lexer.column);
return Token(TokenType.ASSIGN, "=", lexer.line, lexer.column);
return Token(TokenType.NOT_EQUAL, "!=", lexer.line, lexer.column);
return Token(TokenType.LESS_THAN, "<", lexer.line, lexer.column);
return Token(TokenType.GREATER_THAN, ">", lexer.line, lexer.column);
return Token(TokenType.PLUS, "+", lexer.line, lexer.column);
return Token(TokenType.MINUS, "-", lexer.line, lexer.column);
return Token(TokenType.MULTIPLY, "*", lexer.line, lexer.column);
return Token(TokenType.DIVIDE, "/", lexer.line, lexer.column);
return Token(TokenType.SEMICOLON, ";", lexer.line, lexer.column);
return Token(TokenType.COMMA, ",", lexer.line, lexer.column);
return Token(TokenType.DOT, ".", lexer.line, lexer.column);
return Token(TokenType.LPAREN, "(", lexer.line, lexer.column);
return Token(TokenType.RPAREN, ")", lexer.line, lexer.column);
return Token(TokenType.LBRACE, "{", lexer.line, lexer.column);
return Token(TokenType.RBRACE, "}", lexer.line, lexer.column);
return Token("ERROR", ("Unknown character: " + char), lexer.line, lexer.column);
let tokens = [];
let token = lexer.nextToken();
return tokens;
return tokens;
return lexer;
