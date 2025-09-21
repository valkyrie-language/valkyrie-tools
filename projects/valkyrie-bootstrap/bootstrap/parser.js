import fs from "fs";
import path from "path";
import { createProgram, createVariableDeclaration, createFunctionDeclaration, createIfStatement, createBlockStatement, createExpressionStatement, createAssignmentExpression, createBinaryExpression, createUnaryExpression, createCallExpression, createIdentifier, createNumberLiteral, createStringLiteral, createBooleanLiteral } from './ast.js';

// Valkyrie Runtime Support
const ValkyrieRuntime = {
  print: console.log,
  assert: (condition, message) => {
    if (!condition) throw new Error(message || "Assertion failed");
  }
};

const Parser = {tokens: [], current: 0};
function initParser(tokens) {
  const parser = {};
  parser.tokens = tokens;
  parser.current = 0;
  return parser;
}

function currentToken(parser) {
  if ((parser.current >= parser.tokens.length)) {
    parser.tokens[(parser.tokens.length - 1)];
  } else {
    parser.tokens[parser.current];
  }
}

function peekToken(parser) {
  if (((parser.current + 1) >= parser.tokens.length)) {
    parser.tokens[(parser.tokens.length - 1)];
  } else {
    parser.tokens[(parser.current + 1)];
  }
}

function advance(parser) {
  if ((parser.current < (parser.tokens.length - 1))) {
    parser.current = (parser.current + 1);
  }
  return parser;
}

function check(parser, tokenType) {
  const token = currentToken(parser);
  return (token.type === tokenType);
}

function match(parser, tokenType) {
  if (check(parser, tokenType)) {
    const token = currentToken(parser);
    advance(parser);
    token;
  } else {
  }
}

function expect(parser, tokenType) {
  const token = match(parser, tokenType);
  if ((token === {})) {
    const current = currentToken(parser);
  } else {
    token;
  }
}

function parseProgram(parser) {
  const statements = [];
  while ((!check(parser, "EOF"))) {
    const stmt = parseStatement(parser);
    if ((stmt.type !== "")) {
      statements = (statements + [stmt]);
    }
  }
  return createProgram(statements, 1, 1);
}

function parseStatement(parser) {
  const token = currentToken(parser);
  if ((token.type === "LET")) {
    return parseVariableDeclaration(parser);
  } else {
    if ((token.type === "MICRO")) {
      return parseFunctionDeclaration(parser);
    } else {
      if ((token.type === "IF")) {
        return parseIfStatement(parser);
      } else {
        if ((token.type === "LBRACE")) {
          return parseBlockStatement(parser);
        } else {
          return parseExpressionStatement(parser);
        }
      }
    }
  }
}

function parseVariableDeclaration(parser) {
  const letToken = expect(parser, "LET");
  const nameToken = expect(parser, "IDENTIFIER");
  expect(parser, "ASSIGN");
  const initializer = parseExpression(parser);
  return createVariableDeclaration(nameToken.value, initializer, letToken.line, letToken.column);
}

function parseFunctionDeclaration(parser) {
  const microToken = expect(parser, "MICRO");
  const nameToken = expect(parser, "IDENTIFIER");
  expect(parser, "LPAREN");
  const parameters = [];
  if ((!check(parser, "RPAREN"))) {
    const param = expect(parser, "IDENTIFIER");
    parameters = (parameters + [createParameter(param.value, param.line, param.column)]);
    while (check(parser, "COMMA")) {
      advance(parser);
      param = expect(parser, "IDENTIFIER");
      parameters = (parameters + [createParameter(param.value, param.line, param.column)]);
    }
  }
  expect(parser, "RPAREN");
  const body = parseBlockStatement(parser);
  return createFunctionDeclaration(nameToken.value, parameters, body, microToken.line, microToken.column);
}

function parseIfStatement(parser) {
  const ifToken = expect(parser, "IF");
  const condition = parseExpression(parser);
  const thenBranch = parseBlockStatement(parser);
  const elseBranch = {};
  if (check(parser, "ELSE")) {
    advance(parser);
    if (check(parser, "IF")) {
      elseBranch = parseIfStatement(parser);
    } else {
      elseBranch = parseBlockStatement(parser);
    }
  }
  return createIfStatement(condition, thenBranch, elseBranch, ifToken.line, ifToken.column);
}

function parseBlockStatement(parser) {
  const lbraceToken = expect(parser, "LBRACE");
  const statements = [];
  while (((!check(parser, "RBRACE")) && (!check(parser, "EOF")))) {
    const stmt = parseStatement(parser);
    if ((stmt.type !== "")) {
      statements = (statements + [stmt]);
    }
  }
  expect(parser, "RBRACE");
  return createBlockStatement(statements, lbraceToken.line, lbraceToken.column);
}

function parseExpressionStatement(parser) {
  const expr = parseExpression(parser);
  return createExpressionStatement(expr, expr.line, expr.column);
}

function parseExpression(parser) {
  return parseAssignment(parser);
}

function parseAssignment(parser) {
  const expr = parseLogicalOr(parser);
  if (check(parser, "ASSIGN")) {
    const assignToken = advance(parser);
    const right = parseAssignment(parser);
    createAssignmentExpression(expr, right, assignToken.line, assignToken.column);
  } else {
    expr;
  }
}

function parseLogicalOr(parser) {
  const expr = parseLogicalAnd(parser);
  while (check(parser, "OR")) {
    const operator = advance(parser);
    const right = parseLogicalAnd(parser);
    expr = createBinaryExpression(expr, operator.value, right, operator.line, operator.column);
  }
  return expr;
}

function parseLogicalAnd(parser) {
  const expr = parseEquality(parser);
  while (check(parser, "AND")) {
    const operator = advance(parser);
    const right = parseEquality(parser);
    expr = createBinaryExpression(expr, operator.value, right, operator.line, operator.column);
  }
  return expr;
}

function parseEquality(parser) {
  const expr = parseComparison(parser);
  while ((check(parser, "EQ") || check(parser, "NE"))) {
    const operator = advance(parser);
    const right = parseComparison(parser);
    expr = createBinaryExpression(expr, operator.value, right, operator.line, operator.column);
  }
  return expr;
}

function parseComparison(parser) {
  const expr = parseAddition(parser);
  while ((((check(parser, "LT") || check(parser, "LE")) || check(parser, "GT")) || check(parser, "GE"))) {
    const operator = advance(parser);
    const right = parseAddition(parser);
    expr = createBinaryExpression(expr, operator.value, right, operator.line, operator.column);
  }
  return expr;
}

function parseAddition(parser) {
  const expr = parseMultiplication(parser);
  while ((check(parser, "PLUS") || check(parser, "MINUS"))) {
    const operator = advance(parser);
    const right = parseMultiplication(parser);
    expr = createBinaryExpression(expr, operator.value, right, operator.line, operator.column);
  }
  return expr;
}

function parseMultiplication(parser) {
  const expr = parseUnary(parser);
  while (((check(parser, "MULTIPLY") || check(parser, "DIVIDE")) || check(parser, "MODULO"))) {
    const operator = advance(parser);
    const right = parseUnary(parser);
    expr = createBinaryExpression(expr, operator.value, right, operator.line, operator.column);
  }
  return expr;
}

function parseUnary(parser) {
  if ((check(parser, "NOT") || check(parser, "MINUS"))) {
    const operator = advance(parser);
    const operand = parseUnary(parser);
    createUnaryExpression(operator.value, operand, operator.line, operator.column);
  } else {
    parseCall(parser);
  }
}

function parseCall(parser) {
  const expr = parsePrimary(parser);
  while (check(parser, "LPAREN")) {
    advance(parser);
    const args = [];
    if ((!check(parser, "RPAREN"))) {
      args = (args + [parseExpression(parser)]);
      while (check(parser, "COMMA")) {
        advance(parser);
        args = (args + [parseExpression(parser)]);
      }
    }
    const rparenToken = expect(parser, "RPAREN");
    expr = createCallExpression(expr, args, expr.line, expr.column);
  }
  return expr;
}

function parsePrimary(parser) {
  const token = currentToken(parser);
  if ((token.type === "NUMBER")) {
    advance(parser);
    createNumberLiteral(token.value, token.line, token.column);
  } else {
    if ((token.type === "STRING")) {
      advance(parser);
      createStringLiteral(token.value, token.line, token.column);
    } else {
      if ((token.type === "TRUE")) {
        advance(parser);
        createBooleanLiteral(true, token.line, token.column);
      } else {
        if ((token.type === "FALSE")) {
          advance(parser);
          createBooleanLiteral(false, token.line, token.column);
        } else {
          if ((token.type === "IDENTIFIER")) {
            advance(parser);
            createIdentifier(token.value, token.line, token.column);
          } else {
            if ((token.type === "LBRACE")) {
              advance(parser);
              const properties = [];
              if (check(parser, "RBRACE")) {
                expect(parser, "RBRACE");
                createObjectLiteral([], token.line, token.column);
              } else {
                while (((!check(parser, "RBRACE")) && (!check(parser, "EOF")))) {
                  const keyToken = expect(parser, "IDENTIFIER");
                  expect(parser, "ASSIGN");
                  const value = parseExpression(parser);
                  properties = (properties + [value]);
                  if (check(parser, "COMMA")) {
                    advance(parser);
                  } else {
                    const dummy = 0;
                  }
                }
                expect(parser, "RBRACE");
                createObjectLiteral(properties, token.line, token.column);
              }
            } else {
              if ((token.type === "LPAREN")) {
                advance(parser);
                const expr = parseExpression(parser);
                expect(parser, "RPAREN");
                expr;
              } else {
              }
            }
          }
        }
      }
    }
  }
}

function parse(tokens) {
  const parser = initParser(tokens);
  return parseProgram(parser);
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
export { ValkyrieCompiler, compiler, parse };
