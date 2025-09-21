# Changelog

本文档记录了项目的所有重要变更。

## [v0.1.0] - 2020/2/29

### 🐛 Bug Fixes

- :bug: **Remove bad decimal** (d9aaca5) by @Aster
- :bug: **Derive debug for literal objects** (3904f9a) by @Aster

### 🔧 Build System

- :wrench: **`{ }` now can call only once** (d024e78) by @Aster
- :wrench: **Fix dot call** (d8341da) by @Aster
- :wrench: **Fix slice and infix call** (7b1ec71) by @Aster
- :wrench: **Fix the problem that some digital analysis is lost** (933b63c) by @Aster
- :wrench: **Fix slice and infix call** (20a666d) by @Aster
- :wrench: **Fix symbol parsing** (1e31dc8) by @Aster
- :wrench: **Check the order of evaluation for unary operators** (8e31766) by @Aster
- :wrench: **Debug for unary call** (ad091cc) by @Aster
- :wrench: **Debug for if statement** (bfc0f73) by @Aster
- :wrench: **Debug for null, ture, false** (923d314) by @Aster
- :wrench: **Debug for numbers' parsing** (c66937b) by @Aster
- :wrench: **Fix parse boolean** (164a98e) by @Aster
- :wrench: **Fix error kinds** (76636fe) by @Aster
- :wrench: **Cancel the trinocular operators** (94f27fb) by @Aster
- :wrench: **Cancel the strongly coupled interfaces** (10c53da) by @Aster
- :wrench: **Add more test cases** (2a6b762) by @Aster
- :wrench: **Fixed AST parameter passing** (f2eafa2) by @Aster
- :wrench: **Release AST and valkyrie parser** (808e129) by @Aster
- :wrench: **Adjust the parsing order of some operations** (0e612d2) by @Aster
- :wrench: **Config projects' settings** (854b9aa) by @Aster
- :wrench: **Add nyar vampire** (e8c239a) by @Aster
- :wrench: **Add nyar vanilla** (06e5294) by @Aster
- :wrench: **Add nyar ast** (f3efe5a) by @Aster
- :wrench: **Add test files** (0d69836) by @Aster
- :wrench: **Add build.rs for pre-build** (149b621) by @Aster

### 🚀 Chores

- :rocket: **Release version 0.1** (68f3014) by @Aster

### other

-  **Distinguish list, index and slice** (0d8759d) by @Aster
-  **Access control character** (3ecbe58) by @Aster
-  **Introduce Zero-Sized Types** (092d895) by @Aster
-  **Support primitive numbers** (3012aa7) by @Aster
-  **Allow generic calls** (3dc1dde) by @Aster
-  **Design OOP system** (28dcc4a) by @Aster
-  **Pretty print for call infix** (38a9abc) by @Aster
-  **Simplify the display of numbers** (64e6e8d) by @Aster
-  **Update valkyrie ast** (66feb90) by @Aster
-  **Update valkyrie parser** (c341153) by @Aster
-  **Update pest lexer** (304963a) by @Aster
-  **Add mir nodes** (6bb6367) by @Aster
-  **Reorganize control flow module** (80b3d4b) by @Aster
-  **Reorganize call statements** (fb9271e) by @Aster
-  **Reorganize parser module** (cb92dd6) by @Aster
-  **Reorganize ast module** (7150462) by @Aster
-  **Support type boolean collector** (abf0608) by @Aster
-  **Support type refine** (5c1b4fc) by @Aster
-  **Add type display** (9b8cfc4) by @Aster
-  **Remove some parser** (6ebbd27) by @Aster
-  **Add options to check extra and override arguments** (0c23685) by @Aster
-  **Add lazy static function prototype** (d1cf57c) by @Aster
-  **Add new interpreter** (5148a4c) by @Aster
-  **Refactoring Valkyrie parser** (a77eb2a) by @Aster
-  **Support class & trait extension** (faeccc8) by @Aster
-  **Support define parametric type functions** (c291a59) by @Aster
-  **Adjust statement parse** (8013135) by @Aster
-  **Adjust function apply expression syntax** (d2f767c) by @Aster
-  **Valkyrie Language Specification** (ff33684) by @Aster
-  **Support list expression** (09fe058) by @Aster
-  **Support string refine** (dded1bc) by @Aster
-  **Support global settings** (89bdf70) by @Aster
-  **Fix number parse edge cases** (b917d62) by @Aster
-  **Support number parse** (cff6b9f) by @Aster
-  **Use conditional pairs for if statements** (3934cbb) by @Aster
-  **Support nullable modifier** (7c5498e) by @Aster
-  **Support symbol parse** (82a4944) by @Aster
-  **Expression terms resolve** (0e4a9ed) by @Aster
-  **Support bracket call and trinocular** (fab7635) by @Aster
-  **AST save and load as json** (40640c3) by @Aster
-  **Solve the problem of parsing operation priority** (d7d14b0) by @Aster
-  **Support expression terms and nodes** (11fec40) by @Aster
-  **Add lazy static PREC_CLIMBER** (2cb1d53) by @Aster
-  **Remove data type in AST root** (3a1f289) by @Aster
-  **Expression terms parse** (14e2c91) by @Aster
-  **Support complex number parse** (1406ef6) by @Aster
-  **Support integer and decimal parse** (4f2013e) by @Aster
-  **Support byte number parse** (86e49c3) by @Aster
-  **Support AST transform** (2ee99e9) by @Aster
-  **Update string parser** (00a7747) by @Aster
-  **Support block or expression statement** (ecf755d) by @Aster
-  **Import statement struct** (ee070af) by @Aster
-  **Parse using alias** (b3070a6) by @Aster
-  **Support relative path reference** (12d3ff9) by @Aster
-  **Better import statement resolve** (6b270f0) by @Aster
-  **Support annotation** (3da7819) by @Aster
-  **Support set assign statements** (f7cb2a2) by @Aster
-  **Support let assign statements** (bcb628e) by @Aster
-  **Parse bool** (21b05a6) by @Aster
-  **Support AST serde** (bf266d4) by @Aster
-  **Add string parser** (d14b07f) by @Aster
-  **Add assign statements** (fd58009) by @Aster
-  **Add trait statements** (f36eefb) by @Aster
-  **Better string type** (31a50e0) by @Aster
-  **Automatic AST derive** (5cea170) by @Aster
-  **Refine use statements** (3c32b48) by @Aster
-  **Use use keyword** (d3475e9) by @Aster
-  **Remove except keyword** (cb8a328) by @Aster
-  **Support import statement** (46c184c) by @Aster
-  **Refine if statement** (0a3ccaf) by @Aster
-  **Support if statement** (670463c) by @Aster
-  **Remove end pattern of block** (ac8d67d) by @Aster
-  **Remove traditional for statement** (333c03c) by @Aster
-  **Support for statement** (4c4b000) by @Aster
-  **Support type expressions** (c8c8ebe) by @Aster
-  **Use pre build code gen** (983f214) by @Aster
-  **Support expressions** (f74736c) by @Aster
-  **Re-format the files** (cf93921) by @Aster
-  **Support data type** (a1904d6) by @Aster
-  **Support boolean type** (457816f) by @Aster
-  **Define symbol syntax** (9c807ed) by @Aster
-  **Remove exponent number** (119e9c5) by @Aster
-  **Support number literal** (363ec1a) by @Aster
-  **Support string literal** (031f404) by @Aster
-  **Add comment syntax** (89ac728) by @Aster
-  **Adjust the directory structure** (56c2c3d) by @Aster
-  **Add empty statements** (a160cdb) by @Aster
-  **Add basic operators** (20ddbb5) by @Aster
-  **`new` methods for ValueData** (26e0897) by @Aster
-  **Type wraps for rust types** (31ce35a) by @Aster
-  **GC and display form for types** (9f14e45) by @Aster
-  **Define primitive types** (4b66012) by @Aster
-  **Basic numberic types** (49a2272) by @Aster
-  **Project initialized!** (57549d4) by @Aster

