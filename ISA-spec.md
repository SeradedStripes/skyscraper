<!-- Licensed under Creative Commons Attribution 4.0 International License -->
# ISA Specification for Skyscraper

**Version:** 1.1
**Status:** Draft

---

## 1. Overview

Skyscraper ISA is a RISC-style, architecture-independent instruction set designed to compile efficiently to native CPU architectures (x86-64, aarch64, and future targets). It uses fixed-width 32-bit instructions, a flat memory model, and a register-oriented execution model.

Key properties:

- Fixed 32-bit instruction width
- 32 integer 64-bit registers (split into purpose-defined classes)
- 16 vec 128-bit registers
- 16 mask 64-bit registers
- Byte-addressable, little-endian memory
- Load-store architecture (memory accessed only via load/store instructions)
- Condition codes set by arithmetic and comparison instructions
- PC-relative addressing for branches and calls

---

## 2. Data Types

| Type       | Size     | Description                     |
|------------|----------|---------------------------------|
| `byte`     | 8-bit    | Unsigned byte                   |
| `sbyte`    | 8-bit    | Signed byte                     |
| `half`     | 16-bit   | Unsigned halfword               |
| `shalf`    | 16-bit   | Signed halfword                 |
| `word`     | 32-bit   | Unsigned word                   |
| `sword`    | 32-bit   | Signed word                     |
| `dword`    | 64-bit   | Unsigned doubleword             |
| `sdword`   | 64-bit   | Signed doubleword               |
| `single`   | 32-bit   | IEEE 754 single-precision float |
| `double`   | 64-bit   | IEEE 754 double-precision float |
| `ptr`      | 64-bit   | Pointer (architecture address)  |

All values are stored in little-endian byte order. Operations default to 64-bit (dword) unless otherwise specified by a type suffix.

---

## 3. Registers

Skyscraper has three register files: a 32-entry integer file, a 16-entry vec file, and a16-entry mask file. Integer registers are split into purpose-defined classes. The register encoding uses 5-bit fields for integer registers and 4-bit fields for vec/mask registers.

### 3.1 Integer Registers (64-bit)

| Register | Physical Index | Class    | Notes                                    |
|----------|----------------|----------|------------------------------------------|
| `r0`     | 0              | GPR      | General-purpose                          |
| `r1`     | 1              | GPR      | General-purpose                          |
| `r2`     | 2              | GPR      | General-purpose                          |
| `r3`     | 3              | GPR      | General-purpose                          |
| `r4`     | 4              | GPR      | General-purpose                          |
| `r5`     | 5              | GPR      | General-purpose                          |
| `r6`     | 6              | GPR      | General-purpose                          |
| `r7`     | 7              | GPR      | General-purpose                          |
| `r8`     | 8              | GPR      | General-purpose                          |
| `r9`     | 9              | GPR      | General-purpose                          |
| `r10`    | 10             | GPR      | General-purpose                          |
| `r11`    | 11             | GPR      | General-purpose                          |
| `r12`    | 12             | GPR      | General-purpose                          |
| `r13`    | 13             | GPR      | General-purpose                          |
| `r14`    | 14             | GPR      | General-purpose                          |
| `r15`    | 15             | GPR      | General-purpose                          |
| `arg0`   | 16             | Argument | Function argument 0                      |
| `arg1`   | 17             | Argument | Function argument 1                      |
| `arg2`   | 18             | Argument | Function argument 2                      |
| `arg3`   | 19             | Argument | Function argument 3                      |
| `arg4`   | 20             | Argument | Function argument 4                      |
| `arg5`   | 21             | Argument | Function argument 5                      |
| `arg6`   | 22             | Argument | Function argument 6                      |
| `arg7`   | 23             | Argument | Function argument 7                      |
| `ret0`   | 24             | Return   | Return value 0                           |
| `ret1`   | 25             | Return   | Return value 1                           |
| `temp0`  | 26             | Temp     | Caller-saved temporary                   |
| `temp1`  | 27             | Temp     | Caller-saved temporary                   |
| `temp2`  | 28             | Temp     | Caller-saved temporary                   |
| `temp3`  | 29             | Temp     | Caller-saved temporary                   |
| `temp4`  | 30             | Temp     | Caller-saved temporary                   |
| `temp5`  | 31             | Temp     | Caller-saved temporary                   |

### 3.2 Special-Purpose Registers

These registers are not accessible via the 5-bit register field in instructions. They are accessed through dedicated instructions (`mfsp`/`mtsp`, `mffp`/`mtfp`, `mflr`/`mtlr`, etc.).

| Register  | Description                                  |
|-----------|----------------------------------------------|
| `sp`      | Stack pointer                                |
| `fp`      | Frame pointer                                |
| `pc`      | Program counter                              |
| `lr`      | Link register                                |
| `flags`   | Condition flags (N, Z, C, V)                 |
| `zero`    | Hardwired to 0 on read, writes discarded     |
| `one`     | Hardwired to 1 on read, writes discarded     |

### 3.3 Vector Registers (128-bit)

Vector registers hold 128-bit SIMD data. Each vec register can be viewed as:
- 2x 64-bit elements (`.2d`)
- 4x 32-bit elements (`.4s`)
- 8x 16-bit elements (`.8h`)
- 16x 8-bit elements (`.16b`)

| Register   | Index | Notes                          |
|------------|-------|--------------------------------|
| `vec0`  | 0     | Vector register 0              |
| `vec1`  | 1     | Vector register 1              |
| `vec2`  | 2     | Vector register 2              |
| `vec3`  | 3     | Vector register 3              |
| `vec4`  | 4     | Vector register 4              |
| `vec5`  | 5     | Vector register 5              |
| `vec6`  | 6     | Vector register 6              |
| `vec7`  | 7     | Vector register 7              |
| `vec8`  | 8     | Vector register 8              |
| `vec9`  | 9     | Vector register 9              |
| `vec10` | 10    | Vector register 10             |
| `vec11` | 11    | Vector register 11             |
| `vec12` | 12    | Vector register 12             |
| `vec13` | 13    | Vector register 13             |
| `vec14` | 14    | Vector register 14             |
| `vec15` | 15    | Vector register 15             |

### 3.4 Mask Registers (64-bit)

Mask registers are used for conditional vec operations and predicated execution.

| Register  | Index | Notes                          |
|-----------|-------|--------------------------------|
| `mask0`   | 0     | Mask register 0                |
| `mask1`   | 1     | Mask register 1                |
| `mask2`   | 2     | Mask register 2                |
| `mask3`   | 3     | Mask register 3                |
| `mask4`   | 4     | Mask register 4                |
| `mask5`   | 5     | Mask register 5                |
| `mask6`   | 6     | Mask register 6                |
| `mask7`   | 7     | Mask register 7                |
| `mask8`   | 8     | Mask register 8                |
| `mask9`   | 9     | Mask register 9                |
| `mask10`  | 10    | Mask register 10               |
| `mask11`  | 11    | Mask register 11               |
| `mask12`  | 12    | Mask register 12               |
| `mask13`  | 13    | Mask register 13               |
| `mask14`  | 14    | Mask register 14               |
| `mask15`  | 15    | Mask register 15               |

### 3.5 Register Class Summary

| Class     | Registers           | Count | Encoding | Saved By     | Usage                              |
|-----------|---------------------|-------|----------|--------------|------------------------------------|
| GPR       | `r0`-`r15`          | 16    | 5-bit    | Varies       | General-purpose computation        |
| Argument  | `arg0`-`arg7`       | 8     | 5-bit    | Caller       | Function arguments                 |
| Return    | `ret0`-`ret1`       | 2     | 5-bit    | Caller       | Function return values             |
| Temp      | `temp0`-`temp5`     | 6     | 5-bit    | Caller       | Scratch / temporaries              |
| Vector    | `vec0`-`vec15`| 16    | 4-bit    | Caller       | SIMD data                          |
| Mask      | `mask0`-`mask15`    | 16    | 4-bit    | Caller       | Conditional / predicated ops       |
| Special   | `sp`,`fp`,`pc`,`lr`,`flags`,`zero`,`one` | 7 | dedicated | Callee (sp,fp,lr) | System/control           |

### 3.6 Virtual Registers

Virtual registers are a compiler abstraction, not part of the hardware ISA. The register allocator maps virtual registers to physical registers and spills to the stack when pressure is high.

- `v0`-`v31`: Virtual registers, allocated by the compiler. Mapped to any available physical integer register (`r0`-`r15`, `arg0`-`arg7`, `ret0`-`ret1`, `temp0`-`temp5`) when free; spilled to `[fp - offset]` when no physical register is available.

---

## 4. Memory Model

- **Address space:** 64-bit flat address space
- **Alignment:** Natural alignment required (1-byte for byte, 2-byte for half, 4-byte for word, 8-byte for dword)
- **Endianness:** Little-endian
- **Memory access:** Only through `load` and `store` instructions (load-store architecture)
- **Stack:** Grows downward (toward lower addresses). `sp` points to the top of the stack (last pushed item). Stack must be 16-byte aligned at all times.

### 4.1 Addressing Modes

| Mode                 | Syntax             | Description                                |
|----------------------|--------------------|--------------------------------------------|
| Register direct      | `r1`               | Effective address is in register           |
| Register + immediate | `[r1 + imm9]`      | Base + 9-bit signed offset                 |
| Register + register  | `[r1 + r2]`        | Base + index (scaled by access size)       |
| PC-relative          | `[pc + imm12]`     | PC + 12-bit signed offset (for branches)   |
| Absolute (pseudo)    | `[imm32]`          | Synthesized via `lui` + `load` pair        |

---

## 5. Instruction Encoding

All instructions are 32 bits wide. Integer register fields use 5-bit encoding (32 registers). Vector/mask register fields use 4-bit encoding (16 registers).

### 5.1 Format Types

**R-type (Register-Register):**

```
 31      28 27    24 23   19 18   14 13    9 8  7  6              0
+----------+--------+------+------+--------+----+----------------+
| opcode   | func   | Rd   | Rs1  | Rs2    | 0  |    7 bits res  |
+----------+--------+------+------+--------+----+----------------+
  4 bits    4 bits   5 bits 5 bits 5 bits  1 bit  7 bits (reserved)
```

**I-type (Immediate):**

```
 31      28 27    24 23   19 18   14 13                     5 4    0
+----------+--------+------+------+------------------------+------+
| opcode   | func   | Rd   | Rs1  |        imm9            | 0000 |
+----------+--------+------+------+------------------------+------+
  4 bits    4 bits   5 bits 5 bits 9 bits (signed)          4 bits(res)
```

**S-type (Store):**

```
 31      28 27    24 23   19 18   14 13                     5 4    0
+----------+--------+------+------+------------------------+------+
| opcode   | func   | Rs2  | Rs1  |        imm9            | 0000 |
+----------+--------+------+------+------------------------+------+
  4 bits    4 bits   5 bits 5 bits 9 bits (signed)          4 bits(res)
```

**B-type (Branch):**

```
 31      28 27    24 23                    16 15   11 10       5 4    0
+----------+--------+----------------------+--------+-----------+------+
| opcode   | cond   |      imm12           | Rs1    |   00000   | 0000 |
+----------+--------+----------------------+--------+-----------+------+
  4 bits    4 bits   12 bits (PC-relative)  5 bits   5 bits(res) 4 bits(res)
```

**U-type (Upper Immediate):**

```
 31      28 27    24 23   19 18                                  0
+----------+--------+------+------------------------------------+
| opcode   | func   | Rd   |           imm19                    |
+----------+--------+------+------------------------------------+
  4 bits    4 bits   5 bits 19 bits (unsigned, shifted left 12)
```

**J-type (Jump):**

```
 31      28 27    24 23                                        0
+----------+--------+----------------------------------------+
| opcode   | func   |           imm24 (PC-relative)           |
+----------+--------+----------------------------------------+
  4 bits    4 bits   24 bits (signed, shifted left 2)
```

**V-type (Vector):**

```
 31      28 27    24 23   19 18   14 13    9 8  7  6  5  4    0
+----------+--------+------+------+--------+----+----+----+----+
| opcode   | func   | Vd   | Vs1  | Vs2    | Ms | 0  | 0  | 0  |
+----------+--------+------+------+--------+----+----+----+----+
  4 bits    4 bits   4 bits 4 bits 4 bits  4 bits 4 bits(reserved)
```

### 5.2 Opcode Map

| Opcode | Binary | Category        | Description                    |
|--------|--------|-----------------|--------------------------------|
| `NOP`  | `0000` | Control         | No operation                   |
| `ALU`  | `0001` | Arithmetic/Logic| ALU operations (R/I-type)      |
| `MEM`  | `0010` | Memory          | Load/Store (I/S-type)          |
| `BR`   | `0011` | Branch          | Conditional branches (B-type)  |
| `JMP`  | `0100` | Jump            | Unconditional jump/call (J-type)|
| `LUI`  | `0101` | Upper Immediate | Load upper immediate (U-type)  |
| `SYSCALL`| `0110`| System         | System call / trap             |
| `VEC`  | `0111` | SIMD/Vector     | Vector operations (V-type)     |
| `CMP`  | `1000` | Comparison      | Compare and set flags (R/I-type)|
| `MOV`  | `1001` | Data Movement   | Register move / sign-extend    |
| `LDI`  | `1010` | Load Immediate  | Load 9-bit immediate (I-type)  |
| `MASK` | `1011` | Mask            | Mask register operations       |
| `UNUSED`| `1100`-`1111` | Reserved  | Reserved for future use        |

---

## 6. Instruction Set

### 6.1 ALU Operations (`ALU` opcode)

All ALU operations write the result to `Rd`. The `func` field selects the operation. Register fields use 5-bit encoding (any of the 32 integer registers).

#### R-type (register-register)

| Instruction           | Func | Operation             | Flags Set     |
|-----------------------|------|-----------------------|---------------|
| `add Rd, Rs1, Rs2`    | `0000` | `Rd = Rs1 + Rs2`     | N, Z, C, V   |
| `sub Rd, Rs1, Rs2`    | `0001` | `Rd = Rs1 - Rs2`     | N, Z, C, V   |
| `mul Rd, Rs1, Rs2`    | `0010` | `Rd = Rs1 * Rs2`     | N, Z         |
| `div Rd, Rs1, Rs2`    | `0011` | `Rd = Rs1 / Rs2`     | N, Z         |
| `mod Rd, Rs1, Rs2`    | `0100` | `Rd = Rs1 % Rs2`     | N, Z         |
| `and Rd, Rs1, Rs2`    | `0101` | `Rd = Rs1 & Rs2`     | N, Z         |
| `or Rd, Rs1, Rs2`     | `0110` | `Rd = Rs1 | Rs2`     | N, Z         |
| `xor Rd, Rs1, Rs2`    | `0111` | `Rd = Rs1 ^ Rs2`     | N, Z         |
| `shl Rd, Rs1, Rs2`    | `1000` | `Rd = Rs1 << Rs2`    | N, Z, C      |
| `shr Rd, Rs1, Rs2`    | `1001` | `Rd = Rs1 >> Rs2`    | N, Z, C      |
| `sar Rd, Rs1, Rs2`    | `1010` | `Rd = Rs1 >>> Rs2`   | N, Z, C      |
| `neg Rd, Rs1`         | `1011` | `Rd = -Rs1`          | N, Z, V      |
| `not Rd, Rs1`         | `1100` | `Rd = ~Rs1`          | N, Z         |

#### I-type (register-immediate)

9-bit signed immediate range: -256 to 255. For larger values, use `lui` + `ori` sequences.

| Instruction             | Func | Operation                  | Flags Set     |
|-------------------------|------|----------------------------|---------------|
| `addi Rd, Rs1, imm9`    | `0000` | `Rd = Rs1 + sign(imm9)`  | N, Z, C, V   |
| `subi Rd, Rs1, imm9`    | `0001` | `Rd = Rs1 - sign(imm9)`  | N, Z, C, V   |
| `muli Rd, Rs1, imm9`    | `0010` | `Rd = Rs1 * sign(imm9)`  | N, Z         |
| `divi Rd, Rs1, imm9`    | `0011` | `Rd = Rs1 / sign(imm9)`  | N, Z         |
| `modi Rd, Rs1, imm9`    | `0100` | `Rd = Rs1 % sign(imm9)`  | N, Z         |
| `andi Rd, Rs1, imm9`    | `0101` | `Rd = Rs1 & zext(imm9)`  | N, Z         |
| `ori Rd, Rs1, imm9`     | `0110` | `Rd = Rs1 | zext(imm9)`  | N, Z         |
| `xori Rd, Rs1, imm9`    | `0111` | `Rd = Rs1 ^ zext(imm9)`  | N, Z         |
| `shli Rd, Rs1, imm9`    | `1000` | `Rd = Rs1 << imm9`       | N, Z, C      |
| `shri Rd, Rs1, imm9`    | `1001` | `Rd = Rs1 >> imm9`       | N, Z, C      |
| `sari Rd, Rs1, imm9`    | `1010` | `Rd = Rs1 >>> imm9`      | N, Z, C      |

### 6.2 Memory Operations (`MEM` opcode)

#### Loads (I-type)

| Instruction               | Func | Operation                             |
|---------------------------|------|---------------------------------------|
| `lb Rd, [Rs1 + imm9]`     | `0000` | Load byte, zero-extend to 64-bit     |
| `lh Rd, [Rs1 + imm9]`     | `0001` | Load half, zero-extend to 64-bit     |
| `lw Rd, [Rs1 + imm9]`     | `0010` | Load word, zero-extend to 64-bit     |
| `ld Rd, [Rs1 + imm9]`     | `0011` | Load doubleword (64-bit)             |
| `lbs Rd, [Rs1 + imm9]`    | `0100` | Load byte, sign-extend to 64-bit     |
| `lhs Rd, [Rs1 + imm9]`    | `0101` | Load half, sign-extend to 64-bit     |
| `lws Rd, [Rs1 + imm9]`    | `0110` | Load word, sign-extend to 64-bit     |
| `lfs Rd, [Rs1 + imm9]`    | `0111` | Load single-precision float          |
| `lfd Rd, [Rs1 + imm9]`    | `1000` | Load double-precision float          |

#### Stores (S-type)

| Instruction                | Func | Operation                             |
|----------------------------|------|---------------------------------------|
| `sb Rs2, [Rs1 + imm9]`     | `0000` | Store byte                           |
| `sh Rs2, [Rs1 + imm9]`     | `0001` | Store half                           |
| `sw Rs2, [Rs1 + imm9]`     | `0010` | Store word                           |
| `sd Rs2, [Rs1 + imm9]`     | `0011` | Store doubleword                     |
| `sfs Rs2, [Rs1 + imm9]`    | `0100` | Store single-precision float         |
| `sfd Rs2, [Rs1 + imm9]`    | `0101` | Store double-precision float         |

### 6.3 Branch Operations (`BR` opcode)

Branches use B-type encoding. The `cond` field selects the condition. The `imm12` field is a PC-relative offset in bytes (signed, aligned to 4 bytes). The branch target is `PC + sign(imm12) << 2`. Register fields use 5-bit encoding.

| Instruction             | Cond  | Condition                              |
|-------------------------|-------|----------------------------------------|
| `beq Rs1, imm12`        | `0000` | Branch if `flags.Z == 1`              |
| `bne Rs1, imm12`        | `0001` | Branch if `flags.Z == 0`              |
| `blt Rs1, imm12`        | `0010` | Branch if `flags.N != flags.V`        |
| `bge Rs1, imm12`        | `0011` | Branch if `flags.N == flags.V`        |
| `bltu Rs1, imm12`       | `0100` | Branch if `flags.C == 0` (unsigned <) |
| `bgeu Rs1, imm12`       | `0101` | Branch if `flags.C == 1` (unsigned >=)|
| `ble Rs1, imm12`        | `0110` | Branch if `flags.Z == 1` or `flags.N != flags.V` |
| `bgt Rs1, imm12`        | `0111` | Branch if `flags.Z == 0` and `flags.N == flags.V` |
| `bzs Rs1, imm12`        | `1000` | Branch if `Rs1 == 0` (branch-if-zero, no flags) |
| `bnz Rs1, imm12`        | `1001` | Branch if `Rs1 != 0` (branch-if-not-zero, no flags) |

**Note:** Most branches test the flags register. `bzs`/`bnz` test the value of `Rs1` directly without affecting or reading flags.

### 6.4 Jump Operations (`JMP` opcode)

| Instruction      | Func | Operation                                    |
|------------------|------|----------------------------------------------|
| `j imm24`         | `0000` | `PC = PC + sign(imm24) << 2`                |
| `jal imm24`       | `0001` | `lr = PC + 4; PC = PC + sign(imm24) << 2`   |
| `jr Rs1`          | `0010` | `PC = Rs1` (register indirect jump)         |
| `jalr Rs1`        | `0011` | `lr = PC + 4; PC = Rs1` (register indirect call) |
| `ret`             | `0100` | `PC = lr` (return from function)            |

### 6.5 Load Upper Immediate (`LUI` opcode)

| Instruction     | Func | Operation                                      |
|-----------------|------|------------------------------------------------|
| `lui Rd, imm19`  | `0000` | `Rd = imm19 << 12` (load upper 19 bits)       |

Used in combination with `ori` to load a full 32-bit constant:
```
lui ret0, upper19
ori ret0, ret0, lower12
```

### 6.6 Load Immediate (`LDI` opcode)

| Instruction      | Func | Operation                                      |
|------------------|------|------------------------------------------------|
| `ldi Rd, imm9`    | `0000` | `Rd = sign(imm9)` (load signed 9-bit)         |
| `ldiu Rd, imm9`   | `0001` | `Rd = zext(imm9)` (load unsigned 9-bit)       |

For loading larger constants, use `lui` + `ori` or `lui` + `ldi` sequences.

### 6.7 Comparison Operations (`CMP` opcode)

Comparisons set the `flags` register. The result is also written to `Rd`.

#### R-type

| Instruction           | Func | Operation                               | Flags Set     |
|-----------------------|------|-----------------------------------------|---------------|
| `cmp Rd, Rs1, Rs2`    | `0000` | `Rd = (Rs1 - Rs2); set flags`         | N, Z, C, V   |
| `tst Rd, Rs1, Rs2`    | `0001` | `Rd = (Rs1 & Rs2); set flags`         | N, Z         |

#### I-type

| Instruction             | Func | Operation                                     | Flags Set     |
|-------------------------|------|-----------------------------------------------|---------------|
| `cmpi Rd, Rs1, imm9`    | `0000` | `Rd = (Rs1 - sign(imm9)); set flags`        | N, Z, C, V   |
| `tsti Rd, Rs1, imm9`    | `0001` | `Rd = (Rs1 & zext(imm9)); set flags`        | N, Z         |

### 6.8 Data Movement (`MOV` opcode)

| Instruction       | Func | Operation                                        |
|-------------------|------|--------------------------------------------------|
| `mv Rd, Rs1`       | `0000` | `Rd = Rs1`                                      |
| `seb Rd, Rs1`      | `0001` | `Rd = sign_extend(Rs1[7:0])` (sign-extend byte)   |
| `seh Rd, Rs1`      | `0010` | `Rd = sign_extend(Rs1[15:0])` (sign-extend half)  |
| `sew Rd, Rs1`      | `0011` | `Rd = sign_extend(Rs1[31:0])` (sign-extend word)  |
| `zlb Rd, Rs1`      | `0100` | `Rd = zero_extend(Rs1[7:0])` (zero-extend byte)   |
| `zlh Rd, Rs1`      | `0101` | `Rd = zero_extend(Rs1[15:0])` (zero-extend half)  |
| `zlw Rd, Rs1`      | `0110` | `Rd = zero_extend(Rs1[31:0])` (zero-extend word)  |
| `mfpc Rd`          | `0111` | `Rd = pc` (read program counter)               |
| `mtpc Rs1`         | `1000` | `pc = Rs1` (write program counter, privileged) |
| `mflr Rd`          | `1001` | `Rd = lr` (read link register)                 |
| `mtlr Rs1`         | `1010` | `lr = Rs1` (write link register)               |
| `mfsp Rd`          | `1011` | `Rd = sp` (read stack pointer)                 |
| `mtsp Rs1`         | `1100` | `sp = Rs1` (write stack pointer)               |
| `mffp Rd`          | `1101` | `Rd = fp` (read frame pointer)                 |
| `mtfp Rs1`         | `1110` | `fp = Rs1` (write frame pointer)               |

### 6.9 Vector/SIMD Operations (`VEC` opcode)

Vector operations use V-type encoding with 4-bit vec register fields. Each vec register is128 bits and can hold 2x 64-bit, 4x 32-bit, 8x 16-bit, or 16x 8-bit elements. The `func` field selects the operation. The optional `Ms` field selects a mask register for predicated operations (0 = no masking).

| Instruction                   | Func | Operation                                    |
|-------------------------------|------|----------------------------------------------|
| `vadd.v Vd, Vs1, Vs2`         | `0000` | Vector add (element-wise)                   |
| `vsub.v Vd, Vs1, Vs2`         | `0001` | Vector subtract (element-wise)              |
| `vmul.v Vd, Vs1, Vs2`         | `0010` | Vector multiply (element-wise)              |
| `vdiv.v Vd, Vs1, Vs2`         | `0011` | Vector divide (element-wise)                |
| `vand.v Vd, Vs1, Vs2`         | `0100` | Vector bitwise AND                          |
| `vor.v Vd, Vs1, Vs2`          | `0101` | Vector bitwise OR                           |
| `vxor.v Vd, Vs1, Vs2`         | `0110` | Vector bitwise XOR                          |
| `vnot.v Vd, Vs1`              | `0111` | Vector bitwise NOT                          |
| `vld.b Vd, [Rs1 + imm9]`      | `1000` | Load 128-bit vec from memory (byte)      |
| `vst.b Vs2, [Rs1 + imm9]`     | `1001` | Store 128-bit vec to memory (byte)       |
| `vld.w Vd, [Rs1 + imm9]`      | `1010` | Load 128-bit vec from memory (word)      |
| `vst.w Vs2, [Rs1 + imm9]`     | `1011` | Store 128-bit vec to memory (word)       |
| `vdup.b Vd, Rs1`              | `1100` | Broadcast scalar byte to all 16 lanes       |
| `vdup.h Vd, Rs1`              | `1101` | Broadcast scalar half to all 8 lanes        |
| `vdup.w Vd, Rs1`              | `1110` | Broadcast scalar word to all 4 lanes        |
| `vdup.d Vd, Rs1`              | `1111` | Broadcast scalar dword to all 2 lanes       |

**Note:** Vector load/store instructions use integer registers for address computation (`Rs1`) and vec registers for data (`Vd`/`Vs2`).

### 6.10 Mask Operations (`MASK` opcode)

Mask operations manipulate mask registers for conditional/predicated execution.

| Instruction            | Func | Operation                                    |
|------------------------|------|----------------------------------------------|
| `mset Rd, imm9`        | `0000` | Set mask register from immediate             |
| `mand Md, Ms1, Ms2`    | `0001` | `Md = Ms1 & Ms2` (mask AND)                 |
| `mor Md, Ms1, Ms2`     | `0010` | `Md = Ms1 | Ms2` (mask OR)                  |
| `mxor Md, Ms1, Ms2`    | `0011` | `Md = Ms1 ^ Ms2` (mask XOR)                 |
| `mnot Md, Ms1`         | `0100` | `Md = ~Ms1` (mask NOT)                      |
| `mrd Rd, Ms1`          | `0101` | `Rd = Ms1` (read mask to integer register)  |
| `mwr Rd, Ms1`          | `0110` | `Ms1 = Rd` (write integer register to mask) |
| `mbs Rd, Ms1, imm4`    | `0111` | Set bit `imm4` in mask `Ms1`                |
| `mbc Rd, Ms1, imm4`    | `1000` | Clear bit `imm4` in mask `Ms1`              |
| `mbt Rd, Ms1, imm4`    | `1001` | Test bit `imm4` in mask `Ms1`, result in Rd |

### 6.11 System Operations (`SYSCALL` opcode)

| Instruction      | Func | Operation                                    |
|------------------|------|----------------------------------------------|
| `syscall imm9`    | `0000` | Trap to runtime with syscall number `imm9`  |
| `halt`            | `0001` | Halt execution                               |
| `nop`             | `0010` | No operation                                 |

**System Call Convention:**

| Register | Usage                      |
|----------|----------------------------|
| `ret0`   | Syscall number             |
| `arg0`-`arg7` | Arguments            |
| `ret0`-`ret1` | Return values        |

**Skyscraper ABI Syscall Numbers:**

These are architecture-independent syscall numbers defined by Skyscraper. The backend maps them to OS-specific syscalls (Linux, Windows, macOS) at the platform layer.

| Number | Name       | Arguments                                     | Description                     |
|--------|------------|-----------------------------------------------|---------------------------------|
| 0      | `exit`     | `arg0` = code                                 | Terminate process               |
| 1      | `read`     | `arg0` = fd, `arg1` = buf, `arg2` = count    | Read from file descriptor       |
| 2      | `write`    | `arg0` = fd, `arg1` = buf, `arg2` = count    | Write to file descriptor        |
| 3      | `open`     | `arg0` = path, `arg1` = flags, `arg2` = mode | Open file                       |
| 4      | `close`    | `arg0` = fd                                   | Close file descriptor           |
| 5      | `seek`     | `arg0` = fd, `arg1` = offset, `arg2` = whence| Seek in file                    |
| 6      | `stat`     | `arg0` = path, `arg1` = buf                   | Get file status                 |
| 7      | `mmap`     | `arg0` = addr, `arg1` = len, `arg2` = prot, `arg3` = flags, `arg4` = fd, `arg5` = offset | Map memory |
| 8      | `munmap`   | `arg0` = addr, `arg1` = len                   | Unmap memory                    |
| 9      | `brk`      | `arg0` = addr (0 = query)                     | Set/clear heap break            |
| 10     | `clock`    | (none)                                        | Get monotonic clock (ns)        |
| 11     | `yield`    | (none)                                        | Yield to scheduler              |
| 12     | `getpid`   | (none)                                        | Get process ID                  |
| 13     | `fork`     | (none)                                        | Fork process (0 in child, pid in parent) |
| 14     | `exec`     | `arg0` = path, `arg1` = argv, `arg2` = envp  | Execute program                 |
| 15     | `pipe`     | `arg0` = fds (2-element array)                | Create pipe                     |
| 16     | `dup`      | `arg0` = fd                                   | Duplicate file descriptor       |
| 17     | `dup2`     | `arg0` = oldfd, `arg1` = newfd                | Duplicate to specific fd        |
| 18     | `ioctl`    | `arg0` = fd, `arg1` = request, `arg2` = arg   | Device I/O control              |
| 19     | `time`     | `arg0` = buf (8 bytes, seconds since epoch)   | Get wall clock time             |
| 20     | `sleep`    | `arg0` = nanoseconds                          | Sleep for duration              |
| 21     | `mprotect` | `arg0` = addr, `arg1` = len, `arg2` = prot   | Change memory protection        |
| 22     | `getdents` | `arg0` = fd, `arg1` = buf, `arg2` = count    | Read directory entries          |
| 23     | `unlink`   | `arg0` = path                                 | Delete file                     |
| 24     | `rename`   | `arg0` = oldpath, `arg1` = newpath            | Rename file                     |
| 25     | `mkdir`    | `arg0` = path, `arg1` = mode                  | Create directory                |
| 26     | `rmdir`    | `arg0` = path                                 | Remove directory                |
| 27     | `chdir`    | `arg0` = path                                 | Change working directory        |
| 28     | `getcwd`   | `arg0` = buf, `arg1` = size                   | Get current working directory   |
| 29-31  | (reserved) |                                               | Future use                      |

**Seek Whence Values (for `seek` syscall):**

| Value | Name     | Description                   |
|-------|----------|-------------------------------|
| 0     | `SEEK_SET` | Seek from beginning of file |
| 1     | `SEEK_CUR` | Seek from current position  |
| 2     | `SEEK_END` | Seek from end of file       |

**Memory Protection Flags (for `mmap`/`mprotect`):**

| Bit | Name   | Description       |
|-----|--------|-------------------|
| 0   | `PROT_READ`  | Page can be read    |
| 1   | `PROT_WRITE` | Page can be written |
| 2   | `PROT_EXEC`  | Page can be executed|

**Note:** The backend platform layer (e.g., `isa/x86-64/linux`, `isa/aarch64/linux`) translates these abstract syscall numbers into the appropriate OS-specific syscall interface. This keeps the ISA portable across platforms.

---

## 7. Flags Register

The `flags` register is a 64-bit register with the following layout:

| Bit | Name | Description                                    |
|-----|------|------------------------------------------------|
| 0   | Z    | Zero flag (result was zero)                    |
| 1   | N    | Negative flag (result was negative, bit 63 set)|
| 2   | C    | Carry flag (unsigned overflow / borrow)        |
| 3   | V    | Overflow flag (signed overflow)                |

Bits 4-63 are reserved and must be zero.

Flags are set by:
- All ALU operations
- Comparison operations (`cmp`, `cmpi`, `tst`, `tsti`)
- Load upper immediate (`lui`) does NOT set flags

Flags are read by:
- Conditional branches (`beq`, `bne`, `blt`, `bge`, `bltu`, `bgeu`, `ble`, `bgt`)

---

## 8. Calling Convention

Skyscraper uses a register-based calling convention. The dedicated argument, return, and temporary registers make calling convention compliance explicit in the ISA.

### 8.1 Register Usage

| Register(s)        | Role              | Saved By    |
|---------------------|-------------------|-------------|
| `ret0`, `ret1`      | Return values     | Caller      |
| `arg0`-`arg7`       | Arguments         | Caller      |
| `temp0`-`temp5`     | Caller-saved temps| Caller      |
| `r0`-`r15`          | Callee-saved      | Callee      |
| `sp`                | Stack pointer     | Callee      |
| `fp`                | Frame pointer     | Callee      |
| `lr`                | Link register     | Caller      |
| `vec0`-`vec15`| Vector temps      | Caller      |
| `mask0`-`mask15`    | Mask temps        | Caller      |

### 8.2 Stack Frame Layout

```
High address
+-------------------+
| Previous frame    |
+-------------------+
| Saved lr          |  [fp + 16]
+-------------------+
| Saved fp          |  [fp + 8]
+-------------------+
| Saved r-registers |  [fp]
| Local variables   |
| Spill slots       |
| Arguments 8+      |
+-------------------+
| Outgoing args     |
+-------------------+
Low address         <- sp
```

### 8.3 Function Call Sequence

**Caller:**
1. Save any caller-saved registers needed after the call (temp0-temp5, vec registers)
2. Place arguments in `arg0`-`arg7` (args 0-7)
3. For args beyond 8, push onto stack
4. Execute `jalr target` (or `jal imm24`)
5. Retrieve return value from `ret0` / `ret1`
6. Restore caller-saved registers

**Callee:**
1. Push `fp` and `lr` onto stack
2. Set `fp = sp` (create new frame)
3. Save any callee-saved registers used (r0-r15)
4. Adjust `sp` for local variables and spill space
5. (execute function body)
6. Place return values in `ret0` / `ret1`
7. Restore callee-saved registers
8. `sp = fp` (deallocate frame)
9. Pop `fp` and `lr`
10. Execute `ret`

---

## 9. Assembler Syntax

### 9.1 General Format

```
[label:] mnemonic [operands...]
```

### 9.2 Operand Types

| Syntax         | Description                          |
|----------------|--------------------------------------|
| `r0`-`r15`     | General-purpose register             |
| `arg0`-`arg7`  | Argument register                    |
| `ret0`-`ret1`  | Return value register                |
| `temp0`-`temp5`| Temporary register                   |
| `vec0`-`vec15` | Vector register                 |
| `mask0`-`mask15`     | Mask register                 |
| `sp`           | Stack pointer                        |
| `fp`           | Frame pointer                        |
| `lr`           | Link register                        |
| `imm9`         | 9-bit signed immediate (-256..255)   |
| `imm19`        | 19-bit unsigned immediate            |
| `imm24`        | 24-bit signed immediate              |
| `[r1 + 8]`     | Memory reference: base + offset      |
| `[r1 + r2]`    | Memory reference: base + index       |
| `[vec0]`    | Vector memory reference              |
| `label`        | Symbol reference (resolved at link)  |

### 9.3 Number Literals

| Format     | Example   | Description              |
|------------|-----------|--------------------------|
| Decimal    | `42`      | Unsigned decimal         |
| Hex        | `0xFF`    | Hexadecimal              |
| Binary     | `0b1010`  | Binary                   |
| Octal      | `0o52`    | Octal                    |
| Character  | `'A'`     | ASCII character constant |
| Negative   | `-42`     | Negative decimal         |

---

## 10. Directives

| Directive           | Description                                      |
|---------------------|--------------------------------------------------|
| `.text`             | Switch to code section                           |
| `.data`             | Switch to initialized data section               |
| `.bss`              | Switch to zero-initialized data section          |
| `.org addr`         | Set current address                              |
| `.align n`          | Align to `n`-byte boundary (power of 2)          |
| `.byte val, ...`    | Emit 8-bit values                                |
| `.half val, ...`    | Emit 16-bit values                               |
| `.word val, ...`    | Emit 32-bit values                               |
| `.dword val, ...`   | Emit 64-bit values                               |
| `.string str`       | Emit null-terminated string                      |
| `.asciz str`        | Emit null-terminated string (alias)              |
| `.zero n`           | Emit `n` zero bytes                              |
| `.global symbol`    | Mark symbol as globally visible                  |
| `.local symbol`     | Mark symbol as file-local                        |
| `.extern symbol`    | Declare symbol as external (defined elsewhere)   |
| `.size symbol, expr`| Set symbol size                                  |
| `.type symbol, type`| Set symbol type (`function`, `object`)           |
| `.section name`     | Switch to named section                          |
| `.include "file"`   | Include another source file                      |
| `.macro name [args]`| Begin macro definition                           |
| `.endm`             | End macro definition                             |

---

## 11. Labels and Symbols

Labels are names for memory addresses. They are resolved during assembly and linking.

**Syntax:**
```
my_label:
    add r1, r2, r3
    bne my_label
```

**Forward references** are supported: a label can be used before it is defined.

**Symbol types:**
- **Local symbols:** Scoped to the current file (prefix with `.L` or use `.local`)
- **Global symbols:** Visible to the linker (use `.global`)
- **External symbols:** Defined in another object file (use `.extern`)

---

## 12. Binary Format

Skyscraper produces ELF-like object files for the initial target (Linux x86-64). The exact format is defined by the backend implementation.

### 12.1 Object File Layout

```
+-------------------+
| ELF Header        |
+-------------------+
| Section Headers   |
+-------------------+
| .text             |  (code)
+-------------------+
| .data             |  (initialized data)
+-------------------+
| .bss              |  (zero-initialized data)
+-------------------+
| .rodata           |  (read-only data, constants)
+-------------------+
| .symtab           |  (symbol table)
+-------------------+
| .strtab           |  (string table)
+-------------------+
| .shstrtab         |  (section name string table)
+-------------------+
```

### 12.2 Relocation Types

| Type         | Description                                    |
|--------------|------------------------------------------------|
| `R_IMM9`     | 9-bit signed immediate field (I/S-type)        |
| `R_IMM12`    | 12-bit PC-relative offset (B-type branch)      |
| `R_IMM19`    | 19-bit upper immediate (U-type)                |
| `R_IMM24`    | 24-bit PC-relative offset (J-type jump/call)   |
| `R_ABS64`    | 64-bit absolute address (for data references)  |
| `R_PC64`     | 64-bit PC-relative offset                      |

---

## 13. Example Programs

### 13.1 Hello World

```asm
.data
msg:
    .string "Hello, World!\n"
msg_len = . - msg

.text
.global _start

_start:
    ldi ret0, 2             ; syscall: write
    ldi arg0, 1             ; fd: stdout
    lui arg1, msg           ; buf (upper)
    ori arg1, arg1, msg     ; buf (lower)
    ldi arg2, msg_len       ; count
    syscall 2               ; write(fd, buf, count)

    ldi ret0, 0             ; syscall: exit
    ldi arg0, 0             ; exit code 0
    syscall 0               ; exit(0)
```

### 13.2 Function Call

```asm
.text
.global _start

_start:
    ldi arg0, 10            ; argument 0 = 10
    ldi arg1, 20            ; argument 1 = 20
    jal compute             ; call compute(10, 20)
    ; result in ret0

    ldi ret0, 0             ; exit
    ldi arg0, 0
    syscall 0

compute:
    add ret0, arg0, arg1    ; ret0 = arg0 + arg1
    ret
```

### 13.3 Loop

```asm
.text
.global _start

_start:
    ldi r0, 0               ; i = 0
    ldi r1, 10              ; limit = 10

loop:
    cmp temp0, r0, r1       ; compare i, limit
    bge done                ; if i >= limit, exit

    addi r0, r0, 1          ; i++
    j loop                  ; repeat

done:
    ldi ret0, 0
    ldi arg0, 0
    syscall 0
```

### 13.4 Vector Operations

```asm
.data
vec_a: .word 1, 2, 3, 4
vec_b: .word 5, 6, 7, 8
vec_c: .word 0, 0, 0, 0

.text
.global _start

_start:
    lui r0, vec_a           ; load address of vec_a
    ori r0, r0, vec_a
    vld.w vec0, [r0]     ; load vec_a into vec0

    lui r0, vec_b
    ori r0, r0, vec_b
    vld.w vec1, [r0]     ; load vec_b into vec1

    vadd.v vec2, vec0, vec1  ; vec2 = vec0 + vec1

    lui r0, vec_c
    ori r0, r0, vec_c
    vst.w vec2, [r0]     ; store result to vec_c

    ldi ret0, 0
    ldi arg0, 0
    syscall 0
```

---

## 14. Encoding Reference

### Complete Opcode Table

| Opcode | Binary  | Format | Description              |
|--------|---------|--------|--------------------------|
| `NOP`  | `0000`  | -      | No operation             |
| `ALU`  | `0001`  | R/I    | Arithmetic/logic ops     |
| `MEM`  | `0010`  | I/S    | Memory load/store        |
| `BR`   | `0011`  | B      | Conditional branch       |
| `JMP`  | `0100`  | J/R    | Jump/call/return         |
| `LUI`  | `0101`  | U      | Load upper immediate     |
| `SYSCALL`| `0110`| -      | System call              |
| `VEC`  | `0111`  | V      | Vector/SIMD operations   |
| `CMP`  | `1000`  | R/I    | Compare and set flags    |
| `MOV`  | `1001`  | R      | Data movement            |
| `LDI`  | `1010`  | I      | Load immediate           |
| `MASK` | `1011`  | R/I    | Mask register operations |
| `UNUSED`| `1100` | -      | Reserved                 |
| `UNUSED`| `1101` | -      | Reserved                 |
| `UNUSED`| `1110` | -      | Reserved                 |
| `UNUSED`| `1111` | -      | Reserved                 |

---

## 15. File Extensions and Naming

| Extension | Purpose                        | Example               |
|-----------|--------------------------------|-----------------------|
| `.sky`    | Skyscraper source files        | `main.sky`, `lib.sky` |
| `.conf`   | Project configuration (TOML)   | `skyscraper.conf`     |
| `.skyo`   | Skyscraper object files        | `main.skyo`           |
| `.skyb`   | Skyscraper binary (linked)     | `main.skyb`           |

**Language name:** Skyscraper (Skyscraper Assembly)
**File extension:** `.sky`

---

## 16. Lexer Design

The lexer processes `.sky` source files and produces a stream of tokens.

### 16.1 Token Types

| Token           | Pattern                                  | Example            |
|-----------------|------------------------------------------|--------------------|
| `INSTRUCTION`   | `[a-z][a-z0-9]*` (match ISA mnemonics)  | `add`, `ldi`, `jal`|
| `REGISTER`      | `r[0-9]+`, `arg[0-7]`, `ret[01]`, `temp[0-5]`, `vec[0-9]+`, `mask[0-9]+`, `sp`, `fp`, `lr` | `r0`, `arg3`, `vec12` |
| `DIRECTIVE`     | `\.[a-zA-Z]+`                            | `.text`, `.string`  |
| `LABEL_DEF`     | `[a-zA-Z_][a-zA-Z0-9_]*:`               | `loop:`, `_start:`  |
| `LABEL_REF`     | `[a-zA-Z_][a-zA-Z0-9_]*` (not instruction/directive/register) | `loop`, `_start` |
| `NUMBER`        | See 16.2                                 | `42`, `0xFF`        |
| `CHAR`          | `'.'` or `'\\.'`                         | `'A'`, `'\n'`      |
| `STRING`        | `"..."` (with escape sequences)          | `"Hello\n"`         |
| `LPAREN`        | `\(`                                     | `(`                 |
| `RPAREN`        | `\)`                                     | `)`                 |
| `LBRACKET`      | `\[`                                     | `[`                 |
| `RBRACKET`      | `\]`                                     | `]`                 |
| `COMMA`         | `,`                                      | `,`                 |
| `PLUS`          | `\+`                                     | `+`                 |
| `MINUS`         | `-`                                      | `-`                 |
| `STAR`          | `\*`                                     | `*`                 |
| `SLASH`         | `/`                                      | `/`                 |
| `PERCENT`       | `%`                                      | `%`                 |
| `AMPERSAND`     | `&`                                      | `&`                 |
| `PIPE`          | `\|`                                     | `\|`                |
| `CARET`         | `\^`                                     | `^`                 |
| `TILDE`         | `~`                                      | `~`                |
| `BANG`          | `!`                                      | `!`                 |
| `COLON`         | `:` (standalone)                         | `:`                 |
| `ASSIGN`        | `=`                                      | `=`                 |
| `DOLLAR`        | `$`                                      | `$`                 |
| `COMMENT`       | `;[^\n]*`                                | `; this is a comment` |
| `NEWLINE`       | `\n`                                     |                    |
| `EOF`           | end of file                              |                    |
| `UNKNOWN`       | any unmatched character                  |                    |

### 16.2 Number Literals

| Format     | Regex              | Description                    | Range                   |
|------------|--------------------|--------------------------------|-------------------------|
| Decimal    | `[0-9]+`           | Unsigned decimal               | 0 .. 2^64-1            |
| Hex        | `0x[0-9a-fA-F]+`  | Hexadecimal                    | 0 .. 2^64-1            |
| Binary     | `0b[01]+`          | Binary                         | 0 .. 2^64-1            |
| Octal      | `0o[0-7]+`         | Octal                          | 0 .. 2^64-1            |
| Negative   | `-[0-9]+`          | Negative decimal               | -2^63 .. -1            |
| Char       | `'.'`              | ASCII character (zero-extended) | 0 .. 127               |
| Escaped    | `'\\.'`            | Escape sequence                | See 16.3                |

### 16.3 Escape Sequences

| Sequence | Meaning           | Byte Value |
|----------|-------------------|------------|
| `\n`     | Newline           | 0x0A       |
| `\t`     | Tab               | 0x09       |
| `\r`     | Carriage return   | 0x0D       |
| `\0`     | Null              | 0x00       |
| `\\`     | Backslash         | 0x5C       |
| `\'`     | Single quote      | 0x27       |
| `\"`     | Double quote      | 0x22       |
| `\xHH`   | Hex byte          | 0xHH       |

### 16.4 Tokenization Rules

1. Skip whitespace (spaces, tabs) — do NOT skip newlines
2. Skip comments (`;` to end of line)
3. Match longest valid token first
4. Labels (`name:`) are tokenized as `LABEL_DEF`
5. Identifiers that match instruction mnemonics are `INSTRUCTION`
6. Identifiers that match register names are `REGISTER`
7. Identifiers that start with `.` are `DIRECTIVE`
8. All other identifiers are `LABEL_REF`

### 16.5 Lexer Example

Input:
```asm
_start:
    ldi arg0, 42        ; load 42 into arg0
    jal compute         ; call compute
```

Token stream:
```
LABEL_DEF("_start")
NEWLINE
INSTRUCTION("ldi")
REGISTER("arg0")
COMMA
NUMBER(42)
COMMENT("; load 42 into arg0")
NEWLINE
INSTRUCTION("jal")
LABEL_REF("compute")
COMMENT("; call compute")
NEWLINE
EOF
```

---

## 17. Parser Design

The parser consumes the token stream and produces an Abstract Syntax Tree (AST).

### 17.1 Grammar (BNF)

```bnf
<program>       ::= <statement>*

<statement>     ::= <label_def>
                   | <instruction>
                   | <directive>
                   | <macro_def>
                   | <constant_def>
                   | <empty_line>

<label_def>     ::= LABEL_DEF

<instruction>   ::= INSTRUCTION <operand_list>

<operand_list>  ::= <operand> (COMMA <operand>)*
                   | <empty>

<operand>       ::= <register>
                   | <immediate>
                   | <memory_ref>
                   | <label_ref>
                   | <vec_register>
                   | <mask_register>

<register>      ::= REGISTER

<immediate>     ::= NUMBER
                   | MINUS NUMBER
                   | CHAR
                   | <expr>

<memory_ref>    ::= LBRACKET REGISTER PLUS <immediate> RBRACKET
                   | LBRACKET REGISTER PLUS REGISTER RBRACKET
                   | LBRACKET LABEL_REF RBRACKET

<vec_register>  ::= REGISTER  ; (vec0-vec15)

<mask_register> ::= REGISTER  ; (mask0-mask15)

<label_ref>     ::= LABEL_REF

<directive>     ::= DIRECTIVE <directive_args>

<directive_args> ::= <expr_list>
                    | <string_list>
                    | <empty>

<expr_list>     ::= <expr> (COMMA <expr>)*

<expr>          ::= NUMBER
                   | NUMBER PLUS NUMBER
                   | NUMBER MINUS NUMBER
                   | LABEL_REF
                   | LABEL_REF MINUS LABEL_REF  ; subtraction
                   | DOLLAR  ; current address

<string_list>   ::= STRING (COMMA STRING)*

<macro_def>     ::= DIRECTIVE("macro") LABEL_REF <macro_args>
                     <statement>*
                   DIRECTIVE("endm")

<macro_args>    ::= REGISTER (COMMA REGISTER)*
                   | <empty>

<constant_def>  ::= LABEL_REF ASSIGN <expr>
```

### 17.2 AST Node Types

```rust
enum AstNode {
    Program(Vec<Statement>),
    Statement(Statement),
}

enum Statement {
    LabelDef(String),
    Instruction {
        mnemonic: String,
        operands: Vec<Operand>,
    },
    Directive {
        name: String,
        args: Vec<Expr>,
    },
    ConstantDef {
        name: String,
        value: Expr,
    },
    MacroDef {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
    },
}

enum Operand {
    Register(String),
    VecRegister(String),
    MaskRegister(String),
    Immediate(Expr),
    MemoryRef {
        base: String,
        offset: Option<Expr>,
    },
    LabelRef(String),
}

enum Expr {
    Number(i64),
    LabelRef(String),
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    CurrentAddr,  // $
}

enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}
```

### 17.3 Parser Phases

The parser operates in two passes:

**Pass 1: Parse and collect**
- Tokenize source
- Parse into AST
- Collect all label definitions into symbol table
- Record file positions of forward references

**Pass 2: Resolve and emit**
- Resolve forward references using symbol table
- Encode instructions into 32-bit words
- Emit object file sections (.text, .data, .bss)
- Generate relocation entries for unresolved symbols

### 17.4 Error Handling

Errors include:
- **Lexer errors:** Unrecognized character, unterminated string, invalid number
- **Parse errors:** Unexpected token, missing operand, invalid syntax
- **Assembly errors:** Unknown instruction, invalid register, immediate out of range, undefined label, duplicate label, alignment error

All errors report:
- File name
- Line number
- Column number
- Error message
- Source context (optional)

---

## 18. Project Structure

```
myproject/
├── skyscraper.conf        # Project configuration
├── src/
│   ├── main.sky           # Entry point (contains _start)
│   └── lib.sky            # Library code
├── target/
│   ├── debug/             # Debug build output
│   │   ├── main.skyo      # Object files
│   │   ├── lib.skyo
│   │   └── main.skyb      # Linked binary
│   └── release/           # Release build output
│       ├── main.skyo
│       ├── lib.skyo
│       └── main.skyb
└── pkg/                   # Dependencies (future)
    └── ...
```

**Build output convention:**
- Object files: `<name>.skyo`
- Linked binaries: `<name>.skyb`
- Debug builds: `target/debug/`
- Release builds: `target/release/`

---

## 19. Configuration File

`skyscraper.conf` uses TOML syntax.

```toml
# Project metadata
[package]
name = "myproject"
version = "0.1.0"
authors = ["Author Name <email@example.com>"]
description = "A short description"
license = "Apache-2.0"

# Build configuration
[build]
target = "x86-64/linux"       # Target platform
entry = "src/main.sky"        # Entry point file
opt-level = 0                 # 0=debug, 1=optimized, 2=aggressive
debug = true                  # Include debug info

# Source directories
[paths]
src = "src"                   # Source directory
output = "target"             # Build output directory

# Dependencies (future)
[dependencies]
# pkg-name = "version"
# pkg-name = { version = "1.0", features = ["std"] }
```

### 19.1 Target Triples

| Triple              | Description                    |
|---------------------|--------------------------------|
| `x86-64/linux`      | Linux x86-64 (first target)    |
| `aarch64/linux`     | Linux ARM64                    |
| `x86-64/windows`    | Windows x86-64                 |
| `aarch64/windows`   | Windows ARM64                  |
| `x86-64/macos`      | macOS x86-64                   |
| `aarch64/macos`     | macOS ARM64 (Apple Silicon)    |

---

## 20. Build System

### 20.1 Build Commands

| Command                    | Description                          |
|----------------------------|--------------------------------------|
| `skyscraper build`         | Build the project (debug)            |
| `skyscraper build --release` | Build the project (optimized)      |
| `skyscraper run`           | Build and run                        |
| `skyscraper clean`         | Remove target/ directory             |
| `skyscraper init`          | Create new project scaffold          |
| `skyscraper check`         | Check syntax without emitting binary |
| `skyscraper fmt`           | Format source files                  |
| `skyscraper test`          | Run tests                            |

### 20.2 Build Pipeline

```
.sky files
    │
    ▼
┌─────────┐
│  Lexer  │  Tokenize source
└────┬────┘
     │
     ▼
┌─────────┐
│  Parser │  Build AST
└────┬────┘
     │
     ▼
┌──────────┐
│ Assembler│  Encode instructions, emit .skyo
└────┬─────┘
     │
     ▼
┌─────────┐
│  Linker │  Resolve symbols, produce .skyb
└────┬────┘
     │
     ▼
┌──────────────┐
│ Platform Fixup│  Map syscalls, set entry point
└────┬─────────┘
     │
     ▼
  Native binary (ELF/Mach-O/PE)
```

---

## 21. Package System (Future)

Planned for Phase 5. Inspired by Cargo and npm.

### 21.1 Package Registry

```
skyscraper package search <query>
skyscraper package install <package>
skyscraper package publish
```

### 21.2 Dependency Format in `skyscraper.conf`

```toml
[dependencies]
math = "1.0"                           # From registry
io = { version = "2.1", features = ["file", "net"] }
utils = { git = "https://github.com/user/utils" }
local = { path = "../local-pkg" }      # Local dependency
```

### 21.3 Package Layout

```
pkg-name/
├── skyscraper.conf
├── src/
│   ├── lib.sky
│   └── ...
└── tests/
    └── test.sky
```
