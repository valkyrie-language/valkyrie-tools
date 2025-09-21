module.exports = {
    extends: ['@commitlint/config-conventional'],
    rules: {
        'type-enum': [2, 'always', [
            'feat', 'fix', 'docs', 'style', 'refactor', 'test', 'config', 'perf',
            'release', 'tag', 'ci', 'build', 'revert', 'idea', 'delete', 'complete', 'branch', 'experiment'
        ]],
        'type-empty': [2, 'never'],
        'subject-empty': [2, 'never'],
        'header-max-length': [2, 'always', 72]
    },
    prompt: {
        questions: {
            type: {
                description: '选择你要提交的更改类型:',
                enum: {
                    '✨ feat': {
                        description: '新功能',
                        title: 'Features',
                        emoji: '✨'
                    },
                    '🔧 fix': {
                        description: '修复bug',
                        title: 'Bug Fixes',
                        emoji: '🔧'
                    },
                    '📝 docs': {
                        description: '文档更新',
                        title: 'Documentation',
                        emoji: '📝'
                    },
                    '🎨 style': {
                        description: '代码格式调整',
                        title: 'Styles',
                        emoji: '🎨'
                    },
                    '☢️ refactor': {
                        description: '重构代码',
                        title: 'Code Refactoring',
                        emoji: '☢️'
                    },
                    '🧪 test': {
                        description: '测试相关',
                        title: 'Tests',
                        emoji: '🧪'
                    },
                    '🔨 config': {
                        description: '配置文件修改',
                        title: 'Configuration',
                        emoji: '🔨'
                    },
                    '⚡️ perf': {
                        description: '性能优化',
                        title: 'Performance Improvements',
                        emoji: '⚡️'
                    },
                    '🚀 release': {
                        description: '发布版本',
                        title: 'Release',
                        emoji: '🚀'
                    },
                    '🔖 tag': {
                        description: '标签相关',
                        title: 'Tag',
                        emoji: '🔖'
                    },
                    '🚦 ci': {
                        description: 'CI/CD相关',
                        title: 'CI/CD',
                        emoji: '🚦'
                    },
                    '📦 build': {
                        description: '构建相关',
                        title: 'Build',
                        emoji: '📦'
                    },
                    '⏪ revert': {
                        description: '回滚操作',
                        title: 'Revert',
                        emoji: '⏪'
                    },
                    '💡 idea': {
                        description: '新想法',
                        title: 'Idea',
                        emoji: '💡'
                    },
                    '🧨 delete': {
                        description: '删除文件',
                        title: 'Delete',
                        emoji: '🧨'
                    },
                    '✅ complete': {
                        description: '完成任务',
                        title: 'Complete',
                        emoji: '✅'
                    },
                    '🔀 branch': {
                        description: '分支操作',
                        title: 'Branch',
                        emoji: '🔀'
                    },
                    '🔮 experiment': {
                        description: '实验性功能',
                        title: 'Experiment',
                        emoji: '🔮'
                    }
                }
            }
        }
    }
};