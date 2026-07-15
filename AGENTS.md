# Agent Instructions

## Project Overview

Skyscraper is a custom assembly language with its own ISA that compiles to multiple native CPU architectures (x86-64, aarch64). It features fixed 32-bit instruction width, 32 integer 64-bit registers, 16 vector 128-bit registers, and 16 mask 64-bit registers. The bootstrap compiler is written in Rust.

The project is in **Phase 1 (Bootstrap Foundation)**. The ISA specification is complete (v1.1), but the compiler is scaffolded only as a bare Rust project. See `Roadmap.md` for the full 5-phase plan.

## Build Commands

All commands run from the `bootstrap/` directory:

```bash
cargo build          # Build the bootstrap compiler
cargo run            # Run it
cargo check          # Type-check without producing binary
cargo test           # Run tests
cargo clippy         # Lint
cargo fmt --check    # Check formatting
cargo fmt            # Auto-format
```

There is no external dependencies, the crate has an empty `[dependencies]` section.

## Project Structure

```
skyscraper/
├── bootstrap/              # Rust bootstrap compiler
│   ├── Cargo.toml          # Package manifest (edition 2024)
│   └── src/
│       ├── main.rs         # Entry point
│       └── codegen/        # Code generation modules (planned)
├── ISA-spec.md             # Full ISA specification (reference)
├── skyscraper.asl          # Formal ISA spec in ASL
├── AI_POLICY.md            # AI usage policy for contributions
├── CONTRIBUTING.md         # Contribution guidelines
├── Roadmap.md              # Development roadmap
├── AGENTS.md               # This file
└── LICENSE.md              # Apache 2.0
```

## File Extensions

| Extension | Purpose |
|-----------|---------|
| `.sky`    | Skyscraper assembly source files |
| `.skyo`   | Assembled object files |
| `.skyb`   | Linked binary files |

## Code Architecture

### Planned Compiler Pipeline

```
.sky files → Lexer → Parser (AST) → Assembler (.skyo) → Linker (.skyb) → Platform Fixup → Native binary
```

### Planned Module Layout

- `src/main.rs` — CLI entry point
- `src/codegen/` — Code generation modules
- `src/isa/<arch>/<os>/` — Modular ISA backends (e.g., `isa/x86-64/linux`)

ISA backends are plug-in modules using trait-based abstraction. Adding a new architecture should not require changes to the core compiler.

## Conventions

### Rust

- Edition 2024
- No external dependencies (keep the bootstrap minimal)
- Standard `cargo` project layout

### ISA / Assembly

- Instructions: lowercase mnemonics (`add`, `ldi`, `jal`, `syscall`)
- Registers: lowercase with numeric suffix (`r0`–`r15`, `arg0`–`arg7`, `ret0`–`ret1`, `temp0`–`temp5`, `vec0`–`vec15`, `mask0`–`mask15`)
- Labels: `snake_case` with colon (`_start:`, `loop:`)
- Directives: dot-prefixed (`.text`, `.data`, `.string`)
- Instruction encoding: 7 format types (R/I/S/B/U/J/V), all fixed 32-bit width

### Documentation

- Specs live at the project root (`ISA-spec.md`, `skyscraper.asl`)
- ISA spec is the source of truth for language design
- The `.asl` file is the formal machine-readable ISA description

## Development Workflow

1. Read `ISA-spec.md` before implementing any language feature, it defines the semantics
2. The immediate next steps (per Roadmap.md Phase 1) are: lexer, parser, x86-64 code generation, ELF binary writer
3. Target platform is **Linux x86-64 first**; other platforms come later via modular backends
4. The end-to-end milestone for Phase 1 is: compile and run `exit(0)` natively

## Contributing

- See `CONTRIBUTING.md` for fork/clone/PR workflow
- See `AI_POLICY.md`, all AI usage must be disclosed in PR descriptions
- PRs must include evidence the code works (screen recording, screenshot, or test output)
- License: Apache 2.0 for code, CC BY 4.0 for the ISA specification

## Note:

If I see a single em dash in any PR, I will cry. It will still be merged...