import fs from "fs";
import path from "path";

// Valkyrie Runtime Support
const ValkyrieRuntime = {
  print: console.log,
  assert: (condition, message) => {
    if (!condition) throw new Error(message || "Assertion failed");
  }
};

const Token = {type: "", value: "", line: 0, column: 0};
const TokenType = {LET: "LET", MICRO: "MICRO", IF: "IF", ELSE: "ELSE", IDENTIFIER: "IDENTIFIER", NUMBER: "NUMBER", STRING: "STRING", BOOLEAN: "BOOLEAN", ASSIGN: "ASSIGN", PLUS: "PLUS", MINUS: "MINUS", MULTIPLY: "MULTIPLY", DIVIDE: "DIVIDE", EQUAL: "EQUAL", NOT_EQUAL: "NOT_EQUAL", LESS: "LESS", GREATER: "GREATER", LPAREN: "LPAREN", RPAREN: "RPAREN", LBRACE: "LBRACE", RBRACE: "RBRACE", LBRACKET: "LBRACKET", RBRACKET: "RBRACKET", COMMA: "COMMA", SEMICOLON: "SEMICOLON", COLON: "COLON", ARROW: "ARROW", EOF: "EOF", COMMENT: "COMMENT"};
const Lexer = {source: "", position: 0, line: 1, column: 1, tokens: [], keywords: {}};
function initLexer(source) {
  const lexer = {};
  lexer.source = source;
  lexer.position = 0;
  lexer.line = 1;
  lexer.column = 1;
  lexer.tokens = [];
  lexer.keywords = {"let": TokenType.LET, "micro": TokenType.MICRO, "if": TokenType.IF, "else": TokenType.ELSE, "true": TokenType.BOOLEAN, "false": TokenType.BOOLEAN};
  return lexer;
}

function current(lexer) {
  if ((lexer.position >= lexer.source.length)) {
    "";
  } else {
    lexer.source[lexer.position];
  }
}

function peek(lexer, offset) {
  const pos = (lexer.position + offset);
  if ((pos >= lexer.source.length)) {
    "";
  } else {
    lexer.source[pos];
  }
}

function advance(lexer) {
  if ((lexer.position < lexer.source.length)) {
    if ((lexer.source[lexer.position] === "\n")) {
      lexer.line = (lexer.line + 1);
      lexer.column = 1;
    } else {
      lexer.column = (lexer.column + 1);
    }
    lexer.position = (lexer.position + 1);
  }
}

function skipWhitespace(lexer) {
  const char = current(lexer);
  if (((((char === " ") || (char === "\t")) || (char === "\r")) || (char === "\n"))) {
    advance(lexer);
    skipWhitespace(lexer);
  } else {
  }
}

function readString(lexer) {
  const startLine = lexer.line;
  const startColumn = lexer.column;
  advance(lexer);
  const value = "";
  const char = current(lexer);
  if (((char !== "") && (char !== "\""))) {
    if ((char === "\\")) {
      advance(lexer);
      const escaped = current(lexer);
      if ((escaped === "n")) {
        value = (value + "\n");
      } else {
        if ((escaped === "t")) {
          value = (value + "\t");
        } else {
          if ((escaped === "r")) {
            value = (value + "\r");
          } else {
            if ((escaped === "\\")) {
              value = (value + "\\");
            } else {
              if ((escaped === "\"")) {
                value = (value + "\"");
              } else {
                value = (value + escaped);
              }
            }
          }
        }
      }
    } else {
      value = (value + char);
    }
    advance(lexer);
    const nextValue = readString(lexer);
    value = (value + nextValue.value);
  }
  advance(lexer);
  const token = {};
  token.type = TokenType.STRING;
  token.value = value;
  token.line = startLine;
  token.column = startColumn;
  return token;
}

function readNumber(lexer) {
  const startLine = lexer.line;
  const startColumn = lexer.column;
  const value = "";
  const char = current(lexer);
  if (((char >= "0") && (char <= "9"))) {
    value = (value + char);
    advance(lexer);
    const nextValue = readNumber(lexer);
    value = (value + nextValue.value);
  }
  if ((current(lexer) === ".")) {
    value = (value + ".");
    advance(lexer);
    const char2 = current(lexer);
    if (((char2 >= "0") && (char2 <= "9"))) {
      value = (value + char2);
      advance(lexer);
      const nextValue2 = readNumber(lexer);
      value = (value + nextValue2.value);
    }
  }
  const token = {};
  token.type = TokenType.NUMBER;
  token.value = value;
  token.line = startLine;
  token.column = startColumn;
  return token;
}

function readIdentifier(lexer) {
  const startLine = lexer.line;
  const startColumn = lexer.column;
  const value = "";
  const char = current(lexer);
  if ((((((char >= "a") && (char <= "z")) || ((char >= "A") && (char <= "Z"))) || (char === "_")) || ((char >= "0") && (char <= "9")))) {
    value = (value + char);
    advance(lexer);
    const nextValue = readIdentifier(lexer);
    value = (value + nextValue.value);
  }
  const tokenType = lexer.keywords[value];
  if ((tokenType === "")) {
    tokenType = TokenType.IDENTIFIER;
  }
  const tokenValue = value;
  if ((tokenType === TokenType.BOOLEAN)) {
    if ((value === "true")) {
      tokenValue = true;
    } else {
      tokenValue = false;
    }
  }
  const token = {};
  token.type = tokenType;
  token.value = tokenValue;
  token.line = startLine;
  token.column = startColumn;
  return token;
}

function readComment(lexer) {
  const startLine = lexer.line;
  const startColumn = lexer.column;
  advance(lexer);
  const value = "";
  const char = current(lexer);
  if (((char !== "") && (char !== "\n"))) {
    value = (value + char);
    advance(lexer);
    const nextValue = readComment(lexer);
    value = (value + nextValue.value);
  }
  const token = {};
  token.type = TokenType.COMMENT;
  token.value = value;
  token.line = startLine;
  token.column = startColumn;
  return token;
}

function createToken(type, value, line, column) {
  const token = {};
  token.type = type;
  token.value = value;
  token.line = line;
  token.column = column;
  return token;
}

function tokenize(lexer) {
  if ((lexer.position < lexer.source.length)) {
    skipWhitespace(lexer);
    const char = current(lexer);
    if ((char !== "")) {
      const line = lexer.line;
      const column = lexer.column;
      if ((char === "#")) {
        const comment = readComment(lexer);
        lexer.tokens = (lexer.tokens + [comment]);
        tokenize(lexer);
      } else {
        if ((char === "\"")) {
          const str = readString(lexer);
          lexer.tokens = (lexer.tokens + [str]);
          tokenize(lexer);
        } else {
          if (((char >= "0") && (char <= "9"))) {
            const num = readNumber(lexer);
            lexer.tokens = (lexer.tokens + [num]);
            tokenize(lexer);
          } else {
            if (((((char >= "a") && (char <= "z")) || ((char >= "A") && (char <= "Z"))) || (char === "_"))) {
              const id = readIdentifier(lexer);
              lexer.tokens = (lexer.tokens + [id]);
              tokenize(lexer);
            } else {
              if (((char === "=") && (peek(lexer, 1) === "="))) {
                const token = createToken(TokenType.EQUAL, "==", line, column);
                lexer.tokens = (lexer.tokens + [token]);
                advance(lexer);
                advance(lexer);
                tokenize(lexer);
              } else {
                if (((char === "!") && (peek(lexer, 1) === "="))) {
                  const token = createToken(TokenType.NOT_EQUAL, "!=", line, column);
                  lexer.tokens = (lexer.tokens + [token]);
                  advance(lexer);
                  advance(lexer);
                  tokenize(lexer);
                } else {
                  if (((char === "-") && (peek(lexer, 1) === ">"))) {
                    const token = createToken(TokenType.ARROW, "->", line, column);
                    lexer.tokens = (lexer.tokens + [token]);
                    advance(lexer);
                    advance(lexer);
                    tokenize(lexer);
                  } else {
                    if ((char === "=")) {
                      const token = createToken(TokenType.ASSIGN, "=", line, column);
                      lexer.tokens = (lexer.tokens + [token]);
                      advance(lexer);
                      tokenize(lexer);
                    } else {
                      if ((char === "+")) {
                        const token = createToken(TokenType.PLUS, "+", line, column);
                        lexer.tokens = (lexer.tokens + [token]);
                        advance(lexer);
                        tokenize(lexer);
                      } else {
                        if ((char === "-")) {
                          const token = createToken(TokenType.MINUS, "-", line, column);
                          lexer.tokens = (lexer.tokens + [token]);
                          advance(lexer);
                          tokenize(lexer);
                        } else {
                          if ((char === "*")) {
                            const token = createToken(TokenType.MULTIPLY, "*", line, column);
                            lexer.tokens = (lexer.tokens + [token]);
                            advance(lexer);
                            tokenize(lexer);
                          } else {
                            if ((char === "/")) {
                              const token = createToken(TokenType.DIVIDE, "/", line, column);
                              lexer.tokens = (lexer.tokens + [token]);
                              advance(lexer);
                              tokenize(lexer);
                            } else {
                              if ((char === "<")) {
                                const token = createToken(TokenType.LESS, "<", line, column);
                                lexer.tokens = (lexer.tokens + [token]);
                                advance(lexer);
                                tokenize(lexer);
                              } else {
                                if ((char === ">")) {
                                  const token = createToken(TokenType.GREATER, ">", line, column);
                                  lexer.tokens = (lexer.tokens + [token]);
                                  advance(lexer);
                                  tokenize(lexer);
                                } else {
                                  if ((char === "(")) {
                                    const token = createToken(TokenType.LPAREN, "(", line, column);
                                    lexer.tokens = (lexer.tokens + [token]);
                                    advance(lexer);
                                    tokenize(lexer);
                                  } else {
                                    if ((char === ")")) {
                                      const token = createToken(TokenType.RPAREN, ")", line, column);
                                      lexer.tokens = (lexer.tokens + [token]);
                                      advance(lexer);
                                      tokenize(lexer);
                                    } else {
                                      if ((char === "{")) {
                                        const token = createToken(TokenType.LBRACE, "{", line, column);
                                        lexer.tokens = (lexer.tokens + [token]);
                                        advance(lexer);
                                        tokenize(lexer);
                                      } else {
                                        if ((char === "}")) {
                                          const token = createToken(TokenType.RBRACE, "}", line, column);
                                          lexer.tokens = (lexer.tokens + [token]);
                                          advance(lexer);
                                          tokenize(lexer);
                                        } else {
                                          if ((char === "[")) {
                                            const token = createToken(TokenType.LBRACKET, "[", line, column);
                                            lexer.tokens = (lexer.tokens + [token]);
                                            advance(lexer);
                                            tokenize(lexer);
                                          } else {
                                            if ((char === "]")) {
                                              const token = createToken(TokenType.RBRACKET, "]", line, column);
                                              lexer.tokens = (lexer.tokens + [token]);
                                              advance(lexer);
                                              tokenize(lexer);
                                            } else {
                                              if ((char === ",")) {
                                                const token = createToken(TokenType.COMMA, ",", line, column);
                                                lexer.tokens = (lexer.tokens + [token]);
                                                advance(lexer);
                                                tokenize(lexer);
                                              } else {
                                                if ((char === ";")) {
                                                  const token = createToken(TokenType.SEMICOLON, ";", line, column);
                                                  lexer.tokens = (lexer.tokens + [token]);
                                                  advance(lexer);
                                                  tokenize(lexer);
                                                } else {
                                                  if ((char === ":")) {
                                                    const token = createToken(TokenType.COLON, ":", line, column);
                                                    lexer.tokens = (lexer.tokens + [token]);
                                                    advance(lexer);
                                                    tokenize(lexer);
                                                  } else {
                                                    advance(lexer);
                                                    tokenize(lexer);
                                                  }
                                                }
                                              }
                                            }
                                          }
                                        }
                                      }
                                    }
                                  }
                                }
                              }
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
  const eofToken = createToken(TokenType.EOF, "", lexer.line, lexer.column);
  lexer.tokens = (lexer.tokens + [eofToken]);
  return lexer.tokens;
}


// ValkyrieCompiler 类
class ValkyrieCompiler {
  compile(source, options = {}) {
    const compiler = initCompiler(source);
    const result = compile(compiler);
    return { success: true, code: result, ast: compiler.ast, tokens: compiler.tokens };
  }
  
  compileFile(filePath, options = {}) {
    const source = fs.readFileSync(filePath, "utf8");
    return this.compile(source, options);
  }
  
  compileDirectory(dirPath, options = {}) {
    const results = [];
    const files = fs.readdirSync(dirPath);
    for (const file of files) {
      if (file.endsWith(".valkyrie")) {
        const filePath = path.join(dirPath, file);
        results.push(this.compileFile(filePath, options));
      }
    }
    return results;
  }
}

// 导出编译器实例
const compiler = new ValkyrieCompiler();
export { ValkyrieCompiler, compiler, initLexer, tokenize };
