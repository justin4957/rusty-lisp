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

> 📍 **Status**: Phase 1.2 (Macro Expansion Engine) - Recursive expansion ✅ complete. The macro system now features robust recursive expansion with configurable depth limits, preventing infinite expansion loops while supporting complex nested macro patterns. See [GitHub Issues](https://github.com/justin4957/rusty-lisp/issues) for implementation progress.


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

The compiler follows a traditional compilation pipeline with macro system extensions:

1. **AST** (`src/ast.rs`) - Core `LispExpr` enum supporting both basic Lisp types and macro constructs
2. **Lexer** (`src/lexer.rs`) - Tokenizes source code
3. **Parser** (`src/parser.rs`) - Builds Abstract Syntax Tree
4. **Macro Expander** (`src/macro_expander.rs`) - Expands macro calls with parameter substitution
5. **Compiler** (`src/compiler.rs`) - Generates Rust code from expanded AST
6. **CLI** (`src/main.rs`) - Command-line interface

### AST Structure
The `LispExpr` enum supports:
- **Basic Types**: Numbers, Strings, Symbols, Lists, Booleans, Nil
- **Macro System**: Macro definitions, macro calls, quote families (Quote, Quasiquote, Unquote, Splice)
- **Hygiene**: Gensym for unique symbol generation

### Current Pipeline
```
Source → Lexer → Parser → [AST Transforms] → Macro Expander → Compiler → Rust Code
```

The AST transform phase (Phase 1.5.2) allows plugins to modify the AST before macro expansion:
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
