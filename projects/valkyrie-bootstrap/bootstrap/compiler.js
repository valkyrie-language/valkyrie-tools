// Valkyrie 语言编译器主模块
import { Lexer } from './lexer.js';
import { Parser } from './parser.js';
import { CodeGenerator } from './codegen.js';
import fs from 'fs';
import path from 'path';

export class ValkyrieCompiler {
    constructor() {
        this.lexer = null;
        this.parser = null;
        this.codegen = new CodeGenerator();
    }
    
    compile(source, filename = '<unknown>') {
        try {
            // 词法分析
            this.lexer = new Lexer(source);
            const tokens = this.lexer.tokenize();
            
            // 语法分析
            this.parser = new Parser(tokens);
            const ast = this.parser.parseProgram();
            
            // 代码生成
            const jsCode = this.codegen.generate(ast);
            
            return {
                success: true,
                code: jsCode,
                ast: ast,
                tokens: tokens
            };
        } catch (error) {
            return {
                success: false,
                error: error.message,
                filename: filename
            };
        }
    }
    
    compileFile(inputPath, outputPath = null) {
        try {
            // 读取源文件
            const source = fs.readFileSync(inputPath, 'utf-8');
            
            // 编译
            const result = this.compile(source, inputPath);
            
            if (!result.success) {
                throw new Error(`Compilation failed: ${result.error}`);
            }
            
            // 如果指定了输出路径，写入文件
            if (outputPath) {
                // 确保输出目录存在
                const outputDir = path.dirname(outputPath);
                if (!fs.existsSync(outputDir)) {
                    fs.mkdirSync(outputDir, { recursive: true });
                }
                
                fs.writeFileSync(outputPath, result.code, 'utf-8');
                console.log(`Compiled ${inputPath} -> ${outputPath}`);
            }
            
            return result;
        } catch (error) {
            return {
                success: false,
                error: error.message,
                filename: inputPath
            };
        }
    }
    
    compileDirectory(inputDir, outputDir, extension = '.valkyrie') {
        const results = [];
        
        try {
            // 确保输出目录存在
            if (!fs.existsSync(outputDir)) {
                fs.mkdirSync(outputDir, { recursive: true });
            }
            
            // 递归编译目录中的所有 .valkyrie 文件
            const compileRecursive = (currentInputDir, currentOutputDir) => {
                const entries = fs.readdirSync(currentInputDir, { withFileTypes: true });
                
                for (const entry of entries) {
                    const inputPath = path.join(currentInputDir, entry.name);
                    
                    if (entry.isDirectory()) {
                        const subOutputDir = path.join(currentOutputDir, entry.name);
                        if (!fs.existsSync(subOutputDir)) {
                            fs.mkdirSync(subOutputDir, { recursive: true });
                        }
                        compileRecursive(inputPath, subOutputDir);
                    } else if (entry.isFile() && entry.name.endsWith(extension)) {
                        const baseName = path.basename(entry.name, extension);
                        const outputPath = path.join(currentOutputDir, baseName + '.js');
                        
                        const result = this.compileFile(inputPath, outputPath);
                        results.push({
                            inputPath,
                            outputPath,
                            result
                        });
                    }
                }
            };
            
            compileRecursive(inputDir, outputDir);
            
            return {
                success: true,
                results: results,
                totalFiles: results.length,
                successCount: results.filter(r => r.result.success).length,
                errorCount: results.filter(r => !r.result.success).length
            };
        } catch (error) {
            return {
                success: false,
                error: error.message,
                results: results
            };
        }
    }
    
    // 验证两个目录的编译结果是否相同
    static compareDirectories(dir1, dir2) {
        const getFiles = (dir) => {
            const files = [];
            const traverse = (currentDir, relativePath = '') => {
                const entries = fs.readdirSync(currentDir, { withFileTypes: true });
                for (const entry of entries) {
                    const fullPath = path.join(currentDir, entry.name);
                    const relPath = path.join(relativePath, entry.name);
                    
                    if (entry.isDirectory()) {
                        traverse(fullPath, relPath);
                    } else if (entry.isFile() && entry.name.endsWith('.js')) {
                        files.push({
                            relativePath: relPath,
                            fullPath: fullPath,
                            content: fs.readFileSync(fullPath, 'utf-8')
                        });
                    }
                }
            };
            traverse(dir);
            return files;
        };
        
        const files1 = getFiles(dir1);
        const files2 = getFiles(dir2);
        
        // 检查文件数量
        if (files1.length !== files2.length) {
            return {
                equal: false,
                reason: `Different number of files: ${files1.length} vs ${files2.length}`
            };
        }
        
        // 按相对路径排序
        files1.sort((a, b) => a.relativePath.localeCompare(b.relativePath));
        files2.sort((a, b) => a.relativePath.localeCompare(b.relativePath));
        
        // 逐个比较文件
        for (let i = 0; i < files1.length; i++) {
            const file1 = files1[i];
            const file2 = files2[i];
            
            if (file1.relativePath !== file2.relativePath) {
                return {
                    equal: false,
                    reason: `Different file paths: ${file1.relativePath} vs ${file2.relativePath}`
                };
            }
            
            if (file1.content !== file2.content) {
                return {
                    equal: false,
                    reason: `Different content in file: ${file1.relativePath}`,
                    file: file1.relativePath
                };
            }
        }
        
        return {
            equal: true,
            fileCount: files1.length
        };
    }
}

// 导出默认实例
export const compiler = new ValkyrieCompiler();