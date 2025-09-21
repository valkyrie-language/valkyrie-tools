import fs from "fs";
import path from "path";

// Valkyrie Runtime Support
const ValkyrieRuntime = {
  print: console.log,
  assert: (condition, message) => {
    if (!condition) throw new Error(message || "Assertion failed");
  }
};

const ASTNode = {type: "", line: 0, column: 0};
const Program = {type: "Program", statements: [], line: 0, column: 0};
const VariableDeclaration = {type: "VariableDeclaration", name: "", initializer: {}, line: 0, column: 0};
const FunctionDeclaration = {type: "FunctionDeclaration", name: "", parameters: [], body: {}, line: 0, column: 0};
const Parameter = {type: "Parameter", name: "", line: 0, column: 0};
const BlockStatement = {type: "BlockStatement", statements: [], line: 0, column: 0};
const IfStatement = {type: "IfStatement", condition: {}, thenBranch: {}, elseBranch: {}, line: 0, column: 0};
const ExpressionStatement = {type: "ExpressionStatement", expression: {}, line: 0, column: 0};
const AssignmentExpression = {type: "AssignmentExpression", left: {}, right: {}, line: 0, column: 0};
const BinaryExpression = {type: "BinaryExpression", left: {}, operator: "", right: {}, line: 0, column: 0};
const UnaryExpression = {type: "UnaryExpression", operator: "", operand: {}, line: 0, column: 0};
const CallExpression = {type: "CallExpression", callee: {}, arguments: [], line: 0, column: 0};
const Identifier = {type: "Identifier", name: "", line: 0, column: 0};
const NumberLiteral = {type: "NumberLiteral", value: 0, line: 0, column: 0};
const StringLiteral = {type: "StringLiteral", value: "", line: 0, column: 0};
const BooleanLiteral = {type: "BooleanLiteral", value: false, line: 0, column: 0};
const ObjectLiteral = {type: "ObjectLiteral", properties: [], line: 0, column: 0};
function createProgram(statements, line, column) {
  const node = {};
  node.type = "Program";
  node.statements = statements;
  node.line = line;
  node.column = column;
  return node;
}

function createVariableDeclaration(name, initializer, line, column) {
  const node = {};
  node.name = name;
  node.initializer = initializer;
  node.line = line;
  node.column = column;
  return node;
}

function createFunctionDeclaration(name, parameters, body, line, column) {
  const node = {};
  node.name = name;
  node.parameters = parameters;
  node.body = body;
  node.line = line;
  node.column = column;
  return node;
}

function createParameter(name, line, column) {
  const node = {};
  node.name = name;
  node.line = line;
  node.column = column;
  return node;
}

function createBlockStatement(statements, line, column) {
  const node = {};
  node.statements = statements;
  node.line = line;
  node.column = column;
  return node;
}

function createIfStatement(condition, thenBranch, elseBranch, line, column) {
  const node = {};
  node.condition = condition;
  node.thenBranch = thenBranch;
  node.elseBranch = elseBranch;
  node.line = line;
  node.column = column;
  return node;
}

function createExpressionStatement(expression, line, column) {
  const node = {};
  node.expression = expression;
  node.line = line;
  node.column = column;
  return node;
}

function createAssignmentExpression(left, right, line, column) {
  const node = {};
  node.type = "AssignmentExpression";
  node.left = left;
  node.right = right;
  node.line = line;
  node.column = column;
  return node;
}

function createBinaryExpression(left, operator, right, line, column) {
  const node = {};
  node.left = left;
  node.operator = operator;
  node.right = right;
  node.line = line;
  node.column = column;
  return node;
}

function createUnaryExpression(operator, operand, line, column) {
  const node = {};
  node.operator = operator;
  node.operand = operand;
  node.line = line;
  node.column = column;
  return node;
}

function createCallExpression(callee, args, line, column) {
  const node = {};
  node.callee = callee;
  node.arguments = args;
  node.line = line;
  node.column = column;
  return node;
}

function createIdentifier(name, line, column) {
  const node = {};
  node.name = name;
  node.line = line;
  node.column = column;
  return node;
}

function createNumberLiteral(value, line, column) {
  const node = {};
  node.value = value;
  node.line = line;
  node.column = column;
  return node;
}

function createStringLiteral(value, line, column) {
  const node = {};
  node.value = value;
  node.line = line;
  node.column = column;
  return node;
}

function createBooleanLiteral(value, line, column) {
  const node = {};
  node.value = value;
  node.line = line;
  node.column = column;
  return node;
}

function createObjectLiteral(properties, line, column) {
  const node = {};
  node.properties = properties;
  node.line = line;
  node.column = column;
  return node;
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
export { ValkyrieCompiler, compiler, createProgram, createVariableDeclaration, createFunctionDeclaration, createIfStatement, createBlockStatement, createExpressionStatement, createAssignmentExpression, createBinaryExpression, createUnaryExpression, createCallExpression, createIdentifier, createNumberLiteral, createStringLiteral, createBooleanLiteral };
