// Valkyrie 语言抽象语法树节点定义

export class ASTNode {
    constructor(type, line, column) {
        this.type = type;
        this.line = line;
        this.column = column;
    }
}

// 程序根节点
export class Program extends ASTNode {
    constructor(statements, line, column) {
        super('Program', line, column);
        this.statements = statements;
    }
}

// 变量声明
export class VariableDeclaration extends ASTNode {
    constructor(name, value, mutable = false, typeAnnotation = null, line, column) {
        super('VariableDeclaration', line, column);
        this.name = name;
        this.value = value;
        this.mutable = mutable;
        this.typeAnnotation = typeAnnotation;
    }
}

// 函数声明
export class FunctionDeclaration extends ASTNode {
    constructor(name, parameters, body, returnType = null, line, column) {
        super('FunctionDeclaration', line, column);
        this.name = name;
        this.parameters = parameters;
        this.body = body;
        this.returnType = returnType;
    }
}

// 匿名函数
export class AnonymousFunction extends ASTNode {
    constructor(parameters, body, returnType = null, line, column) {
        super('AnonymousFunction', line, column);
        this.parameters = parameters;
        this.body = body;
        this.returnType = returnType;
    }
}

// 函数参数
export class Parameter extends ASTNode {
    constructor(name, type = null, defaultValue = null, line, column) {
        super('Parameter', line, column);
        this.name = name;
        this.type = type;
        this.defaultValue = defaultValue;
    }
}

// 块语句
export class BlockStatement extends ASTNode {
    constructor(statements, line, column) {
        super('BlockStatement', line, column);
        this.statements = statements;
    }
}

// 表达式语句
export class ExpressionStatement extends ASTNode {
    constructor(expression, line, column) {
        super('ExpressionStatement', line, column);
        this.expression = expression;
    }
}

// 返回语句
export class ReturnStatement extends ASTNode {
    constructor(value = null, line, column) {
        super('ReturnStatement', line, column);
        this.value = value;
    }
}

// If 语句
export class IfStatement extends ASTNode {
    constructor(condition, thenBranch, elseBranch = null, line, column) {
        super('IfStatement', line, column);
        this.condition = condition;
        this.thenBranch = thenBranch;
        this.elseBranch = elseBranch;
    }
}

// 二元表达式
export class BinaryExpression extends ASTNode {
    constructor(left, operator, right, line, column) {
        super('BinaryExpression', line, column);
        this.left = left;
        this.operator = operator;
        this.right = right;
    }
}

// 一元表达式
export class UnaryExpression extends ASTNode {
    constructor(operator, operand, line, column) {
        super('UnaryExpression', line, column);
        this.operator = operator;
        this.operand = operand;
    }
}

// 函数调用
export class CallExpression extends ASTNode {
    constructor(callee, args, line, column) {
        super('CallExpression', line, column);
        this.callee = callee;
        this.arguments = args;
    }
}

// 标识符
export class Identifier extends ASTNode {
    constructor(name, line, column) {
        super('Identifier', line, column);
        this.name = name;
    }
}

// 字面量
export class Literal extends ASTNode {
    constructor(value, line, column) {
        super('Literal', line, column);
        this.value = value;
    }
}

// 数字字面量
export class NumberLiteral extends Literal {
    constructor(value, line, column) {
        super(value, line, column);
        this.type = 'NumberLiteral';
    }
}

// 字符串字面量
export class StringLiteral extends Literal {
    constructor(value, line, column) {
        super(value, line, column);
        this.type = 'StringLiteral';
    }
}

// 布尔字面量
export class BooleanLiteral extends Literal {
    constructor(value, line, column) {
        super(value, line, column);
        this.type = 'BooleanLiteral';
    }
}

// 类型注解
export class TypeAnnotation extends ASTNode {
    constructor(type, line, column) {
        super('TypeAnnotation', line, column);
        this.typeValue = type;
    }
}

// 赋值表达式
export class AssignmentExpression extends ASTNode {
    constructor(left, right, line, column) {
        super('AssignmentExpression', line, column);
        this.left = left;
        this.right = right;
    }
}

// 数组字面量
export class ArrayLiteral extends ASTNode {
    constructor(elements, line, column) {
        super('ArrayLiteral', line, column);
        this.elements = elements;
    }
}

// 成员访问表达式
export class MemberExpression extends ASTNode {
    constructor(object, property, computed = false, line, column) {
        super('MemberExpression', line, column);
        this.object = object;
        this.property = property;
        this.computed = computed; // true for obj[prop], false for obj.prop
    }
}

// 对象字面量
export class ObjectLiteral extends ASTNode {
    constructor(properties, line, column) {
        super('ObjectLiteral', line, column);
        this.properties = properties;
    }
}

// 对象属性
export class Property extends ASTNode {
    constructor(key, value, line, column) {
        super('Property', line, column);
        this.key = key;
        this.value = value;
    }
}