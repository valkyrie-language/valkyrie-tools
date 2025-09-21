#!/usr/bin/env node

import ValkyrieGitLint from './index.js';
import path from 'path';
import fs from 'fs';

interface CLIOptions {
    config?: string;
    from?: string;
    to?: string;
    output?: string;
    help?: boolean;
}

/**
 * Valkyrie Git Lint CLI
 * 命令行接口工具
 */
class CLI {
    private gitLint: ValkyrieGitLint;

    constructor() {
        this.gitLint = new ValkyrieGitLint();
    }

    /**
     * 显示帮助信息
     */
    showHelp(): void {
        console.log(`
🚀 Valkyrie Git Lint - 可配置的 Git 提交规范工具

使用方法:
  valkyrie-git-lint <command> [选项]

命令:
  init                    初始化配置文件
  validate <message>      验证提交消息格式
  validate-emoji <msg>    验证 emoji 提交消息格式
  release <version>       生成 release 报告
  changelog              生成完整 changelog
  stats                  显示提交统计信息
  types                  显示允许的提交类型
  config                 显示当前配置

选项:
  --config <path>        指定配置文件路径
  --from <tag>           起始 tag (用于 release 和 stats)
  --to <tag>             结束 tag (默认: HEAD)
  --output <file>        输出文件名
  --help, -h             显示帮助信息

示例:
  valkyrie-git-lint init
  valkyrie-git-lint validate "feat: add new feature"
  valkyrie-git-lint validate-emoji "✨ 添加新功能"
  valkyrie-git-lint release v1.2.0 --from v1.1.0
  valkyrie-git-lint changelog --output CHANGELOG.md
  valkyrie-git-lint stats --from v1.1.0
  valkyrie-git-lint types
        `);
    }

    /**
     * 初始化配置文件
     */
    async init(): Promise<void> {
        const configPath = path.join(process.cwd(), 'git-lint.json');
        
        try {
            if (fs.existsSync(configPath)) {
                console.log('⚠️  配置文件已存在:', configPath);
                return;
            }

            this.gitLint.createDefaultConfig(configPath);
            console.log('✅ 配置文件已创建:', configPath);
        } catch (error) {
            console.error('❌ 创建配置文件失败:', error);
            process.exit(1);
        }
    }

    /**
     * 验证提交消息
     */
    async validateCommit(message: string, isEmoji: boolean = false): Promise<void> {
        try {
            const result = isEmoji 
                ? this.gitLint.validateEmojiCommitMessage(message)
                : this.gitLint.validateCommitMessage(message);

            if (result.valid) {
                console.log('✅ 提交消息格式正确');
            } else {
                console.log('❌ 提交消息格式错误:');
                for (const error of result.errors) {
                    console.log(`   - ${error}`);
                }
                process.exit(1);
            }
        } catch (error) {
            console.error('❌ 验证失败:', error);
            process.exit(1);
        }
    }

    /**
     * 生成 release 报告
     */
    async generateRelease(version: string, options: CLIOptions = {}): Promise<void> {
        try {
            const report = this.gitLint.generateReleaseReport(
                version,
                options.from || null,
                options.to || 'HEAD'
            );

            if (options.output) {
                fs.writeFileSync(options.output, report, 'utf8');
                console.log(`✅ Release 报告已保存到: ${options.output}`);
            } else {
                console.log(report);
            }
        } catch (error) {
            console.error('❌ 生成 release 报告失败:', error);
            process.exit(1);
        }
    }

    /**
     * 生成完整 changelog
     */
    async generateChangelog(options: CLIOptions = {}): Promise<void> {
        try {
            const changelog = this.gitLint.generateChangelog();
            
            if (options.output) {
                fs.writeFileSync(options.output, changelog, 'utf8');
                console.log(`✅ Changelog 已保存到: ${options.output}`);
            } else {
                console.log(changelog);
            }
        } catch (error) {
            console.error('❌ 生成 changelog 失败:', error);
            process.exit(1);
        }
    }

    /**
     * 显示提交统计信息
     */
    async showStats(options: CLIOptions = {}): Promise<void> {
        try {
            const stats = this.gitLint.getCommitStatistics(
                options.from || null,
                options.to || 'HEAD'
            );

            console.log('📊 提交统计信息:');
            console.log(`总提交数: ${stats.totalCommits}`);
            console.log(`时间范围: ${stats.dateRange.from} -> ${stats.dateRange.to}`);
            
            console.log('\\n按类型分组:');
            for (const [type, count] of Object.entries(stats.commitsByType)) {
                console.log(`  ${type}: ${count}`);
            }
            
            console.log('\\n按作者分组:');
            for (const [author, count] of Object.entries(stats.commitsByAuthor)) {
                console.log(`  ${author}: ${count}`);
            }
        } catch (error) {
            console.error('❌ 获取统计信息失败:', error);
            process.exit(1);
        }
    }

    /**
     * 显示允许的提交类型
     */
    async showTypes(): Promise<void> {
        try {
            const types = this.gitLint.getAllowedTypes();
            
            console.log('📝 允许的提交类型:');
            types.forEach(type => {
                console.log(`  - ${type}`);
            });
        } catch (error) {
            console.error('❌ 获取提交类型失败:', error);
            process.exit(1);
        }
    }

    /**
     * 显示当前配置
     */
    async showConfig(): Promise<void> {
        try {
            const config = this.gitLint.getConfig();
            console.log('⚙️  当前配置:');
            console.log(JSON.stringify(config, null, 2));
        } catch (error) {
            console.error('❌ 获取配置失败:', error);
            process.exit(1);
        }
    }

    /**
     * 解析命令行参数
     */
    parseArgs(args: string[]): { command: string; params: string[]; options: CLIOptions } {
        const options: CLIOptions = {};
        const params: string[] = [];
        let command = '';

        for (let i = 0; i < args.length; i++) {
            const arg = args[i];
            
            if (arg.startsWith('--')) {
                const key = arg.slice(2);
                const value = args[i + 1];
                
                if (key === 'help' || key === 'h') {
                    options.help = true;
                } else if (value && !value.startsWith('--')) {
                    (options as any)[key] = value;
                    i++; // 跳过下一个参数
                }
            } else if (!command) {
                command = arg;
            } else {
                params.push(arg);
            }
        }

        return { command, params, options };
    }

    /**
     * 运行 CLI
     */
    async run(): Promise<void> {
        const args = process.argv.slice(2);
        
        if (args.length === 0) {
            this.showHelp();
            return;
        }

        const { command, params, options } = this.parseArgs(args);

        if (options.help) {
            this.showHelp();
            return;
        }

        // 如果指定了配置文件，重新初始化
        if (options.config) {
            this.gitLint = new ValkyrieGitLint(options.config);
        }

        switch (command) {
            case 'init':
                await this.init();
                break;
                
            case 'validate':
                if (params.length === 0) {
                    console.error('❌ 请提供要验证的提交消息');
                    process.exit(1);
                }
                await this.validateCommit(params[0], false);
                break;
                
            case 'validate-emoji':
                if (params.length === 0) {
                    console.error('❌ 请提供要验证的 emoji 提交消息');
                    process.exit(1);
                }
                await this.validateCommit(params[0], true);
                break;
                
            case 'release':
                if (params.length === 0) {
                    console.error('❌ 请提供版本号');
                    process.exit(1);
                }
                await this.generateRelease(params[0], options);
                break;
                
            case 'changelog':
                await this.generateChangelog(options);
                break;
                
            case 'stats':
                await this.showStats(options);
                break;
                
            case 'types':
                await this.showTypes();
                break;
                
            case 'config':
                await this.showConfig();
                break;
                
            default:
                console.error(`❌ 未知命令: ${command}`);
                this.showHelp();
                process.exit(1);
        }
    }
}

// 如果直接运行此文件
if (import.meta.url === `file://${process.argv[1]}`) {
    const cli = new CLI();
    cli.run().catch(error => {
        console.error('❌ CLI 运行失败:', error);
        process.exit(1);
    });
}

export default CLI;