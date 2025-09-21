#!/usr/bin/env node

import path from 'path';
import fs from 'fs';

// 导入模块
import ValkyrieGitLint from '../src/index.js';
import { CommitLint, ReleaseReportGenerator, ConfigLoader } from '../src/index.js';

// 测试配置文件路径
const testConfigPath = path.join(__dirname, '..', 'git-lint.json');

// 测试结果统计
let totalTests = 0;
let passedTests = 0;
let failedTests = 0;

function test(name: string, fn: () => void): void {
    totalTests++;
    try {
        console.log(`\n🧪 测试: ${name}`);
        fn();
        console.log(`✅ 通过: ${name}`);
        passedTests++;
    } catch (error) {
        console.log(`❌ 失败: ${name}`);
        console.log(`   错误: ${(error as Error).message}`);
        failedTests++;
    }
}

function assert(condition: boolean, message?: string): void {
    if (!condition) {
        throw new Error(message || '断言失败');
    }
}

function assertEqual(actual: any, expected: any, message?: string): void {
    if (actual !== expected) {
        throw new Error(message || `期望 ${expected}, 实际 ${actual}`);
    }
}

function assertNotNull(value: any, message?: string): void {
    if (value === null || value === undefined) {
        throw new Error(message || '值不应为 null 或 undefined');
    }
}

console.log('🚀 开始运行 Valkyrie Git Lint 测试套件\n');

// ConfigLoader 测试
test('ConfigLoader - 加载配置文件', () => {
    const loader = new ConfigLoader();
    const config = loader.loadConfig();
    assertNotNull(config, '配置不应为空');
    assert(typeof config === 'object', '配置应为对象');
});

test('ConfigLoader - 验证配置格式', () => {
    const loader = new ConfigLoader();
    const config = loader.loadConfig();
    loader.validateConfig(config);
    assert(true, '配置验证通过');
});

// CommitLint 测试
test('CommitLint - 验证传统提交格式', () => {
    const commitLint = new CommitLint();
    const result = commitLint.validateCommitMessage('feat: add new feature');
    assert(result.valid, '传统提交格式应该有效');
    assertEqual(result.type, 'feat', '类型应为 feat');
    assertEqual(result.description, 'add new feature', '描述应匹配');
});

test('CommitLint - 验证 emoji 提交格式', () => {
    const commitLint = new CommitLint();
    const result = commitLint.validateEmojiCommitMessage('✨ feat: add new feature');
    assert(result.valid, 'Emoji 提交格式应该有效');
    assertEqual(result.type, 'feat', '类型应为 feat');
    assertEqual(result.description, 'add new feature', '描述应匹配');
});

test('CommitLint - 获取类型选择', () => {
    const commitLint = new CommitLint();
    const choices = commitLint.getTypeChoices();
    assert(Array.isArray(choices), '类型选择应为数组');
    assert(choices.length > 0, '应有可用的类型选择');
});

test('CommitLint - 生成 commitlint 配置', () => {
    const commitLint = new CommitLint();
    const config = commitLint.generateCommitlintConfig();
    assertNotNull(config, 'commitlint 配置不应为空');
    assert(typeof config === 'object', 'commitlint 配置应为对象');
});

// ReleaseReportGenerator 测试
test('ReleaseReportGenerator - 解析提交', () => {
    const generator = new ReleaseReportGenerator();
    const commit = generator.parseCommit('feat: add new feature');
    assertNotNull(commit, '解析的提交不应为空');
    assertEqual(commit.type, 'feat', '类型应为 feat');
    assertEqual(commit.description, 'add new feature', '描述应匹配');
});

test('ReleaseReportGenerator - 分组提交', () => {
    const generator = new ReleaseReportGenerator();
    const commits = [
        { type: 'feat', description: 'feature 1', hash: 'abc123', author: 'test', date: '2023-01-01' },
        { type: 'fix', description: 'fix 1', hash: 'def456', author: 'test', date: '2023-01-02' },
        { type: 'feat', description: 'feature 2', hash: 'ghi789', author: 'test', date: '2023-01-03' }
    ];
    const grouped = generator.groupCommitsByType(commits);
    assertNotNull(grouped, '分组结果不应为空');
    assert(grouped.feat && grouped.feat.length === 2, 'feat 类型应有 2 个提交');
    assert(grouped.fix && grouped.fix.length === 1, 'fix 类型应有 1 个提交');
});

test('ReleaseReportGenerator - 生成 Markdown', () => {
    const generator = new ReleaseReportGenerator();
    const commits = [
        { type: 'feat', description: 'feature 1', hash: 'abc123', author: 'test', date: '2023-01-01' },
        { type: 'fix', description: 'fix 1', hash: 'def456', author: 'test', date: '2023-01-02' }
    ];
    const markdown = generator.generateMarkdown(commits, '1.0.0', '0.9.0');
    assertNotNull(markdown, 'Markdown 不应为空');
    assert(typeof markdown === 'string', 'Markdown 应为字符串');
    assert(markdown.includes('## 1.0.0'), '应包含版本标题');
});

// ValkyrieGitLint 主类测试
test('ValkyrieGitLint - 主类初始化', () => {
    const gitLint = new ValkyrieGitLint();
    assertNotNull(gitLint, '主类实例不应为空');
});

test('ValkyrieGitLint - 验证提交消息', () => {
    const gitLint = new ValkyrieGitLint();
    const result = gitLint.validateCommitMessage('feat: add new feature');
    assertNotNull(result, '验证结果不应为空');
    assert(typeof result === 'object', '验证结果应为对象');
});

test('ValkyrieGitLint - 获取允许的类型', () => {
    const gitLint = new ValkyrieGitLint();
    const types = gitLint.getAllowedTypes();
    assert(Array.isArray(types), '允许的类型应为数组');
    assert(types.length > 0, '应有可用的类型');
});

test('ValkyrieGitLint - 获取配置', () => {
    const gitLint = new ValkyrieGitLint();
    const config = gitLint.getConfig();
    assertNotNull(config, '配置不应为空');
    assert(typeof config === 'object', '配置应为对象');
});

// 输出测试结果
console.log('\n📊 测试结果统计:');
console.log(`总测试数: ${totalTests}`);
console.log(`通过: ${passedTests} ✅`);
console.log(`失败: ${failedTests} ❌`);

if (failedTests === 0) {
    console.log('\n🎉 所有测试通过！');
    process.exit(0);
} else {
    console.log('\n💥 有测试失败，请检查代码');
    process.exit(1);
}