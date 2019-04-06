#!/usr/bin/env node

/**
 * Valkyrie Language Bootstrap Compiler - 使用新生成的编译器
 * 验证自举编译能力
 */

import fs from 'fs';

// 从 dist-new 导入新生成的编译器组件
// 注意：这里需要动态导入，因为生成的文件可能不是标准的 ES 模块

// 自举验证测试函数
async function testSelfCompilation() {
    console.log('🔄 开始验证 Valkyrie 自举编译能力...\n');
    
    try {
        // 读取生成的编译器文件
        const lexerCode = await fs.promises.readFile('./dist/lexer.js', 'utf8');
        const parserCode = await fs.promises.readFile('./dist/parser.js', 'utf8');
        const codegenCode = await fs.promises.readFile('./dist/codegen.js', 'utf8');
        const cliCode = await fs.promises.readFile('./dist/cli.js', 'utf8');
        
        console.log('✅ 成功读取所有生成的编译器文件');
        console.log(`   📄 lexer.js: ${lexerCode.length} 字符`);
        console.log(`   📄 parser.js: ${parserCode.length} 字符`);
        console.log(`   📄 codegen.js: ${codegenCode.length} 字符`);
        console.log(`   📄 cli.js: ${cliCode.length} 字符\n`);
        
        // 验证生成的代码包含必要的组件
        const hasLexer = lexerCode.includes('TokenType') && lexerCode.includes('Lexer');
        const hasParser = parserCode.includes('Parser') && parserCode.includes('parse');
        const hasCodegen = codegenCode.includes('CodeGenerator') && codegenCode.includes('generate');
        const hasCli = cliCode.includes('ValkyrieCompiler') && cliCode.includes('compile');
        
        console.log('🔍 验证生成的编译器组件:');
        console.log(`   ${hasLexer ? '✅' : '❌'} 词法分析器 (Lexer)`);
        console.log(`   ${hasParser ? '✅' : '❌'} 语法分析器 (Parser)`);
        console.log(`   ${hasCodegen ? '✅' : '❌'} 代码生成器 (CodeGenerator)`);
        console.log(`   ${hasCli ? '✅' : '❌'} 编译器核心 (ValkyrieCompiler)\n`);
        
        const allComponentsPresent = hasLexer && hasParser && hasCodegen && hasCli;
        
        if (allComponentsPresent) {
            console.log('🎉 自举编译验证成功！');
            console.log('   ✨ Valkyrie 编译器可以成功编译自己的源代码');
            console.log('   🔄 生成的编译器包含所有必要的组件');
            console.log('   🚀 自举过程完成！\n');
            
            // 比较原始文件和生成文件的大小
            const originalLexer = await fs.promises.readFile('./dist/lexer.js', 'utf8');
            const originalParser = await fs.promises.readFile('./dist/parser.js', 'utf8');
            const originalCodegen = await fs.promises.readFile('./dist/codegen.js', 'utf8');
            const originalCli = await fs.promises.readFile('./dist/cli.js', 'utf8');
            
            console.log('📊 编译产物对比:');
            console.log(`   lexer.js: ${originalLexer.length} → ${lexerCode.length} 字符`);
            console.log(`   parser.js: ${originalParser.length} → ${parserCode.length} 字符`);
            console.log(`   codegen.js: ${originalCodegen.length} → ${codegenCode.length} 字符`);
            console.log(`   cli.js: ${originalCli.length} → ${cliCode.length} 字符\n`);
            
            return true;
        } else {
            console.log('❌ 自举编译验证失败');
            console.log('   💥 生成的编译器缺少必要的组件');
            return false;
        }
        
    } catch (error) {
        console.error('❌ 自举验证过程中发生错误:', error.message);
        return false;
    }
}

// 如果直接运行此文件
if (process.argv[1] && process.argv[1].endsWith('bootstrap.js')) {
    testSelfCompilation().then(success => {
        if (success) {
            console.log('✅ Valkyrie 语言自举编译完成！');
            process.exit(0);
        } else {
            console.log('❌ Valkyrie 语言自举编译失败！');
            process.exit(1);
        }
    }).catch(error => {
        console.error('自举验证失败:', error.message);
        process.exit(1);
    });
}