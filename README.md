# Lisp Compiler

A next-generation Lisp-to-Rust compiler that combines Lisp's flexibility with Rust's performance and safety, designed for AI agent code manipulation and advanced metaprogramming.

## Vision

This project aims to create a powerful language that uniquely combines:

- **Lisp's Flexibility** - Full macro system and homoiconicity for dynamic code generation
- **Rust's Performance** - Zero-cost abstractions, memory safety, and native speed  
- **AI-First Design** - Code-as-data manipulation optimized for AI agents and automated refactoring
- **Modern Concurrency** - Built-in async/await, channels, and actor model support

### Key Differentiators

🔧 **Macro System** - Powerful compile-time code generation and DSL creation  
🤖 **AI Integration** - Direct AST manipulation for intelligent code analysis and transformation  
⚡ **Rust Integration** - Seamless interop with Rust's type system and concurrency primitives  
🚀 **Performance** - Compiles to efficient Rust code with minimal overhead

The result is a unique language perfect for AI agents, rapid prototyping, and systems where code-as-data manipulation provides significant advantages.

## Features

### Current Implementation
- **Complete Lisp Parser** - Handles atoms, lists, strings, numbers, booleans
- **Rust Code Generation** - Produces idiomatic Rust code
- **Built-in Operations** - Arithmetic, comparisons, conditionals
- **Variable Bindings** - `let` expressions with lexical scoping
- **Error Handling** - Comprehensive parsing and compilation error messages
- **Fast Compilation** - Direct compilation to native Rust code

### Macro System
- **Extended AST** - Full macro infrastructure with Quote, Unquote, Quasiquote, and Splice support ✅
- **Macro Definitions** - `defmacro` syntax parsing with parameter lists and `&rest` support ✅
- **Quote Family** - Complete quote/unquote/quasiquote/splice parsing (shorthand & longhand) ✅
- **Macro Expansion** - Complete macro expansion engine with parameter substitution and recursive expansion ✅
- **Hygienic Macros** - Automatic gensym-based hygiene prevents variable capture ✅
- **Pipeline Integration** - Macro expansion phase integrated into compilation pipeline ✅
- **Pattern Matching** - Advanced parameter patterns with `&rest` for variable-length arguments ✅
- **Error Handling** - Comprehensive error messages with actionable suggestions and context ✅
- **Recursion Control** - Configurable depth limits prevent infinite macro expansion loops ✅
- **Code-as-Data** - Homoiconic design for AI agent manipulation

### AST Transformation System
- **Plugin Architecture** - Extensible transform system for AI agents and code analysis tools ✅
- **Transform Pipeline** - Execute transforms between parsing and macro expansion ✅
- **Echo Transform** - Built-in debugging transform for AST visualization ✅
- **CLI Integration** - `--transforms` flag for applying transforms during compilation ✅
- **Error Handling** - Comprehensive error reporting for transform failures ✅
- **Composability** - Chain multiple transforms for complex code manipulations ✅

### JSON Intermediate Representation (IR)
- **JSON Serialization** - Convert AST to JSON format for AI agents and external tools ✅
- **Round-trip Support** - Deserialize JSON back to AST without loss of information ✅
- **CLI Flags** - `--to-ir` outputs JSON IR, `--from-ir` reads JSON IR ✅
- **AI-Friendly Format** - LLMs excel at generating valid JSON vs. Lisp syntax ✅
- **Tool Integration** - Enable external analysis, transformation, and code generation tools ✅
- **All AST Variants** - Complete support for atoms, lists, macros, and quote families ✅

### AST Validation Engine
- **Type Safety** - Detects type mismatches in operations (e.g., adding strings to numbers) ✅
- **Resource Bounds** - Catches infinite loops and unbounded recursion patterns ✅
- **FFI Restrictions** - Controls access to unsafe Rust operations and FFI calls ✅
- **Complexity Limits** - Prevents overly complex AST structures (nesting depth) ✅
- **Composable Validators** - Plugin architecture allows combining multiple validation rules ✅
- **CLI Flag** - `--validate-safety` enables pre-compilation safety checks ✅
- **Clear Error Messages** - Actionable validation errors with context ✅
- **AI Safety** - Critical for validating AI-generated code before execution ✅

> 📍 **Status**: Phase 2.1.1 (AST Validation) - Complete ✅. The compiler now includes comprehensive validation to ensure AI-generated code is safe before compilation. See [GitHub Issues](https://github.com/justin4957/rusty-lisp/issues) for implementation progress.


## Quick Start

### Prerequisites

- Rust 1.60+ installed
- Cargo package manager

### Installation

```bash
git clone <repository-url>
cd lisp-compiler
cargo build --release
```

### Basic Usage

Create a Lisp file (`example.lisp`):
```lisp
(+ 1 2 3)
(* (+ 2 3) 4)
(if (> 10 5) "greater" "less")
```

Compile to Rust:
```bash
cargo run example.lisp > output.rs
```

With AST transforms:
```bash
# Apply transforms during compilation
cargo run -- --transforms echo example.lisp > output.rs
```

Using JSON Intermediate Representation:
```bash
# Convert Lisp to JSON IR
cargo run -- --to-ir example.lisp > example.ir.json

# Compile JSON IR to Rust
cargo run -- --from-ir example.ir.json > output.rs
```

With AST validation (recommended for AI-generated code):
```bash
# Enable safety validation before compilation
cargo run -- --validate-safety example.lisp > output.rs

# Validation catches errors like type mismatches:
# Error: (+ "string" 42) -> Type mismatch in arithmetic operation
```

Compile and run the generated Rust:
```bash
rustc output.rs -o program && ./program
```

## Language Reference

### Data Types

- **Numbers**: `42`, `3.14`, `-10`
- **Strings**: `"hello world"`, `"with\nnewlines"`
- **Booleans**: `true`, `false`
- **Nil**: `nil`
- **Symbols**: `x`, `my-var`, `+`

### Built-in Operations

#### Arithmetic
```lisp
(+ 1 2 3)        ; Addition: (1 + 2 + 3)
(- 10 3)         ; Subtraction: (10 - 3)
(* 4 5 2)        ; Multiplication: (4 * 5 * 2)
(/ 20 4)         ; Division: (20 / 4)
```

#### Comparison
```lisp
(= 5 5)          ; Equality: (5 == 5)
(< 3 7)          ; Less than: (3 < 7)
(> 8 2)          ; Greater than: (8 > 2)
(<= 4 4)         ; Less or equal: (4 <= 4)
(>= 9 5)         ; Greater or equal: (9 >= 5)
```

#### Conditionals
```lisp
(if (> x 0) 
    "positive" 
    "non-positive")
```

#### Lists
```lisp
(list 1 2 3)     ; Creates: vec![1, 2, 3]
```

### Variable Binding
```lisp
(let ((x 10) (y 20)) 
     (+ x y))
```

### Macro System
```lisp
; Simple macro with parameters
(defmacro double (x)
  `(* ,x 2))

; Macro call - automatically expanded during compilation
(double 5)  ; Expands to: (* 5 2)

; Nested macro expansion
(defmacro quadruple (x)
  `(double (double ,x)))

(quadruple 3)  ; Expands to: (* (* 3 2) 2)

; Macros with &rest parameters for variable arguments
(defmacro add-all (first &rest rest)
  `(+ ,first ,@rest))

(add-all 1 2 3 4 5)  ; Expands to: (+ 1 2 3 4 5)

; Complex example: when macro with multiple body expressions
(defmacro when (condition &rest body)
  `(if ,condition (progn ,@body) nil))

(when (> x 5)
  (print "big")
  (+ x 1))  ; Expands to: (if (> x 5) (progn (print "big") (+ x 1)) nil)

; Quote family - Both shorthand and longhand forms supported
'(+ 1 2 3)                    ; Quote shorthand
(quote (+ 1 2 3))            ; Quote longhand

`(+ ,x ,(* 2 3))             ; Quasiquote with unquote shorthand
(quasiquote (+ (unquote x) (unquote (* 2 3))))  ; Longhand

`(list ,@numbers)            ; Splice shorthand
(quasiquote (list (unquote-splicing numbers)))  ; Splice longhand
```

### AST Transformation Hooks

The compiler includes a plugin system for transforming the AST before macro expansion:

```bash
# Apply echo transform for AST debugging
cargo run -- --transforms echo example.lisp

# Chain multiple transforms (future)
cargo run -- --transforms logging,optimization example.lisp
```

#### Built-in Transforms
- **echo** - Print AST structure for debugging and inspection

#### Creating Custom Transforms

```rust
use crate::ast::LispExpr;
use crate::transform::{ASTTransform, TransformError};

struct MyTransform;

impl ASTTransform for MyTransform {
    fn name(&self) -> &str {
        "my_transform"
    }

    fn transform(&self, ast: &mut LispExpr) -> Result<(), TransformError> {
        // Modify AST here
        // Example: double all numbers
        match ast {
            LispExpr::Number(n) => {
                *n *= 2.0;
                Ok(())
            }
            _ => Ok(())
        }
    }
}
```

Transforms enable:
- **AI Agent Refactoring** - Automated code restructuring
- **Code Instrumentation** - Add logging, metrics, debugging
- **Optimization** - Constant folding, dead code elimination
- **Style Enforcement** - Naming conventions, formatting
- **Security Scanning** - Pattern detection, vulnerability checking

### JSON Intermediate Representation

The compiler supports JSON serialization/deserialization of the AST, providing an AI-friendly format for code generation and manipulation.

#### Benefits for AI Agents
- **Easier Generation**: LLMs excel at producing valid JSON vs. Lisp syntax
- **Error Reduction**: Structured data reduces parsing failures
- **Tool Integration**: External analysis and transformation tools can work with JSON
- **Debugging**: Human-readable JSON format for AST inspection

#### Usage

Convert Lisp to JSON IR:
```bash
cargo run -- --to-ir example.lisp > example.ir.json
```

Compile JSON IR to Rust:
```bash
cargo run -- --from-ir example.ir.json > output.rs
```

#### JSON Format

The AST is serialized using serde's default enum representation. Each variant is represented as an object with a single key:

```json
[
  {
    "List": [
      {
        "Symbol": "+"
      },
      {
        "Number": 1.0
      },
      {
        "Number": 2.0
      }
    ]
  }
]
```

All AST variants are supported:
- **Atoms**: `{"Number": 42.0}`, `{"Symbol": "foo"}`, `{"String": "hello"}`, `{"Bool": true}`, `"Nil"`
- **Lists**: `{"List": [...]}`
- **Macros**: `{"Macro": {"name": "...", "parameters": [...], "body": {...}}}`
- **Quote Family**: `{"Quote": {...}}`, `{"Quasiquote": {...}}`, `{"Unquote": {...}}`, `{"Splice": {...}}`
- **Hygiene**: `{"Gensym": "unique_123"}`

#### Round-trip Example

```bash
# 1. Original Lisp code
echo "(+ 1 (* 2 3))" > example.lisp

# 2. Convert to JSON IR
cargo run -- --to-ir example.lisp > example.ir.json
# Output: [{"List": [{"Symbol": "+"}, {"Number": 1.0}, {"List": [{"Symbol": "*"}, {"Number": 2.0}, {"Number": 3.0}]}]}]

# 3. Compile JSON IR to Rust
cargo run -- --from-ir example.ir.json > output.rs
# Output: fn main() { println!("{:?}", (1 + (2 * 3))); }

# 4. Run the Rust code
rustc output.rs && ./output
# Output: 7
```

### AST Validation for Safe AI-Generated Code

The validation engine provides comprehensive safety checks for AST structures before compilation. This is especially critical for AI-generated code to prevent common errors and unsafe operations.

#### Validation Rules

The validator implements four categories of safety checks:

1. **Type Safety** - Catches basic type mismatches
2. **Resource Bounds** - Detects infinite loops and unbounded recursion
3. **FFI Restrictions** - Controls access to unsafe Rust operations
4. **Complexity Limits** - Prevents overly complex AST structures

#### Usage

Enable validation with the `--validate-safety` flag:

```bash
cargo run -- --validate-safety example.lisp > output.rs
```

#### Examples

**Type Safety Violation:**
```lisp
; This will fail validation
(+ "hello" 42)

; Error: Validation failed with 1 error(s):
;   - TypeSafety violation: Type mismatch: arithmetic operation '+'
;     requires numeric operands, got String
```

**Resource Bounds Violation:**
```lisp
; This will fail validation - obvious infinite recursion
(define (infinite-loop) (infinite-loop))

; Error: Validation failed with 1 error(s):
;   - ResourceBounds violation: Infinite recursion detected:
;     function 'infinite-loop' calls itself without any conditional base case
```

**FFI Restriction Violation:**
```lisp
; This will fail validation - unsafe operation not allowed
(rust-unsafe "std::ptr::null()")

; Error: Validation failed with 1 error(s):
;   - FFIRestrictions violation: FFI restriction: unsafe operation
;     'rust-unsafe' is not allowed
```

**Valid Code Passes:**
```lisp
; This passes validation
(+ 1 (* 2 3))

; Compiles successfully with validation enabled
cargo run -- --validate-safety example.lisp > output.rs
```

#### Validation in the Pipeline

Validation runs **before macro expansion** in the compilation pipeline:

```
Source → Parse → Transform → [VALIDATE] → Macro Expand → Compile → Rust
```

This ensures that:
- Invalid code is caught early, before expensive macro expansion
- Error messages reference the original source code structure
- AI-generated code is verified for safety before execution
- Multiple validation errors are reported together for efficient debugging

#### Why Validation Matters for AI

AI-generated code can contain subtle errors that are syntactically correct but semantically invalid:

- **Type errors**: LLMs may mix incompatible types in operations
- **Infinite loops**: Generated recursive functions may lack base cases
- **Unsafe operations**: AI may attempt to generate unsafe Rust code
- **Complex structures**: Generated code may exceed reasonable complexity bounds

The validation engine catches these issues **before compilation**, providing a crucial safety layer for AI-first workflows.

## Examples

### Basic Arithmetic
```lisp
; Input
(+ (* 2 3) (- 10 5))

; Generated Rust
println!("{:?}", ((2 * 3) + (10 - 5)));  // Output: 11
```

### Conditional Logic
```lisp
; Input
(if (>= 100 50) 
    (* 2 25) 
    (/ 100 4))

; Generated Rust
println!("{:?}", if (100 >= 50) { (2 * 25) } else { (100 / 4) });  // Output: 50
```

### Complex Expressions
```lisp
; Input
(let ((base 5) (height 10))
     (* 0.5 base height))

; Generated Rust
println!("{:?}", { let base = 5; let height = 10; (0.5 * base * height) });  // Output: 25
```

## Architecture

The compiler follows a traditional compilation pipeline with macro system extensions and safety validation:

1. **AST** (`src/ast.rs`) - Core `LispExpr` enum supporting both basic Lisp types and macro constructs
2. **Lexer** (`src/lexer.rs`) - Tokenizes source code
3. **Parser** (`src/parser.rs`) - Builds Abstract Syntax Tree
4. **Validator** (`src/validator.rs`) - Optional safety validation (type checking, resource bounds, FFI restrictions)
5. **Macro Expander** (`src/macro_expander.rs`) - Expands macro calls with parameter substitution
6. **Compiler** (`src/compiler.rs`) - Generates Rust code from expanded AST
7. **CLI** (`src/main.rs`) - Command-line interface

### AST Structure
The `LispExpr` enum supports:
- **Basic Types**: Numbers, Strings, Symbols, Lists, Booleans, Nil
- **Macro System**: Macro definitions, macro calls, quote families (Quote, Quasiquote, Unquote, Splice)
- **Hygiene**: Gensym for unique symbol generation

### Current Pipeline
```
Source → Lexer → Parser → [AST Transforms] → [Validator*] → Macro Expander → Compiler → Rust Code
                                              (* optional with --validate-safety)
```

The AST transform phase (Phase 1.5.2) allows plugins to modify the AST before macro expansion and validation:
- **Transform Registry**: Manage multiple transform plugins
- **Ordered Execution**: Transforms applied in registration order
- **AI Integration**: Direct AST manipulation for refactoring, optimization, and instrumentation
- **Custom Transforms**: Implement `ASTTransform` trait for custom code transformations

The macro expansion phase features:
- **Registration**: Captures macro definitions from `defmacro` forms
- **Pattern Matching**: Advanced parameter binding supporting:
  - Simple parameters: `(defmacro double (x) ...)`
  - &rest parameters: `(defmacro add-all (first &rest rest) ...)`
  - Multiple required + rest: `(defmacro foo (a b &rest others) ...)`
- **Parameter Substitution**: Replaces parameters in macro body with actual arguments
- **Recursive Expansion**: Automatically expands nested macro calls to arbitrary depth
  - Macros can call other macros (composition)
  - Supports self-recursive macros with depth limits
  - Configurable maximum expansion depth (default: 100)
  - Prevents infinite expansion loops with clear error messages
- **Hygiene**: Applies gensym-based renaming to prevent variable capture
- **Depth Limiting**: Prevents infinite recursion with configurable max depth (default: 100)
- **Error Handling**: Comprehensive, actionable error messages with:
  - Parameter count mismatches (with macro name and expected/actual counts)
  - Maximum depth exceeded (with macro name and depth limit)
  - Invalid parameter patterns (with pattern details and suggestions)
  - Malformed macro definitions (with clear explanations)
  - Context tracking for splice errors
  - Implementation of `std::error::Error` trait for proper error chaining
  - Helpful "Help:" sections with suggestions for fixing issues
  - Clone and PartialEq support for error testing

The integration ensures:
- Macro definitions are removed from the final output (return `Nil`)
- All macro calls are fully expanded before code generation
- Both basic and variadic macros work seamlessly
- Regular code passes through unchanged
- Pattern validation catches errors like `&rest` without a following parameter name

## Testing

Run the test suite:
```bash
cargo test
```

Run with verbose output:
```bash
cargo test -- --nocapture
```
