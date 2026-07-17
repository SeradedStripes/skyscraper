# Skyscraper Roadmap

## Phase 1: Bootstrap Foundation

- [x] Project scaffolding (Rust cargo project)
- [x] ISA subset definition (instructions needed for self-hosting) — ISA-spec.md v1.1 complete (32 integer regs, 16 vec regs, 16 mask regs, 5-bit register encoding)
- [x] Lexer/parser design (sections 16-17 of ISA-spec.md) — token types, grammar, AST, two-pass assembly
- [x] Project structure and config format (sections 18-21) — `.sky` files, `skyscraper.conf`, `target/` layout, build commands
- [x] Lexer / tokenizer for Skyscraper assembly
- [x] Parser producing AST
- [x] x86-64 code generation (minimal instruction subset)
- [x] ELF binary writer (Linux x86-64)
- [x] End-to-end test (`exit(0)` running natively)

## Phase 2: Language Essentials

- [x] Labels and jumps (labels defined in code and data, label references in immediates)
- [x] Constants (`$` and `.` for current address, `$name` for named constants)
- [x] String and data directives (`.string`, `.text`, `.data`, `.bss`)
- [x] Control flow (beq, bne, blt, bge, bltu, bgeu, ble, bgt, bzs, bnz, j, jal, jr, jalr, ret)
- [x] Arithmetic/logic (add, sub, and, or, xor, not, neg, shl, shr + imm variants)
- [x] Comparison (cmp, cmpi)
- [x] Memory (ld, sd with base+offset addressing)
- [x] Function calls and returns (jal/ret working, need stack frame convention)
- [x] Stack frame management
- [x] Basic error reporting (line numbers, messages)
- [x] Multi-file programs (entry point, imports)

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
