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

- **Complete Lisp Parser** - Handles atoms, lists, strings, numbers, booleans
- **Rust Code Generation** - Produces idiomatic Rust code
- **Built-in Operations** - Arithmetic, comparisons, conditionals
- **Error Handling** - Comprehensive parsing and compilation error messages
- **Fast Compilation** - Direct compilation to native Rust code

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

The compiler follows a traditional compilation pipeline:

1. **Lexer** (`src/lexer.rs`) - Tokenizes source code
2. **Parser** (`src/parser.rs`) - Builds Abstract Syntax Tree
3. **Compiler** (`src/compiler.rs`) - Generates Rust code
4. **CLI** (`src/main.rs`) - Command-line interface

## Testing

Run the test suite:
```bash
cargo test
```

Run with verbose output:
```bash
cargo test -- --nocapture
```
