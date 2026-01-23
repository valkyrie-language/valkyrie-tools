import { package_compiler_generate_single_js } from "@valkyrie-language/valkyrie-bootstrap";

const testContent = {
    "test.valkyrie": 'micro hello() { return "Hello World"; }',
};

try {
    const result = package_compiler_generate_single_js(testContent);
    console.log("Result type:", typeof result);
    console.log("Result:", result);
} catch (err) {
    console.error("Error:", err);
}
