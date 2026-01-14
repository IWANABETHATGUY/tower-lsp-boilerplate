# boilerplate for a rust language server powered by `tower-lsp`

> [!note]
> This repo uses [l-lang](https://github.com/IWANABETHATGUY/l-lang), a simple statically-typed programming language with structs, functions, and expressions.

> [!tip]
> If you want a `chumsky` based language implementation, please check out the tag [v1.0.0](https://github.com/IWANABETHATGUY/tower-lsp-boilerplate/tree/v1.0.0)

## A valid program in l-lang

```rust
struct Point {
    x: int,
    y: int,
}

struct Rectangle {
    top_left: Point,
    bottom_right: Point,
}

fn add_points(a: Point, b: Point) -> Point {
    return Point { x: a.x + b.x, y: a.y + b.y };
}

fn main() {
    let p1 = Point { x: 10, y: 20 };
    let p2 = Point { x: 5, y: 15 };
    let result = add_points(p1, p2);

    let rect = Rectangle {
        top_left: Point { x: 0, y: 100 },
        bottom_right: Point { x: 100, y: 0 },
    };

    return result;
}
```

## Language Features

L-lang is a statically-typed language that supports:

- **Struct definitions** with typed fields
- **Functions** with typed parameters and return types
- **Variable bindings** with type inference
- **Arithmetic operations** (+, -, \*, /)
- **Field access** for structs
- **Struct literals** for instantiation
- **Basic types**: `int`, `bool`, `string`

## Introduction

This repo is a template for building Language Server Protocol (LSP) implementations using `tower-lsp`, demonstrating how to create language servers with full IDE support.

## Development using VSCode

1. `pnpm i`
2. `cargo build`
3. Open the project in VSCode: `code .`
4. In VSCode, press <kbd>F5</kbd> or change to the Debug panel and click <kbd>Launch Client</kbd>.
5. In the newly launched VSCode instance, open the file `examples/test.nrs` from this project.
6. If the LSP is working correctly you should see syntax highlighting and the features described below should work.

> [!note]
> If encountered errors like `Cannot find module '/xxx/xxx/dist/extension.js'`
> please try run command `tsc -b` manually, you could refer https://github.com/IWANABETHATGUY/tower-lsp-boilerplate/issues/6 for more details

### Preview and test extension locally with `VsCode`

1. Make sure all dependency are installed.
2. Make sure the `nrs-language-server` is under your `PATH`
3. `pnpm run package`
4. `code --install-extension nrs-language-server-${version}.vsix`, the `version` you could inspect in file system.
5. Restart the `VsCode`, and write a minimal `nano rust` file, then inspect the effect.

For other editor, please refer the related manual, you could skip steps above.

## Features

This Language Server Protocol implementation for l-lang provides comprehensive IDE support with the following features:

- [x] **Semantic Tokens** - Syntax highlighting based on semantic analysis
  - Functions, variables, parameters, structs, and fields are highlighted according to their semantic roles
  - Make sure semantic highlighting is enabled in your editor settings:
    ```json
    {
      "editor.semanticHighlighting.enabled": true
    }
    ```

- [x] **Inlay Hints** - Type annotations for variables
      ![inlay hint](https://user-images.githubusercontent.com/17974631/156926412-c3823dac-664e-430e-96c1-c003a86eabb2.gif)

- [x] **Syntactic and Semantic Error Diagnostics** - Real-time error reporting

https://user-images.githubusercontent.com/17974631/156926382-a1c4c911-7ea1-4d3a-8e08-3cf7271da170.mp4

- [x] **Code Completion** - Context-aware suggestions for symbols

https://user-images.githubusercontent.com/17974631/156926355-010ef2cd-1d04-435b-bd1e-8b0dab9f44f1.mp4

- [x] **Go to Definition** - Navigate to symbol declarations

https://user-images.githubusercontent.com/17974631/156926103-94d90bd3-f31c-44e7-a2ce-4ddfde89bc33.mp4

- [x] **Find References** - Locate all usages of a symbol

https://user-images.githubusercontent.com/17974631/157367235-7091a36c-631a-4347-9c1e-a3b78db81714.mp4

- [x] **Rename** - Rename symbols across the entire codebase

https://user-images.githubusercontent.com/17974631/157367229-99903896-5583-4f67-a6da-1ae1cf206876.mp4

## Implementation Details

### Semantic Token Support

The LSP implementation provides full semantic token support using l-lang's semantic analysis:

- **Token Types**: Functions, Variables, Parameters, Structs, Fields (Properties)
- **Token Sources**: Both symbol definitions and references are highlighted
- **Delta Encoding**: Efficient LSP protocol format for token transmission
- **Range Support**: Both full document and range-based token requests

The implementation extracts semantic information from l-lang's two-pass analysis:

1. **Symbol Resolution Pass**: Collects all declarations and resolves references
2. **Type Checking Pass**: Infers types and validates semantics

All tokens are properly mapped from byte offsets to line/character positions using the Rope data structure for accurate highlighting.
