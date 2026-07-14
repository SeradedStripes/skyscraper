# ISA specification for Skyscraper

## Registers

- `r0-r128`: General purpose registers
- `sp`: Stack pointer
- `fp`: Frame pointer
- `pc`: Program counter
- `lr`: Link register
- `flags`: Flags register (for condition codes)
- `zero`: Zero register (always reads as 0)
- `one`: One register (always reads as 1)
- `temp0-temp15`: Temporary registers for intermediate calculations
- `arg0-arg128`: Argument registers for function calls
- `ret0-ret128`: Return value registers for function calls
- `status`: Status register (for system-level information)
- `control`: Control register (for system-level control)
- `vector0-vector15`: Vector registers for SIMD operations
- `mask0-mask15`: Mask registers for conditional operations

## Data Types

- i8
- i16
- i32
- i64
- i128
- u8
- u16
- u32
- u64
- u128
- f16
- f32
- f64
- bool

## Memory model

- Little-endian
- Byte-addressable
- Flat virtual address space
- Alignment rules
- Memory protection

## Instruction format

```
OPCODE DEST SRC1 SRC2 SRC3
```
Note: SRC3 is optional

Example:
```
mov r0, 5
add r1, r0, r2
load r3, [r4]
store [r5], r6
```

## Calling Convention

### Function Arguments

Arguments are passed with the `arg` registers.

Example:
```
arg0 = first argument
arg1 = second argument
...
```

If the number of arguments exceed the number of argument registers, the remaining arguments are passed on the stack.  
But you should not have that many arguments :sob:

### Return Values

Functions return values through the `ret` registers.

```
ret0 = primary return value
ret1 = secondary return value
...
```

Multiple return values are supported

### Stack

The stack grows downward.

The `sp` register always points to the current top of the stack.

The `fp` register is optional and may be omitted by optimized code.

The `lr` register stores the return address after a function call.

## Stack Layout

Typical Stack layout:

```
Higher addresses
+----------------------+
| Function arguments   |
+----------------------+
| Return address       |
+----------------------+
| Previous frame ptr   |
+----------------------+
| Local variables      |
+----------------------+
| Temporary storage    |
+----------------------+
Lower addresses
```

## Condition Codes

The `flags` register stores the results of comparison and arithmetic instructions.

Supported flags:

- Z (Zero)
- N (Negative)
- C (Carry)
- O (Overflow)
- P (Parity)

Comparison Instructions update the flags regiser

## Branch Instructions

Conditional branches evaluate the current flag state.

Core instructions:

```
jmp label
je label
jne label
jg label
jge label
jl label
jle label
call function
ret
```

Implementations may lower these into architecture-specific instructions.

## Arithmetic Instructions

Core arithmetic instructions include:

```
add
sub
mul
div
mod
inc
dec
neg
abs
```

Bitwise instructions:

```
and
or
xor
not
shl
shr
rol
ror
```

Comparison instructions:

```
cmp
test
```

Arithmetic instructions update the flags register unless otherwise specified.

## Memory Instructions

Memory is accessed through explicit load/store instructions.

Examples:

```
load r0, [r1]
store [r2], r3

load8
load16
load32
load64
load128

store8
store16
store32
store64
store128
```

Memory accesses must respect alignment requirements unless otherwise documented.

## SIMD Instructions

Vector registers (`vector0-vector15`) support SIMD operations.

Examples:

```
vadd
vsub
vmul
vdiv
vand
vor
vxor
vload
vstore
```

Vector width is implementation-defined and mapped to the target architecture where possible.

## System Instructions

Privileged instructions include:

```
syscall
break
halt
nop
yield
fence
```

Implementations may restrict privileged instructions in user mode.

## Binary Encoding

Instructions are encoded into machine-independent bytecode before backend translation.

Each instruction contains:

```
Opcode
Operand Count
Destination Register
Source Registers
Immediate Values (optional)
Flags (optional)
```

The binary format is versioned to ensure compatibility between assembler and runtime.

## ABI

The Application Binary Interface defines:

- Register usage
- Calling convention
- Stack layout
- Function prologue and epilogue
- Symbol visibility
- Object file layout

All backends must conform to the Skyscraper ABI while generating architecture-specific binaries.

## Backend Requirements

A backend must provide:

- Register allocation
- Instruction lowering
- Calling convention mapping
- ABI compliance
- Binary generation
- Relocation support
- Debug information generation (optional)

Target architectures currently include:

Nada, havent done them yet

Additional backends may be implemented without modifying the ISA.

## Reserved Opcodes

Reserved opcodes are allocated for future ISA revisions.

Assemblers must reject reserved instructions.

Backends must treat unknown opcodes as invalid.

Reserved instruction ranges may later be assigned to:

- Cryptographic extensions
- AI acceleration
- Advanced SIMD
- Debugging
- Virtualization
- Experimental instructions