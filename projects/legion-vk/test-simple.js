import { fileURLToPath } from "url";
import { dirname, join } from "path";
import fs from "fs";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// 动态导入新版编译器
const bootstrapCompilerPath =
    "file:///E:/RustroverProjects/nyar-framework/valkyrie.valkyrie/projects/valkyrie-bootstrap/bootstrap/index.js";
const compiler = await import(bootstrapCompilerPath);

// 读取 builder.valkyrie 文件
const builderPath = join(__dirname, "library/builder.valkyrie");
const builderContent = fs.readFileSync(builderPath, "utf8");

console.log("=== 测试单个文件编译 ===");
const fileContents = {
    "builder.valkyrie": builderContent,
};

console.log("调用 package_compiler_generate_single_js...");
const result = compiler.package_compiler_generate_single_js(fileContents);

console.log("返回结果类型:", typeof result);
console.log("返回结果结构:", JSON.stringify(result, null, 2));

if (result && typeof result === "object") {
    if (result.success === true) {
        console.log("✅ 编译成功！");
        console.log("生成的代码长度:", result.code ? result.code.length : 0);
    } else if (result.success === false) {
        console.log("❌ 编译失败:", result.error);
    } else {
        console.log("🤔 未知的返回格式");
    }
} else {
    console.log("🤔 返回结果不是对象格式");
}
