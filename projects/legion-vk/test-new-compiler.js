// 测试新版 bootstrap 编译器
import { readFileSync } from "fs";
import { join } from "path";

// 尝试从 valkyrie-bootstrap 项目导入新版编译器
const bootstrapCompilerPath =
    "file:///E:/RustroverProjects/nyar-framework/valkyrie.valkyrie/projects/valkyrie-bootstrap/bootstrap/index.js";

try {
    console.log("正在加载新版 bootstrap 编译器...");
    const compiler = await import(bootstrapCompilerPath);

    // 读取 builder.valkyrie 文件
    const builderPath =
        "E:\\RustroverProjects\\nyar-framework\\valkyrie.valkyrie\\projects\\legion\\library\\builder.valkyrie";
    const builderContent = readFileSync(builderPath, "utf8");

    console.log("正在编译 builder.valkyrie 文件...");

    // 使用新版的 package_compiler_compile_text 函数
    if (
        typeof compiler.package_compiler_package_compiler_compile_text ===
        "function"
    ) {
        const result =
            compiler.package_compiler_package_compiler_compile_text(
                builderContent
            );

        console.log("编译结果类型:", typeof result);
        console.log("编译结果:", result);

        if (result && typeof result === "object" && result.success === false) {
            console.error("编译错误:", result.error);
        } else if (typeof result === "string" && result.startsWith("Error:")) {
            console.error("编译错误:", result);
        } else {
            console.log("编译成功!");
        }
    } else {
        console.error(
            "找不到 package_compiler_package_compiler_compile_text 函数"
        );
        console.log("可用的函数:", Object.keys(compiler));
    }
} catch (error) {
    console.error("加载或编译失败:", error.message);
    console.error(error.stack);
}
