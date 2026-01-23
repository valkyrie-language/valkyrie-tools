import { package_compiler_generate_single_js } from "@valkyrie-language/valkyrie-bootstrap";
import fs from "fs";
import path from "path";

// 读取 builder.valkyrie 文件
const builderPath = path.join(process.cwd(), "library", "builder.valkyrie");
const builderContent = fs.readFileSync(builderPath, "utf8");

const testContent = {
    "builder.valkyrie": builderContent,
};

try {
    const result = package_compiler_generate_single_js(testContent);
    console.log("Result type:", typeof result);
    if (typeof result === "object" && result.success === false) {
        console.log("Compilation failed:", result.error);
    } else {
        console.log("Compilation successful");
        console.log("Result:", result.code || result);
    }
} catch (err) {
    console.error("Error:", err);
}
