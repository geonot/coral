# Coral Language TODO List

This document outlines future development tasks and improvements for the Coral language project.

## Language Core & Interpreter (`src/`)

### Interpreter (`src/interpreter.rs`)
- [ ] **Implement `make_method` Execution**:
    - [ ] For `ObjectDef`: Ensure `make` methods (from `ObjectDef.methods`) are executed upon object instantiation. This requires a proper type system where `ObjectDef` is stored and used to create new instances, rather than cloning prototype `Value::Object`s.
    - [ ] For `StoreDef` & `ActorDef`: Implement initialization logic that executes their `make_method: Option<MethodDef>`. This also requires a system for defining and instantiating stores/actors.
- [ ] **Implement Iteration Operations**:
    - [ ] `Expression::AcrossOperation`: Implement evaluation logic for "across" loops.
    // - [ ] `Expression::IterateOperation`: Implement evaluation logic for "iterate" loops.
- [ ] **Collection Mutability**:
    - [ ] Implement proper in-place modification for collection methods like `push`, `add`, `pop`, `remove_at` when the collection is a variable in the environment. This involves using `environment.set()` after modifying a retrieved `Value::Array`.
- [ ] **Method Dispatch for User-Defined Objects**:
    - [ ] Extend `eval_method_call` to look up and execute methods defined in `ObjectDef` (and eventually `StoreDef`, `ActorDef`) when the object is an instance of such a type. This requires linking `Value::Object` instances back to their type definitions.
- [ ] **Review and Update Built-in Functions/Methods**:
    - [ ] Systematically review all planned or partially implemented built-in functions and methods for arrays, strings, etc.
    - [ ] Ensure consistent argument handling and return values.
    - [ ] Add missing common collection methods (e.g., `clear`, `is_empty`, `get_optional`, `remove_value`).
- [ ] **Error Handling Constructs**:
    - [ ] Implement evaluation for `Expression::ErrorHandling` (e.g., `expr err ...`).
- [ ] **State Management for Stores & Actors**:
    - [ ] Define how `Value::Store` and `Value::Actor` (or equivalent state representations) are handled, including their properties and methods.
- [ ] **Concurrency for Actors**:
    - [ ] Plan and implement the actor model's concurrency and message passing features.
- [ ] **Module System/`use` statements**:
    - [ ] Implement loading and scoping for `use` statements.

### Semantic Analyzer (`src/semantic.rs`)
- [ ] **Implement Full Semantic Analysis Pass**:
    - [ ] AST Traversal: Create the main analysis function that walks the AST.
    - [ ] Scope Management: Populate and utilize the `Scope` and `Symbol` structures during AST traversal.
    - [ ] Type System: Define and implement a type system (e.g., `Type` enum).
    - [ ] Type Checking: Perform type checking for expressions, assignments, function calls, method calls, returns, etc.
    - [ ] Type Inference: Implement basic type inference where possible.
    - [ ] Variable/Function Resolution: Ensure all identifiers are defined before use.
    - [ ] Method Resolution: For method calls, verify the method exists on the object's type and signatures match.
    - [ ] `make_method` Validation: Check that `make` methods (if special rules apply, e.g. no parameters, no return value) conform to these.
    - [ ] `AcrossOperation` & `IterateOperation` Validation: Check types of collections, functions/operations used.
    - [ ] Built-in Function/Method Signature Checks.
    - [ ] Error Reporting: Generate meaningful semantic error messages.

### Lexer & Parser (`src/lexer.rs`, `src/parser.rs`, `src/ast.rs`)
- [ ] **Review AST**: Ensure all AST nodes are optimally structured and used.
- [ ] **Parser Error Recovery**: Enhance parser error recovery mechanisms for more graceful handling of syntax errors.
- [ ] **String Interpolation**: Fully implement parsing and ensure AST represents interpolated segments. Interpreter needs to handle evaluation.

## Tooling

### Language Server Protocol (LSP - `src/lsp.rs`, `src/lsp_main.rs`)
- [ ] **Resolve LSP Server Compilation Issues**:
    - [ ] Address the Rust toolchain/MSRV conflicts to allow `coral-lsp` to build. This likely involves setting up a CI environment with a fixed, compatible Rust version or carefully managing dependencies.
- [ ] **Implement Advanced LSP Features** (once LSP build is stable and semantic analysis is available):
    - [ ] Go-to-Definition: Use semantic analysis info.
    - [ ] Hover Information: Provide type information and documentation from semantic analysis.
    - [ ] Completions: Offer context-aware completions based on scope and type information.
    - [ ] Diagnostics: Integrate semantic errors from the analyzer.
    - [ ] Find References.
    - [ ] Rename Refactoring.
- [ ] **Tree-sitter Integration in LSP**: Consider using the Tree-sitter parser within the LSP for faster and more fault-tolerant parsing, potentially for syntax highlighting scopes or document structure analysis, even if the main compilation still uses the hand-rolled parser.

### Tree-sitter Grammar (`tree-sitter-coral/`)
- [ ] **Implement External Scanner**: Create `external_scanner.c` to robustly handle newline, indentation, and dedentation for Python-style blocks. This is critical for the grammar to parse real-world code accurately.
- [ ] **Refine `grammar.js`**:
    - [ ] Complete all remaining EBNF constructs (e.g., postfix `unless`, full error handling syntax, remaining built-ins).
    - [ ] Test thoroughly and resolve any remaining conflicts or ambiguities reported by `tree-sitter generate`.
    - [ ] Ensure string interpolation is fully parsed.
- [ ] **Tree-sitter Queries**: Develop queries for syntax highlighting, code folding, and other structural features if Tree-sitter is used directly by editors.

### VS Code Extension (`vscode-coral/`)
- [ ] **Syntax Highlighting**:
    - [ ] Fully synchronize `syntaxes/coral.tmLanguage.json` with the Tree-sitter grammar.
    - [ ] Alternatively, investigate using Tree-sitter directly for syntax highlighting in VS Code via the LSP or a dedicated Tree-sitter mechanism, which would provide more accurate results than the TextMate grammar.
- [ ] **LSP Client Integration**: Ensure robust starting, stopping, and restarting of the `coral-lsp` client once it's buildable.
- [ ] **Snippets**: Add more useful code snippets.
- [ ] **Semantic Indentation**: If possible, use LSP feedback or Tree-sitter to provide more accurate auto-indentation than regex-based rules in `language-configuration.json`.

## Documentation (`docs/`)
- [ ] **Language Guide & Reference**:
    - [ ] Write a comprehensive guide covering all language features, syntax, semantics, and standard library (if any).
    - [ ] Create a detailed reference manual.
- [ ] **Tooling Guides**: Document how to use the LSP, VS Code extension, and any other tools.
- [ ] **Update `README.md`**: Ensure the main README is up-to-date with project status and goals.
- [ ] **Contribution Guide**: If applicable.

## Examples (`examples/`)
- [ ] **Create More Examples**:
    - [ ] Develop a diverse set of examples showcasing all language features, from basic syntax to complex applications (e.g., actors, stores).
    - [ ] Include examples for error handling, data structures, control flow, etc.
- [ ] **Update Existing Examples**:
    - [ ] Review and update any current examples to reflect language changes from the EBNF/parser updates.
    - [ ] Ensure examples follow current best practices for the language.

## Testing
- [ ] **Language Core Tests**:
    - [ ] Expand test coverage in `src/main.rs` (if used for testing) or a dedicated test suite for parsing all valid syntax constructs and rejecting invalid ones.
    - [ ] Include edge cases for lexer, parser.
- [ ] **Interpreter Tests** (once interpreter features are more complete):
    - [ ] Unit tests for individual expression evaluations.
    - [ ] Tests for statement execution, scope handling.
    - [ ] Tests for `make_method` execution, collection operations, control flow.
- [ ] **Semantic Analyzer Tests** (once implemented):
    - [ ] Tests for type checking, error reporting, scope resolution.
- [ ] **LSP Server Tests**:
    - [ ] Unit and integration tests for LSP features (diagnostics, completion, hover, etc.).
- [ ] **VS Code Extension Tests**:
    - [ ] Basic activation tests.
    - [ ] Syntax highlighting correctness (visual or snapshot).
- [ ] **Tree-sitter Grammar Tests**:
    - [ ] Use `tree-sitter test` with corpus examples to validate the grammar.

## Build & CI/CD
- [ ] **Setup CI**: Implement Continuous Integration (e.g., GitHub Actions) to:
    - [ ] Run Rust tests (`cargo test`).
    - [ ] Build the compiler/interpreter, LSP.
    - [ ] Generate Tree-sitter parser and run its tests.
    - [ ] Build the VS Code extension.
- [ ] **Consistent Toolchain**: Use `rust-toolchain.toml` to pin down the Rust version for the project to ensure all developers and CI use the same version, avoiding MSRV issues during development.```markdown
# Coral Language TODO List

This document outlines future development tasks and improvements for the Coral language project.

## Language Core & Interpreter (`src/`)

### Interpreter (`src/interpreter.rs`)
- [ ] **Implement `make_method` Execution**:
    - [ ] For `ObjectDef`: Ensure `make` methods (from `ObjectDef.methods`) are executed upon object instantiation. This requires a proper type system where `ObjectDef` is stored and used to create new instances, rather than cloning prototype `Value::Object`s.
    - [ ] For `StoreDef` & `ActorDef`: Implement initialization logic that executes their `make_method: Option<MethodDef>`. This also requires a system for defining and instantiating stores/actors.
- [ ] **Implement Iteration Operations**:
    - [ ] `Expression::AcrossOperation`: Implement evaluation logic for "across" loops.
    - [ ] `Expression::IterateOperation`: Implement evaluation logic for "iterate" loops.
- [ ] **Collection Mutability**:
    - [ ] Implement proper in-place modification for collection methods like `push`, `add`, `pop`, `remove_at` when the collection is a variable in the environment. This involves using `environment.set()` after modifying a retrieved `Value::Array`.
- [ ] **Method Dispatch for User-Defined Objects**:
    - [ ] Extend `eval_method_call` to look up and execute methods defined in `ObjectDef` (and eventually `StoreDef`, `ActorDef`) when the object is an instance of such a type. This requires linking `Value::Object` instances back to their type definitions.
- [ ] **Review and Update Built-in Functions/Methods**:
    - [ ] Systematically review all planned or partially implemented built-in functions and methods for arrays, strings, etc.
    - [ ] Ensure consistent argument handling and return values.
    - [ ] Add missing common collection methods (e.g., `clear`, `is_empty`, `get_optional`, `remove_value`).
- [ ] **Error Handling Constructs**:
    - [ ] Implement evaluation for `Expression::ErrorHandling` (e.g., `expr err ...`).
- [ ] **State Management for Stores & Actors**:
    - [ ] Define how `Value::Store` and `Value::Actor` (or equivalent state representations) are handled, including their properties and methods.
- [ ] **Concurrency for Actors**:
    - [ ] Plan and implement the actor model's concurrency and message passing features.
- [ ] **Module System/`use` statements**:
    - [ ] Implement loading and scoping for `use` statements.

### Semantic Analyzer (`src/semantic.rs`)
- [ ] **Implement Full Semantic Analysis Pass**:
    - [ ] AST Traversal: Create the main analysis function that walks the AST.
    - [ ] Scope Management: Populate and utilize the `Scope` and `Symbol` structures during AST traversal.
    - [ ] Type System: Define and implement a type system (e.g., `Type` enum).
    - [ ] Type Checking: Perform type checking for expressions, assignments, function calls, method calls, returns, etc.
    - [ ] Type Inference: Implement basic type inference where possible.
    - [ ] Variable/Function Resolution: Ensure all identifiers are defined before use.
    - [ ] Method Resolution: For method calls, verify the method exists on the object's type and signatures match.
    - [ ] `make_method` Validation: Check that `make` methods (if special rules apply, e.g. no parameters, no return value) conform to these.
    - [ ] `AcrossOperation` & `IterateOperation` Validation: Check types of collections, functions/operations used.
    - [ ] Built-in Function/Method Signature Checks.
    - [ ] Error Reporting: Generate meaningful semantic error messages.

### Lexer & Parser (`src/lexer.rs`, `src/parser.rs`, `src/ast.rs`)
- [ ] **Review AST**: Ensure all AST nodes are optimally structured and used.
- [ ] **Parser Error Recovery**: Enhance parser error recovery mechanisms for more graceful handling of syntax errors.
- [ ] **String Interpolation**: Fully implement parsing and ensure AST represents interpolated segments. Interpreter needs to handle evaluation.

## Tooling

### Language Server Protocol (LSP - `src/lsp.rs`, `src/lsp_main.rs`)
- [ ] **Resolve LSP Server Compilation Issues**:
    - [ ] Address the Rust toolchain/MSRV conflicts to allow `coral-lsp` to build. This likely involves setting up a CI environment with a fixed, compatible Rust version or carefully managing dependencies (e.g. via `rust-toolchain.toml` and potentially a blessed `Cargo.lock`).
- [ ] **Implement Advanced LSP Features** (once LSP build is stable and semantic analysis is available):
    - [ ] Go-to-Definition: Use semantic analysis info.
    - [ ] Hover Information: Provide type information and documentation from semantic analysis.
    - [ ] Completions: Offer context-aware completions based on scope and type information.
    - [ ] Diagnostics: Integrate semantic errors from the analyzer.
    - [ ] Find References.
    - [ ] Rename Refactoring.
- [ ] **Tree-sitter Integration in LSP**: Consider using the Tree-sitter parser within the LSP for faster and more fault-tolerant parsing, potentially for syntax highlighting scopes or document structure analysis, even if the main compilation still uses the hand-rolled parser.

### Tree-sitter Grammar (`tree-sitter-coral/`)
- [ ] **Implement External Scanner**: Create `external_scanner.c` to robustly handle newline, indentation, and dedentation for Python-style blocks. This is critical for the grammar to parse real-world code accurately.
- [ ] **Refine `grammar.js`**:
    - [ ] Complete all remaining EBNF constructs (e.g., postfix `unless`, full error handling syntax, remaining built-ins).
    - [ ] Test thoroughly and resolve any remaining conflicts or ambiguities reported by `tree-sitter generate`.
    - [ ] Ensure string interpolation is fully parsed.
- [ ] **Tree-sitter Queries**: Develop queries for syntax highlighting, code folding, and other structural features if Tree-sitter is used directly by editors.

### VS Code Extension (`vscode-coral/`)
- [ ] **Syntax Highlighting**:
    - [ ] Fully synchronize `syntaxes/coral.tmLanguage.json` with the Tree-sitter grammar.
    - [ ] Alternatively, investigate using Tree-sitter directly for syntax highlighting in VS Code via the LSP or a dedicated Tree-sitter mechanism, which would provide more accurate results than the TextMate grammar.
- [ ] **LSP Client Integration**: Ensure robust starting, stopping, and restarting of the `coral-lsp` client once it's buildable.
- [ ] **Snippets**: Add more useful code snippets.
- [ ] **Semantic Indentation**: If possible, use LSP feedback or Tree-sitter to provide more accurate auto-indentation than regex-based rules in `language-configuration.json`.

## Documentation (`docs/`)
- [ ] **Language Guide & Reference**:
    - [ ] Write a comprehensive guide covering all language features, syntax, semantics, and standard library (if any).
    - [ ] Create a detailed reference manual.
- [ ] **Tooling Guides**: Document how to use the LSP, VS Code extension, and any other tools.
- [ ] **Update `README.md`**: Ensure the main README is up-to-date with project status and goals.
- [ ] **Contribution Guide**: If applicable.

## Examples (`examples/`)
- [ ] **Create More Examples**:
    - [ ] Develop a diverse set of examples showcasing all language features, from basic syntax to complex applications (e.g., actors, stores).
    - [ ] Include examples for error handling, data structures, control flow, etc.
- [ ] **Update Existing Examples**:
    - [ ] Review and update any current examples to reflect language changes from the EBNF/parser updates.
    - [ ] Ensure examples follow current best practices for the language.

## Testing
- [ ] **Language Core Tests**:
    - [ ] Expand test coverage in `src/main.rs` (if used for testing) or a dedicated test suite for parsing all valid syntax constructs and rejecting invalid ones.
    - [ ] Include edge cases for lexer, parser.
- [ ] **Interpreter Tests** (once interpreter features are more complete):
    - [ ] Unit tests for individual expression evaluations.
    - [ ] Tests for statement execution, scope handling.
    - [ ] Tests for `make_method` execution, collection operations, control flow.
- [ ] **Semantic Analyzer Tests** (once implemented):
    - [ ] Tests for type checking, error reporting, scope resolution.
- [ ] **LSP Server Tests**:
    - [ ] Unit and integration tests for LSP features (diagnostics, completion, hover, etc.).
- [ ] **VS Code Extension Tests**:
    - [ ] Basic activation tests.
    - [ ] Syntax highlighting correctness (visual or snapshot).
- [ ] **Tree-sitter Grammar Tests**:
    - [ ] Use `tree-sitter test` with corpus examples to validate the grammar.

## Build & CI/CD
- [ ] **Setup CI**: Implement Continuous Integration (e.g., GitHub Actions) to:
    - [ ] Run Rust tests (`cargo test`).
    - [ ] Build the compiler/interpreter, LSP.
    - [ ] Generate Tree-sitter parser and run its tests.
    - [ ] Build the VS Code extension.
- [ ] **Consistent Toolchain**: Use `rust-toolchain.toml` to pin down the Rust version for the project to ensure all developers and CI use the same version, avoiding MSRV issues during development.
```
