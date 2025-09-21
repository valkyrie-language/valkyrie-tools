#!/usr/bin/env node

// 创建合并的编译器文件
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const STAGE0_DIR = path.join(__dirname, 'dist', 'stage-0');
const OUTPUT_FILE = path.join(STAGE0_DIR, 'merged-compiler.js');

function createMergedCompiler() {
    console.log('Creating merged compiler file...');
    
    // 读取所有生成的文件
    const files = ['lexer.js', 'ast.js', 'parser.js', 'codegen.js', 'compiler.js'];
    let mergedContent = '';
    let seenFunctions = new Set();
    let seenVariables = new Set();
    
    // 添加重复函数名的映射
    const functionRenames = {
        'advance': ['advanceLexer', 'advanceParser']
    };
    
    // 添加ES模块导入
    mergedContent += 'import fs from "fs";\n';
    mergedContent += 'import path from "path";\n\n';
    
    // 添加运行时支持
    mergedContent += '// Valkyrie Runtime Support\n';
    mergedContent += 'const ValkyrieRuntime = {\n';
    mergedContent += '  print: console.log,\n';
    mergedContent += '  assert: (condition, message) => {\n';
    mergedContent += '    if (!condition) throw new Error(message || "Assertion failed");\n';
    mergedContent += '  }\n';
    mergedContent += '};\n\n';
    
    // 合并所有文件的内容，但跳过重复的导入和运行时支持
    for (const fileName of files) {
        const filePath = path.join(STAGE0_DIR, fileName);
        if (fs.existsSync(filePath)) {
            console.log(`Merging ${fileName}...`);
            let content = fs.readFileSync(filePath, 'utf-8');
            
            // 移除重复的导入语句和运行时支持
            content = content.replace(/^import.*?;\s*$/gm, '');
            content = content.replace(/\/\/ Valkyrie Runtime Support[\s\S]*?};\s*$/m, '');
            
            // 处理重复的函数定义
            if (fileName === 'parser.js') {
                // 将parser.js中的advance函数重命名为advanceParser
                content = content.replace(/function advance\(/g, 'function advanceParser(');
                content = content.replace(/advance\(parser\)/g, 'advanceParser(parser)');
            }
            
            // 移除重复的ValkyrieCompiler类定义（只保留最后一个）
            if (fileName !== 'compiler.js') {
                content = content.replace(/\/\/ ValkyrieCompiler 类[\s\S]*?export \{ ValkyrieCompiler, compiler \};/m, '');
            }
            
            mergedContent += `// ===== Content from ${fileName} =====\n`;
            mergedContent += content.trim() + '\n\n';
        }
    }
    
    // 写入合并的文件
    fs.writeFileSync(OUTPUT_FILE, mergedContent, 'utf-8');
    console.log(`Merged compiler created: ${OUTPUT_FILE}`);
}

createMergedCompiler();