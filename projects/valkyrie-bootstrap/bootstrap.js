#!/usr/bin/env node

// Valkyrie 语言自举编译器主程序
import { ValkyrieCompiler } from './bootstrap/compiler.js';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// 项目路径配置
const PATHS = {
    bootstrap: path.join(__dirname, 'bootstrap'),
    library: path.join(__dirname, 'library'),
    dist: path.join(__dirname, 'dist'),
    stage0: path.join(__dirname, 'dist', 'stage-0'),
    stage1: path.join(__dirname, 'dist', 'stage-1')
};

// 创建编译器实例
const compiler = new ValkyrieCompiler();

// 日志函数
function log(message) {
    console.log(`[Valkyrie Bootstrap] ${message}`);
}

function error(message) {
    console.error(`[Valkyrie Bootstrap] ERROR: ${message}`);
}

// 确保目录存在
function ensureDir(dirPath) {
    if (!fs.existsSync(dirPath)) {
        fs.mkdirSync(dirPath, { recursive: true });
        log(`Created directory: ${dirPath}`);
    }
}

// 清理目录
function cleanDir(dirPath) {
    if (fs.existsSync(dirPath)) {
        fs.rmSync(dirPath, { recursive: true, force: true });
        log(`Cleaned directory: ${dirPath}`);
    }
}

// 复制目录
function copyDir(src, dest) {
    ensureDir(dest);
    
    const entries = fs.readdirSync(src, { withFileTypes: true });
    
    for (const entry of entries) {
        const srcPath = path.join(src, entry.name);
        const destPath = path.join(dest, entry.name);
        
        if (entry.isDirectory()) {
            copyDir(srcPath, destPath);
        } else {
            fs.copyFileSync(srcPath, destPath);
        }
    }
}

// 编译单个文件
function compileFile(inputPath, outputPath) {
    try {
        const result = compiler.compileFile(inputPath, outputPath);
        
        if (!result.success) {
            error(`Failed to compile ${inputPath}: ${result.error}`);
            return false;
        }
        
        log(`Compiled: ${path.relative(__dirname, inputPath)} -> ${path.relative(__dirname, outputPath)}`);
        return true;
    } catch (err) {
        error(`Exception while compiling ${inputPath}: ${err.message}`);
        return false;
    }
}

// 编译目录中的所有 .valkyrie 文件
function compileDirectory(inputDir, outputDir) {
    log(`Compiling directory: ${path.relative(__dirname, inputDir)} -> ${path.relative(__dirname, outputDir)}`);
    
    ensureDir(outputDir);
    
    const result = compiler.compileDirectory(inputDir, outputDir);
    
    if (!result.success) {
        error(`Failed to compile directory ${inputDir}: ${result.error}`);
        return false;
    }
    
    log(`Compilation completed: ${result.successCount}/${result.totalFiles} files successful`);
    
    if (result.errorCount > 0) {
        error(`${result.errorCount} files failed to compile`);
        
        // 显示错误详情
        for (const fileResult of result.results) {
            if (!fileResult.result.success) {
                error(`  ${fileResult.inputPath}: ${fileResult.result.error}`);
            }
        }
        
        return false;
    }
    
    return true;
}

// 比较两个目录的内容
function compareDirectories(dir1, dir2) {
    log(`Comparing directories: ${path.relative(__dirname, dir1)} vs ${path.relative(__dirname, dir2)}`);
    
    const result = ValkyrieCompiler.compareDirectories(dir1, dir2);
    
    if (result.equal) {
        log(`✅ Directories are identical (${result.fileCount} files)`);
        return true;
    } else {
        error(`❌ Directories differ: ${result.reason}`);
        if (result.file) {
            error(`  Different file: ${result.file}`);
        }
        return false;
    }
}

// 执行自举过程
async function bootstrap() {
    console.log("DEBUG: bootstrap() function called");
    log("Starting Valkyrie language bootstrap process...");
    
    try {
        // 步骤 1: 清理输出目录
        log("Step 1: Cleaning output directories");
        cleanDir(PATHS.dist);
        ensureDir(PATHS.dist);
        
        // 步骤 2: 使用 bootstrap 编译器编译 library 到 stage-0
        log("Step 2: Compiling library with bootstrap compiler to stage-0");
        if (!compileDirectory(PATHS.library, PATHS.stage0)) {
            throw new Error("Stage-0 compilation failed");
        }
        
        // 步骤 3: 使用 stage-0 编译器编译 library 到 stage-1
        log("Step 3: Compiling library with stage-0 compiler to stage-1");
        
        // 创建 stage-0 编译器实例
        // 注意：这里需要动态导入 stage-0 编译的 JavaScript 代码
        // 由于 stage-0 是编译后的 JavaScript，我们需要特殊处理
        
        // 首先检查 stage-0 是否包含所需的编译器文件
        const stage0CompilerPath = path.join(PATHS.stage0, 'compiler.js');
        if (!fs.existsSync(stage0CompilerPath)) {
            throw new Error("Stage-0 compiler not found");
        }
        
        // 动态导入 stage-0 编译器
        const stage0Module = await import(stage0CompilerPath);
        const stage0Compiler = new stage0Module.ValkyrieCompiler();
        
        // 使用 stage-0 编译器编译 library
        const stage1Result = stage0Compiler.compileDirectory(PATHS.library, PATHS.stage1);
        
        if (!stage1Result.success) {
            throw new Error(`Stage-1 compilation failed: ${stage1Result.error}`);
        }
        
        log(`Stage-1 compilation completed: ${stage1Result.successCount}/${stage1Result.totalFiles} files`);
        
        // 步骤 4: 比较 stage-0 和 stage-1
        log("Step 4: Comparing stage-0 and stage-1 outputs");
        if (!compareDirectories(PATHS.stage0, PATHS.stage1)) {
            throw new Error("Bootstrap verification failed: stage-0 and stage-1 differ");
        }
        
        // 步骤 5: 自举成功，替换 bootstrap
        log("Step 5: Bootstrap successful! Updating bootstrap directory");
        
        // 备份当前 bootstrap
        const backupPath = path.join(__dirname, 'bootstrap.backup.' + Date.now());
        log(`Backing up current bootstrap to: ${path.relative(__dirname, backupPath)}`);
        copyDir(PATHS.bootstrap, backupPath);
        
        // 清理并替换 bootstrap
        cleanDir(PATHS.bootstrap);
        copyDir(PATHS.stage1, PATHS.bootstrap);
        
        log("🎉 Bootstrap completed successfully!");
        log("The Valkyrie compiler has successfully bootstrapped itself.");
        
        return true;
        
    } catch (err) {
        error(`Bootstrap failed: ${err.message}`);
        return false;
    }
}

// 命令行接口
function showHelp() {
    console.log(`
Valkyrie Language Bootstrap Compiler

Usage:
  node bootstrap.js [command] [options]

Commands:
  bootstrap, boot    Run the complete bootstrap process
  compile <input>    Compile a single .valkyrie file
  compile-dir <dir>  Compile all .valkyrie files in a directory
  help, -h, --help  Show this help message

Examples:
  node bootstrap.js bootstrap
  node bootstrap.js compile example.valkyrie
  node bootstrap.js compile-dir ./src
`);
}

// 主函数
async function main() {
    console.log("DEBUG: main() function called with args:", process.argv.slice(2));
    const args = process.argv.slice(2);
    
    if (args.length === 0 || args[0] === 'help' || args[0] === '-h' || args[0] === '--help') {
        showHelp();
        return;
    }
    
    const command = args[0];
    console.log("DEBUG: command is:", command);
    
    switch (command) {
        case 'bootstrap':
        case 'boot':
            console.log("DEBUG: calling bootstrap()");
            const success = await bootstrap();
            console.log("DEBUG: bootstrap() returned:", success);
            process.exit(success ? 0 : 1);
            break;
            
        case 'compile':
            if (args.length < 2) {
                error("Please specify input file");
                process.exit(1);
            }
            
            const inputFile = args[1];
            const outputFile = args[2] || inputFile.replace(/\.valkyrie$/, '.js');
            
            if (compileFile(inputFile, outputFile)) {
                log("Compilation successful");
            } else {
                process.exit(1);
            }
            break;
            
        case 'compile-dir':
            if (args.length < 2) {
                error("Please specify input directory");
                process.exit(1);
            }
            
            const inputDir = args[1];
            const outputDir = args[2] || path.join(inputDir, '../compiled');
            
            if (compileDirectory(inputDir, outputDir)) {
                log("Directory compilation successful");
            } else {
                process.exit(1);
            }
            break;
            
        default:
            error(`Unknown command: ${command}`);
            showHelp();
            process.exit(1);
    }
}

// 运行主函数
console.log("DEBUG: Module loaded, checking execution condition");
console.log("DEBUG: import.meta.url =", import.meta.url);
console.log("DEBUG: process.argv[1] =", process.argv[1]);

// 修复 Windows 路径问题：将反斜杠转换为正斜杠
const normalizedArgv1 = process.argv[1].replace(/\\/g, '/');
const expectedUrl = `file:///${normalizedArgv1}`;
console.log("DEBUG: normalized argv[1] =", normalizedArgv1);
console.log("DEBUG: expected URL =", expectedUrl);

if (import.meta.url === expectedUrl) {
    console.log("DEBUG: Condition matched, calling main()");
    main().catch(err => {
        console.error("DEBUG: Error in main():", err);
        error(`Bootstrap failed: ${err.message}`);
        process.exit(1);
    });
} else {
    console.log("DEBUG: Condition not matched, not calling main()");
}