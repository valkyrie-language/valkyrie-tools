import fs from "fs";
import path from "path";
import { execSync } from "child_process";

console.log("Debug script starting...");

try {
    console.log("Testing basic fs functionality...");
    const testPath = path.join(process.cwd(), "test");
    console.log("Test path:", testPath);

    console.log("Testing valkyrie-bootstrap import...");
    import { package_compiler_generateSingleJSFromAnalysis } from "@valkyrie-language/valkyrie-bootstrap";
    console.log("valkyrie-bootstrap imported successfully");

    console.log("Debug script completed successfully!");
} catch (error) {
    console.error("Error in debug script:", error);
    console.error("Stack trace:", error.stack);
}
