// Valkyrie 语言代码生成器 - 生成 JavaScript 代码
import * as AST from './ast.js';

export class CodeGenerator {
    constructor() {
        this.indentLevel = 0;
        this.output = [];
    }
    
    indent() {
        return '  '.repeat(this.indentLevel);
    }
    
    emit(code) {
        this.output.push(code);
    }
    
    emitLine(code = '') {
        this.output.push(this.indent() + code + '\n');
    }
    
    increaseIndent() {
        this.indentLevel++;
    }
    
    decreaseIndent() {
        this.indentLevel--;
    }
    
    generate(node) {
        switch (node.type) {
            case 'Program':
                return this.generateProgram(node);
            case 'VariableDeclaration':
                return this.generateVariableDeclaration(node);
            case 'FunctionDeclaration':
                return this.generateFunctionDeclaration(node);
            case 'AnonymousFunction':
                return this.generateAnonymousFunction(node);
            case 'BlockStatement':
                return this.generateBlockStatement(node);
            case 'ExpressionStatement':
                return this.generateExpressionStatement(node);
            case 'ReturnStatement':
                return this.generateReturnStatement(node);
            case 'IfStatement':
                return this.generateIfStatement(node);
            case 'WhileStatement':
                return this.generateWhileStatement(node);
            case 'BinaryExpression':
                return this.generateBinaryExpression(node);
            case 'UnaryExpression':
                return this.generateUnaryExpression(node);
            case 'CallExpression':
                return this.generateCallExpression(node);
            case 'AssignmentExpression':
                return this.generateAssignmentExpression(node);
            case 'ArrayLiteral':
                return this.generateArrayLiteral(node);
            case 'MemberExpression':
                return this.generateMemberExpression(node);
            case 'ObjectLiteral':
                return this.generateObjectLiteral(node);
            case 'Property':
                return this.generateProperty(node);
            case 'Identifier':
                return this.generateIdentifier(node);
            case 'NumberLiteral':
                return this.generateNumberLiteral(node);
            case 'StringLiteral':
                return this.generateStringLiteral(node);
            case 'BooleanLiteral':
                return this.generateBooleanLiteral(node);
            default:
                throw new Error(`Unknown node type: ${node.type}`);
        }
    }
    
    generateProgram(node) {
        this.output = [];
        
        // 添加ES模块导入
        this.emitLine('import fs from "fs";');
        this.emitLine('import path from "path";');
        this.emitLine();
        
        // 添加运行时支持
        this.emitLine('// Valkyrie Runtime Support');
        this.emitLine('const ValkyrieRuntime = {');
        this.increaseIndent();
        this.emitLine('print: console.log,');
        this.emitLine('assert: (condition, message) => {');
        this.increaseIndent();
        this.emitLine('if (!condition) throw new Error(message || "Assertion failed");');
        this.decreaseIndent();
        this.emitLine('}');
        this.decreaseIndent();
        this.emitLine('};');
        this.emitLine();
        
        // 生成所有语句
        for (const statement of node.statements) {
            this.generate(statement);
        }
        
        // 添加ValkyrieCompiler类导出
        this.emitLine();
        this.emitLine('// ValkyrieCompiler 类');
        this.emitLine('class ValkyrieCompiler {');
        this.increaseIndent();
        this.emitLine('compile(source, options = {}) {');
        this.increaseIndent();
        this.emitLine('const compiler = initCompiler(source);');
        this.emitLine('const result = compile(compiler);');
        this.emitLine('return { success: true, code: result, ast: compiler.ast, tokens: compiler.tokens };');
        this.decreaseIndent();
        this.emitLine('}');
        this.emitLine();
        this.emitLine('compileFile(filePath, options = {}) {');
        this.increaseIndent();
        this.emitLine('const source = fs.readFileSync(filePath, "utf8");');
        this.emitLine('return this.compile(source, options);');
        this.decreaseIndent();
        this.emitLine('}');
        this.emitLine();
        this.emitLine('compileDirectory(dirPath, options = {}) {');
        this.increaseIndent();
        this.emitLine('const results = [];');
        this.emitLine('const files = fs.readdirSync(dirPath);');
        this.emitLine('for (const file of files) {');
        this.increaseIndent();
        this.emitLine('if (file.endsWith(".valkyrie")) {');
        this.increaseIndent();
        this.emitLine('const filePath = path.join(dirPath, file);');
        this.emitLine('results.push(this.compileFile(filePath, options));');
        this.decreaseIndent();
        this.emitLine('}');
        this.decreaseIndent();
        this.emitLine('}');
        this.emitLine('return results;');
        this.decreaseIndent();
        this.emitLine('}');
        this.decreaseIndent();
        this.emitLine('}');
        this.emitLine();
        this.emitLine('// 导出编译器实例');
        this.emitLine('const compiler = new ValkyrieCompiler();');
        this.emitLine('export { ValkyrieCompiler, compiler };');
        
        return this.output.join('');
    }
    
    generateVariableDeclaration(node) {
        const keyword = node.mutable ? 'let' : 'const';
        const value = this.generate(node.value);
        this.emitLine(`${keyword} ${node.name} = ${value};`);
    }
    
    generateFunctionDeclaration(node) {
        const params = node.parameters.map(param => param.name).join(', ');
        this.emitLine(`function ${node.name}(${params}) {`);
        this.increaseIndent();
        
        // 生成函数体
        for (const statement of node.body.statements) {
            this.generate(statement);
        }
        
        // 如果最后一个语句不是 return，添加隐式返回
        if (node.body.statements.length > 0) {
            const lastStatement = node.body.statements[node.body.statements.length - 1];
            if (lastStatement.type === 'ExpressionStatement') {
                // 将最后一个表达式语句转换为返回语句
                this.output.pop(); // 移除最后生成的语句
                const returnValue = this.generate(lastStatement.expression);
                this.emitLine(`return ${returnValue};`);
            }
        }
        
        this.decreaseIndent();
        this.emitLine('}');
        this.emitLine();
    }
    
    generateAnonymousFunction(node) {
        const params = node.parameters.map(param => param.name).join(', ');
        let result = `(${params}) => {`;
        
        // 如果函数体只有一个表达式语句，生成箭头函数
        if (node.body.statements.length === 1 && 
            node.body.statements[0].type === 'ExpressionStatement') {
            const expr = this.generate(node.body.statements[0].expression);
            result = `(${params}) => ${expr}`;
        } else {
            // 生成完整的函数体
            const oldOutput = this.output;
            const oldIndent = this.indentLevel;
            this.output = [];
            this.indentLevel = 0;
            
            for (const statement of node.body.statements) {
                this.generate(statement);
            }
            
            // 处理隐式返回
            if (node.body.statements.length > 0) {
                const lastStatement = node.body.statements[node.body.statements.length - 1];
                if (lastStatement.type === 'ExpressionStatement') {
                    this.output.pop();
                    const returnValue = this.generate(lastStatement.expression);
                    this.emitLine(`return ${returnValue};`);
                }
            }
            
            const bodyCode = this.output.join('').trim();
            this.output = oldOutput;
            this.indentLevel = oldIndent;
            
            result = `(${params}) => {\n${bodyCode}\n}`;
        }
        
        return result;
    }
    
    generateBlockStatement(node) {
        for (const statement of node.statements) {
            this.generate(statement);
        }
    }
    
    generateExpressionStatement(node) {
        const expr = this.generate(node.expression);
        this.emitLine(`${expr};`);
    }
    
    generateReturnStatement(node) {
        if (node.value) {
            const value = this.generate(node.value);
            this.emitLine(`return ${value};`);
        } else {
            this.emitLine('return;');
        }
    }
    
    generateIfStatement(node) {
        const condition = this.generate(node.condition);
        this.emitLine(`if (${condition}) {`);
        this.increaseIndent();
        this.generate(node.thenBranch);
        this.decreaseIndent();
        
        if (node.elseBranch) {
            this.emitLine('} else {');
            this.increaseIndent();
            this.generate(node.elseBranch);
            this.decreaseIndent();
        }
        
        this.emitLine('}');
    }
    
    generateWhileStatement(node) {
        const condition = this.generate(node.condition);
        this.emitLine(`while (${condition}) {`);
        this.increaseIndent();
        this.generate(node.body);
        this.decreaseIndent();
        this.emitLine('}');
    }
    
    generateBinaryExpression(node) {
        const left = this.generate(node.left);
        const right = this.generate(node.right);
        
        // 处理操作符映射
        const operatorMap = {
            '==': '===',
            '!=': '!=='
        };
        
        const operator = operatorMap[node.operator] || node.operator;
        return `(${left} ${operator} ${right})`;
    }
    
    generateUnaryExpression(node) {
        const operand = this.generate(node.operand);
        return `(${node.operator}${operand})`;
    }
    
    generateCallExpression(node) {
        const callee = this.generate(node.callee);
        const args = node.arguments.map(arg => this.generate(arg)).join(', ');
        return `${callee}(${args})`;
    }
    
    generateAssignmentExpression(node) {
        const left = this.generate(node.left);
        const right = this.generate(node.right);
        return `${left} = ${right}`;
    }
    
    generateIdentifier(node) {
        // 处理内置函数映射
        const builtinMap = {
            'print': 'ValkyrieRuntime.print',
            'assert': 'ValkyrieRuntime.assert'
        };
        
        return builtinMap[node.name] || node.name;
    }
    
    generateNumberLiteral(node) {
        return node.value.toString();
    }
    
    generateStringLiteral(node) {
        // 转义字符串中的特殊字符
        const escaped = node.value
            .replace(/\\/g, '\\\\')
            .replace(/"/g, '\\"')
            .replace(/\n/g, '\\n')
            .replace(/\t/g, '\\t')
            .replace(/\r/g, '\\r');
        return `"${escaped}"`;
    }
    
    generateBooleanLiteral(node) {
        return node.value.toString();
    }
    
    // 生成数组字面量
    generateArrayLiteral(node) {
        const elements = node.elements.map(element => this.generate(element));
        return `[${elements.join(', ')}]`;
    }
    
    // 生成成员访问表达式
    generateMemberExpression(node) {
        const object = this.generate(node.object);
        if (node.computed) {
            // obj[prop]
            const property = this.generate(node.property);
            return `${object}[${property}]`;
        } else {
            // obj.prop
            const property = this.generate(node.property);
            return `${object}.${property}`;
        }
    }
    
    // 生成对象字面量
    generateObjectLiteral(node) {
        if (node.properties.length === 0) {
            return '{}';
        }
        const properties = node.properties.map(prop => this.generate(prop));
        return `{${properties.join(', ')}}`;
    }
    
    // 生成对象属性
    generateProperty(node) {
        const key = this.generate(node.key);
        const value = this.generate(node.value);
        return `${key}: ${value}`;
    }
}