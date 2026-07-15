# Skyscraper

## What is Skyscraper?

Skyscraper is an assembler featuring its own ISA (Instruction Set Architecture), however, it compiles to x86-64, aarch64 and  more!

## Design Goals

- Architecture-independent instruction set.
- Portable across multiple native CPU architectures.
- Predictable instruction semantics.
- Efficient native code generation.
- Register-oriented execution model.
- Stable binary format.
- Extensible through future ISA revisions.
- Future ISA-spec additions through proper versioning and backward compatibility.

## Why it was made?

~~Why not?~~  
Skyscraper was made as who wants to learn multiple ISAs when you can just learn one and compile to multiple?

## Stack

- Skyscraper will have a bootstrap compiler written in Rust, which will compile to x86-64 and aarch64.
- Once the bootstrap compiler is complete, Skyscraper will be able to compile itself to x86-64 and aarch64.

## Restrictions

- Skyscraper needs to be super modular, ie:
  - You go to lets say isa/ and then you can add more instruction sets to the compiler to compile to more ISAs. Without having to change the compiler itself.
  - The compiler should be able to compile itself to any ISA that is supported by the compiler.
- We need to be able to compile to linux first, then windows and macos.
  - This should be modular too, ie. isa/x86-64/linux, isa/x86-64/windows, isa/x86-64/macos, etc.

## License

Skyscraper is licensed under Apache License 2.0. See LICENSE for more information.  
The ISAs that Skyscraper compiles to are licensed under their own licenses, check the license file in each ISA folder for more information.  
Skyscraper's language specification is licensed under Creative Commons Attribution 4.0 International License (CC BY 4.0)  
