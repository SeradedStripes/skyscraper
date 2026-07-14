# Skyscraper Roadmap

## Phase 1: Bootstrap Foundation

- [x] Project scaffolding (Rust cargo project)
- [ ] ISA subset definition (instructions needed for self-hosting)
- [ ] Lexer / tokenizer for Skyscraper assembly
- [ ] Parser producing AST
- [ ] x86-64 code generation (minimal instruction subset)
- [ ] ELF binary writer (Linux x86-64)
- [ ] End-to-end test (`exit(0)` running natively)

## Phase 2: Language Essentials

- [ ] Labels and jumps
- [ ] Function calls and returns
- [ ] Stack frame management
- [ ] String and data directives
- [ ] Basic error reporting (line numbers, messages)
- [ ] Multi-file programs (entry point, imports)

## Phase 3: Self-Hosting

- [ ] Expand ISA subset to express the compiler itself
- [ ] Implement enough I/O (file read/write via syscalls)
- [ ] Compile the bootstrap compiler from Skyscraper source
- [ ] Verify self-hosted binary works identically to Rust bootstrap

## Phase 4: Modular ISA

- [ ] Abstract backend trait for ISA implementations
- [ ] aarch64 backend (Linux)
- [ ] ISA versioning support
- [ ] Platform abstraction (Linux -> Windows/macOS)

## Phase 5: Ecosystem

- [ ] Assembler with proper error handling
- [ ] Linker for multi-object programs
- [ ] Standard library (minimal runtime)
- [ ] Documentation and examples
