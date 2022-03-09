# boilerplate for a  rust language server powered by `tower-lsp` 
## Introduction
This repo is a template for `tower-lsp`, a crate that let you write a language server more easily.
## Development
1. `pnpm i`
2. `cargo build`
3. press `F5` or change to the Debug panel and click `Launch Client`
## Features
This repo use a language `nano rust` which first introduced by [ chumsky ](https://github.com/zesterer/chumsky/blob/master/examples/nano_rust.rs). Most common language feature has been implemented, you could preview via the video below.

- [x] InlayHint for LiteralType
![inlay hint](https://user-images.githubusercontent.com/17974631/156926412-c3823dac-664e-430e-96c1-c003a86eabb2.gif)

- [x] semantic token

- [x] syntactic error diagnostic

https://user-images.githubusercontent.com/17974631/156926382-a1c4c911-7ea1-4d3a-8e08-3cf7271da170.mp4

- [x] code completion  

https://user-images.githubusercontent.com/17974631/156926355-010ef2cd-1d04-435b-bd1e-8b0dab9f44f1.mp4

- [x] go to definition  

https://user-images.githubusercontent.com/17974631/156926103-94d90bd3-f31c-44e7-a2ce-4ddfde89bc33.mp4

- [x] find reference

https://user-images.githubusercontent.com/17974631/157367235-7091a36c-631a-4347-9c1e-a3b78db81714.mp4

- [x] rename

https://user-images.githubusercontent.com/17974631/157367229-99903896-5583-4f67-a6da-1ae1cf206876.mp4







