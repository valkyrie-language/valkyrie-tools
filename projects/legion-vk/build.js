import fs from "fs";
import { execSync } from "child_process";
import path from "path";
import { fileURLToPath } from "url";

// 动态导入新版 bootstrap 编译器
const bootstrapCompilerPath =
    "file:///E:/RustroverProjects/nyar-framework/valkyrie.valkyrie/projects/valkyrie-bootstrap/bootstrap/index.js";
const compiler = await import(bootstrapCompilerPath);

// 获取编译器函数
const package_compiler_generate_single_js =
    compiler.package_compiler_generate_single_js;
const package_compiler_package_compiler_compile_text =
    compiler.package_compiler_package_compiler_compile_text;

/**
 * Legion Build System - Self-Bootstrap Script
 * 用于首次自举 legion 工具的构建脚本
 */

class LegionBootstrap {
    constructor(projectRoot) {
        this.projectRoot = projectRoot || process.cwd();

        // 加载工作空间配置
        this.workspaceConfig = this.loadWorkspaceConfig();

        // 加载项目配置
        this.projectConfig = this.loadProjectConfig();

        // 根据配置设置目录路径
        const buildSettings =
            this.workspaceConfig.workspace?.buildSettings || {};
        this.distDir = path.join(
            this.projectRoot,
            buildSettings.outputDir || "dist"
        );
        this.libraryDir = path.join(this.projectRoot, "library");
        this.binaryDir = path.join(this.projectRoot, "binary");

        // 移除 build 目录的使用，直接使用 dist 目录
        this.buildDir = this.distDir;

        // 确保输出目录存在
        this.ensureDirectoryExists(this.distDir);
    }

    /**
     * 确保目录存在
     */
    ensureDirectoryExists(dirPath) {
        if (!fs.existsSync(dirPath)) {
            fs.mkdirSync(dirPath, { recursive: true });
            console.log(`✓ 创建目录: ${dirPath}`);
        }
    }

    /**
     * 读取工作空间配置
     */
    loadWorkspaceConfig() {
        const configPath = path.join(this.projectRoot, "legions.json");
        if (!fs.existsSync(configPath)) {
            console.log(
                "⚠ 找不到 legions.json 工作空间配置文件，使用默认配置"
            );
            return {
                workspace: {
                    name: "legion-workspace",
                    version: "1.0.0",
                    packages: ["./"],
                    sharedDependencies: {},
                    buildSettings: {
                        target: "javascript",
                        outputDir: "build",
                        sourceDir: "src",
                    },
                },
            };
        }

        const configContent = fs.readFileSync(configPath, "utf8");
        return JSON.parse(configContent);
    }

    /**
     * 读取项目配置
     */
    loadProjectConfig() {
        const configPath = path.join(this.projectRoot, "legion.json");
        if (!fs.existsSync(configPath)) {
            throw new Error("找不到 legion.json 配置文件");
        }

        const configContent = fs.readFileSync(configPath, "utf8");
        return JSON.parse(configContent);
    }

    /**
     * 编译库文件
     */
    compileLibraryFiles() {
        if (!fs.existsSync(this.libraryDir)) {
            console.log("⚠ library 目录不存在，跳过库文件编译");
            return;
        }

        const files = fs.readdirSync(this.libraryDir);
        const valkyrieFiles = files.filter((f) => f.endsWith(".valkyrie"));

        console.log(`发现 ${valkyrieFiles.length} 个库文件`);

        // 收集所有库文件内容
        const fileContents = {};

        for (const file of valkyrieFiles) {
            const srcPath = path.join(this.libraryDir, file);
            const sourceCode = fs.readFileSync(srcPath, "utf8");
            fileContents[file] = sourceCode;
        }

        // 使用 Valkyrie 编译器生成单个 JS 文件
        const result = package_compiler_generate_single_js(fileContents);

        // 处理编译结果 - 新版编译器返回对象格式
        let compiledCode;
        if (result && typeof result === "object") {
            if (result.success === false) {
                throw new Error(`库文件编译失败: ${result.error}`);
            }
            if (result.success === true) {
                compiledCode = result.code;
            } else if (result.code) {
                compiledCode = result.code;
            } else {
                compiledCode = result;
            }
        } else if (typeof result === "string" && result.startsWith("Error:")) {
            throw new Error(`库文件编译失败: ${result}`);
        } else {
            compiledCode = result;
        }

        const outputPath = path.join(this.distDir, "index.js");
        fs.writeFileSync(outputPath, compiledCode);
        console.log(`✓ 库文件编译完成: ${outputPath}`);
    }

    /**
     * 编译二进制文件
     */
    compileBinaryFiles() {
        if (!fs.existsSync(this.binaryDir)) {
            console.log("⚠ binary 目录不存在，跳过二进制文件编译");
            return;
        }

        const binaryItems = fs.readdirSync(this.binaryDir, {
            withFileTypes: true,
        });

        for (const item of binaryItems) {
            const itemPath = path.join(this.binaryDir, item.name);

            if (item.isDirectory()) {
                // 如果是文件夹，找到 main.vk 文件编译
                const mainFile = path.join(itemPath, "main.vk");
                if (fs.existsSync(mainFile)) {
                    const outputFile = path.join(
                        this.distDir,
                        `${item.name}.js`
                    );
                    this.compileBinaryFile(mainFile, outputFile, item.name);
                }
            } else if (item.isFile() && item.name.endsWith(".vk")) {
                // 如果是单个 .vk 文件，直接编译
                const outputFile = path.join(
                    this.distDir,
                    `${item.name.replace(".vk", ".js")}`
                );
                this.compileBinaryFile(
                    itemPath,
                    outputFile,
                    item.name.replace(".vk", "")
                );
            }
        }
    }

    /**
     * 编译单个二进制文件
     */
    compileBinaryFile(srcPath, outputPath, projectName) {
        console.log(`编译二进制文件: ${srcPath} -> ${outputPath}`);

        // 读取源文件
        const sourceCode = fs.readFileSync(srcPath, "utf8");

        // 使用 Valkyrie 编译器生成单个 JS 文件
        const fileContents = {
            [path.basename(srcPath)]: sourceCode,
        };

        const result = package_compiler_generate_single_js(fileContents);

        // 处理编译结果 - 新版编译器返回对象格式
        let compiledCode;
        if (result && typeof result === "object") {
            if (result.success === false) {
                throw new Error(`二进制文件编译失败: ${result.error}`);
            }
            if (result.success === true) {
                compiledCode = result.code;
            } else if (result.code) {
                compiledCode = result.code;
            } else {
                compiledCode = result;
            }
        } else if (typeof result === "string" && result.startsWith("Error:")) {
            throw new Error(`二进制文件编译失败: ${result}`);
        } else {
            compiledCode = result;
        }

        // 添加库导入
        compiledCode =
            `// 导入库文件\nimport * as library from './index.js';\n\n` +
            compiledCode;

        // 添加主函数调用
        compiledCode +=
            '\n\n// 执行主函数\nif (typeof main === "function") {\n    main();\n}';

        // 写入编译后的文件
        fs.writeFileSync(outputPath, compiledCode);
        console.log(`✓ 二进制文件编译完成: ${outputPath}`);
    }

    /**
     * 生成 package.json
     */
    generatePackageJson() {
        const config = this.loadProjectConfig();
        const packageJson = {
            name: config.name || "legion-project",
            version: config.version || "1.0.0",
            description: "Legion Workspace Manager - Auto-generated",
            main: "index.js",
            scripts: {
                start: "node index.js",
                build: "node build.js",
                clean: "node build.js clean",
            },
            dependencies: {
                chokidar: "^3.5.3",
                commander: "^11.0.0",
            },
            engines: {
                node: ">=14.0.0",
            },
            author: "Legion Team",
            license: "MIT",
        };

        const packagePath = path.join(this.distDir, "package.json");
        fs.writeFileSync(packagePath, JSON.stringify(packageJson, null, 2));
        console.log("✓ 生成 package.json");
    }

    /**
     * 安装依赖
     */
    installDependencies() {
        console.log("安装依赖...");
        try {
            execSync("npm install", {
                cwd: this.distDir,
                stdio: "inherit",
            });
            console.log("✓ 依赖安装完成");
        } catch (error) {
            console.error("✗ 依赖安装失败:", error.message);
            throw error;
        }
    }

    /**
     * 清理构建目录
     */
    clean() {
        console.log("清理构建目录...");
        if (fs.existsSync(this.distDir)) {
            fs.rmSync(this.distDir, { recursive: true, force: true });
            console.log("✓ 构建目录已清理");
        }
        this.ensureDirectoryExists(this.distDir);
    }

    /**
     * 完整构建流程
     */
    build() {
        console.log("🚀 开始 Legion 工具自举构建...");
        console.log(`项目根目录: ${this.projectRoot}`);
        console.log(`输出目录: ${this.distDir}`);
        console.log(`库目录: ${this.libraryDir}`);
        console.log(`二进制目录: ${this.binaryDir}`);

        try {
            // 1. 清理和准备
            console.log("\n=== 步骤 1: 清理和准备 ===");
            this.clean();

            // 2. 编译库文件
            console.log("\n=== 步骤 2: 编译库文件 ===");
            this.compileLibraryFiles();

            // 3. 编译二进制文件
            console.log("\n=== 步骤 3: 编译二进制文件 ===");
            this.compileBinaryFiles();

            // 4. 生成配置文件
            console.log("\n=== 步骤 4: 生成配置文件 ===");
            this.generatePackageJson();

            // 5. 安装依赖
            console.log("\n=== 步骤 5: 安装依赖 ===");
            this.installDependencies();

            console.log("\n🎉 Legion 工具自举构建完成!");
            console.log(`构建输出目录: ${this.distDir}`);
            console.log("运行以下命令启动工具:");
            console.log(`  cd ${this.distDir} && npm start`);
        } catch (error) {
            console.error("\n✗ 构建失败:", error.message);
            console.error("错误堆栈:", error.stack);
            process.exit(1);
        }
    }
}

// 命令行接口
function main() {
    console.log("=== Legion Bootstrap 开始执行 ===");
    console.log("参数:", process.argv);

    const args = process.argv.slice(2);
    const command = args[0] || "build";
    console.log("命令:", command);

    const bootstrap = new LegionBootstrap();

    switch (command) {
        case "build":
            bootstrap.build();
            break;
        case "clean":
            bootstrap.clean();
            break;
        case "help":
            console.log(`
Legion Bootstrap 构建系统

使用方法:
  node build.js [command]

命令:
  build   执行完整构建 (默认)
  clean   清理构建目录
  help    显示此帮助信息
            `);
            break;
        default:
            console.error(`未知命令: ${command}`);
            console.log('使用 "node build.js help" 查看可用命令');
            process.exit(1);
    }
}

// 如果直接运行此脚本
if (import.meta.url === `file://${process.argv[1]}`) {
    main();
}

// 导出模块接口
export { LegionBootstrap, main };
