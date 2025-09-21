import fs from "fs";
import path from "path";
import { initLexer, tokenize } from './lexer.js';
import { parse } from './parser.js';
import { generate } from './codegen.js';

// Valkyrie Runtime Support
const ValkyrieRuntime = {
  print: console.log,
  assert: (condition, message) => {
    if (!condition) throw new Error(message || "Assertion failed");
  }
};

const Compiler = {source: "", tokens: [], ast: {}, output: ""};
function initCompiler(source) {
  const compiler = {};
  compiler.source = source;
  compiler.tokens = [];
  compiler.ast = {};
  compiler.output = "";
  return compiler;
}

function compile(compiler) {
  try {
    const lexer = initLexer(compiler.source);
    compiler.tokens = tokenize(lexer);
    if ((compiler.tokens.length === 0)) {
      compiler.output = "Error: Lexical analysis failed";
      return compiler;
    } else {
      compiler.ast = parse(compiler.tokens);
      if (!compiler.ast || (compiler.ast.type === "")) {
        compiler.output = "Error: Syntax analysis failed";
        return compiler;
      } else {
        compiler.output = generate(compiler.ast);
        return compiler;
      }
    }
  } catch (error) {
    compiler.output = `Error: ${error.message}`;
    return compiler;
  }
}

const CompilerError = {type: "CompilerError", message: "", line: 0, column: 0};
function createError(message, line, column) {
  const error = {};
  error.message = message;
  error.line = line;
  error.column = column;
  return error;
}

function reportError(error) {
  const errorMessage = ((((("Error at line " + error.line) + ", column ") + error.column) + ": ") + error.message);
  return errorMessage;
}

const CompilerOptions = {outputFormat: "javascript", optimize: false, debug: false};
function createCompilerOptions(outputFormat, optimize, debug) {
  const options = {};
  options.outputFormat = outputFormat;
  options.optimize = optimize;
  options.debug = debug;
  return options;
}

function compileWithOptions(compiler, options) {
  const lexer = initLexer(compiler.source);
  compiler.tokens = tokenize(lexer);
  if ((compiler.tokens.length === 0)) {
    const error = createCompilerError("Lexical analysis failed", 1, 1);
    compiler.output = reportError(error);
    compiler;
  } else {
    compiler.ast = parse(compiler.tokens);
    if ((compiler.ast.type === "")) {
      const error = createCompilerError("Syntax analysis failed", 1, 1);
      compiler.output = reportError(error);
      compiler;
    } else {
      if ((options.outputFormat === "javascript")) {
        compiler.output = generate(compiler.ast);
      } else {
        const error = createCompilerError(("Unsupported output format: " + options.outputFormat), 1, 1);
        compiler.output = reportError(error);
      }
      if (options.debug) {
        compiler.output = ("// Debug mode enabled\n" + compiler.output);
      }
      compiler;
    }
  }
}

const CompilerStats = {tokensCount: 0, astNodesCount: 0, outputSize: 0, compileTime: 0};
function createCompilerStats() {
  const stats = {};
  stats.tokensCount = 0;
  stats.astNodesCount = 0;
  stats.outputSize = 0;
  stats.compileTime = 0;
  return stats;
}

function countASTNodes(node) {
  if ((node.type === "")) {
    0;
  } else {
    if ((node.type === "Program")) {
      const count = 1;
      const i = 0;
      while ((i < node.statements.length)) {
        count = (count + countASTNodes(node.statements[i]));
        i = (i + 1);
      }
      count;
    } else {
      if ((node.type === "BlockStatement")) {
        const count = 1;
        const i = 0;
        while ((i < node.statements.length)) {
          count = (count + countASTNodes(node.statements[i]));
          i = (i + 1);
        }
        count;
      } else {
        if ((node.type === "IfStatement")) {
          const count = 1;
          count = (count + countASTNodes(node.condition));
          count = (count + countASTNodes(node.thenBranch));
          if ((node.elseBranch.type !== "")) {
            count = (count + countASTNodes(node.elseBranch));
          }
          count;
        } else {
          if ((node.type === "BinaryExpression")) {
            const count = 1;
            count = (count + countASTNodes(node.left));
            count = (count + countASTNodes(node.right));
            count;
          } else {
            if ((node.type === "UnaryExpression")) {
              const count = 1;
              count = (count + countASTNodes(node.operand));
              count;
            } else {
              if ((node.type === "CallExpression")) {
                const count = 1;
                count = (count + countASTNodes(node.callee));
                const i = 0;
                while ((i < node.arguments.length)) {
                  count = (count + countASTNodes(node.arguments[i]));
                  i = (i + 1);
                }
                count;
              } else {
                1;
              }
            }
          }
        }
      }
    }
  }
}

function compileWithStats(compiler, options) {
  const stats = createCompilerStats();
  const startTime = 0;
  compileWithOptions(compiler, options);
  stats.tokensCount = compiler.tokens.length;
  stats.astNodesCount = countASTNodes(compiler.ast);
  stats.outputSize = compiler.output.length;
  stats.compileTime = 1;
  if (options.debug) {
    const statsInfo = "\n// Compilation Statistics:\n";
    statsInfo = (((statsInfo + "// Tokens: ") + stats.tokensCount) + "\n");
    statsInfo = (((statsInfo + "// AST Nodes: ") + stats.astNodesCount) + "\n");
    statsInfo = (((statsInfo + "// Output Size: ") + stats.outputSize) + " characters\n");
    statsInfo = (((statsInfo + "// Compile Time: ") + stats.compileTime) + " ms\n");
    compiler.output = (compiler.output + statsInfo);
  }
  return compiler;
}

function compileSource(source) {
  const compiler = initCompiler(source);
  compile(compiler);
  return compiler.output;
}

function compileSourceWithOptions(source, outputFormat, optimize, debug) {
  const compiler = initCompiler(source);
  const options = createCompilerOptions(outputFormat, optimize, debug);
  compileWithStats(compiler, options);
  return compiler.output;
}

function validateSyntax(source) {
  const compiler = initCompiler(source);
  const lexer = initLexer(compiler.source);
  compiler.tokens = tokenize(lexer);
  if ((compiler.tokens.length === 0)) {
    false;
  } else {
    compiler.ast = parse(compiler.tokens);
    (compiler.ast.type !== "");
  }
}


// ValkyrieCompiler 类
class ValkyrieCompiler {
  compile(source, options = {}) {
    const compiler = initCompiler(source);
    const result = compile(compiler);
    
    // 检查编译是否成功
    if (result.output.startsWith("Error:")) {
      return { 
        success: false, 
        error: result.output,
        code: null, 
        ast: result.ast, 
        tokens: result.tokens 
      };
    }
    
    return { 
      success: true, 
      code: result.output, 
      ast: result.ast, 
      tokens: result.tokens 
    };
  }
  
  compileFile(filePath, options = {}) {
    const source = fs.readFileSync(filePath, "utf8");
    return this.compile(source, options);
  }
  
  compileDirectory(dirPath, outputDir) {
    console.log(`[DEBUG] compileDirectory called with dirPath: ${dirPath}, outputDir: ${outputDir}`);
    const results = [];
    let successCount = 0;
    let errorCount = 0;
    
    if (!fs.existsSync(dirPath)) {
      const error = `Input directory does not exist: ${dirPath}`;
      console.log(`[DEBUG] Directory check failed: ${error}`);
      return {
        success: false,
        error: error,
        results: [],
        successCount: 0,
        errorCount: 1,
        totalFiles: 0
      };
    }
    
    // 确保输出目录存在
    if (!fs.existsSync(outputDir)) {
      console.log(`[DEBUG] Creating output directory: ${outputDir}`);
      fs.mkdirSync(outputDir, { recursive: true });
    }
    
    const files = fs.readdirSync(dirPath);
    console.log(`[DEBUG] Found files in directory: ${files.join(', ')}`);
    const valkyrieFiles = files.filter(file => file.endsWith(".valkyrie"));
    console.log(`[DEBUG] Valkyrie files to compile: ${valkyrieFiles.join(', ')}`);
    
    for (const file of valkyrieFiles) {
      const inputPath = path.join(dirPath, file);
      const outputPath = path.join(outputDir, file.replace('.valkyrie', '.js'));
      console.log(`[DEBUG] Compiling ${inputPath} -> ${outputPath}`);
      
      try {
        const compileResult = this.compileFile(inputPath);
        console.log(`[DEBUG] Compile result for ${file}:`, compileResult);
        
        if (compileResult && compileResult.success) {
          // 写入编译结果到输出文件
          fs.writeFileSync(outputPath, compileResult.code, 'utf8');
          successCount++;
          results.push({
            inputPath,
            outputPath,
            result: { success: true, code: compileResult.code }
          });
          console.log(`[DEBUG] Successfully compiled ${file}`);
        } else {
          errorCount++;
          const errorMsg = compileResult ? (compileResult.error || 'Compilation failed') : 'compileFile returned undefined';
          results.push({
            inputPath,
            outputPath,
            result: { success: false, error: errorMsg }
          });
          console.log(`[DEBUG] Failed to compile ${file}: ${errorMsg}`);
        }
      } catch (error) {
        errorCount++;
        console.log(`[DEBUG] Exception while compiling ${file}:`, error);
        results.push({
          inputPath,
          outputPath,
          result: { success: false, error: error.message }
        });
      }
    }
    
    const finalResult = {
      success: errorCount === 0,
      results,
      successCount,
      errorCount,
      totalFiles: valkyrieFiles.length
    };
    
    console.log(`[DEBUG] Final compileDirectory result:`, finalResult);
    return finalResult;
  }
}

// 导出编译器实例
const compiler = new ValkyrieCompiler();
export { ValkyrieCompiler, compiler };
