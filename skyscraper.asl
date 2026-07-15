// Skyscraper ISA - Architecture Specification Language
// Version: 1.1
//
// This file formally describes the Skyscraper ISA for tools,
// verification, and code generation.

// =============================================================================
// Types
// =============================================================================

type bitstring(N) = bits(N);
type uint8  = bits(8);
type uint16 = bits(16);
type uint32 = bits(32);
type uint64 = bits(64);
type sint8  = bits(8);
type sint16 = bits(16);
type sint32 = bits(32);
type sint64 = bits(64);
type byte   = bits(8);
type half   = bits(16);
type word   = bits(32);
type dword  = bits(64);
type ptr    = bits(64);

type Vector128 = bits(128);

// =============================================================================
// Register File Declarations
// =============================================================================

// Integer registers - 32 x 64-bit
// Encoding: 5-bit register index (0-31)
-regclass IntegerReg is
    r0, r1, r2, r3, r4, r5, r6, r7,
    r8, r9, r10, r11, r12, r13, r14, r15,   // GPR (indices 0-15)
    arg0, arg1, arg2, arg3,                  // Argument (indices 16-19)
    arg4, arg5, arg6, arg7,                  // Argument (indices 20-23)
    ret0, ret1,                              // Return (indices 24-25)
    temp0, temp1, temp2,                     // Temporary (indices 26-28)
    temp3, temp4, temp5;                     // Temporary (indices 29-31)

-regfield R[IntegerReg] is RegValue(64);

// Special-purpose registers (not in integer file)
-regclass SpecialReg is
    sp,           // Stack pointer
    fp,           // Frame pointer
    pc,           // Program counter
    lr,           // Link register
    flags,        // Condition flags (N, Z, C, V)
    zero,         // Hardwired to 0
    one;          // Hardwired to 1

-regfield SP   is RegValue(64);
-regfield FP   is RegValue(64);
-regfield PC   is RegValue(64);
-regfield LR   is RegValue(64);
-regfield FLAGS is bits(64);

// Vector registers - 16 x 128-bit
// Encoding: 4-bit register index (0-15)
-regclass VecReg is
    vec0, vec1, vec2, vec3, vec4, vec5, vec6, vec7,
    vec8, vec9, vec10, vec11, vec12, vec13, vec14, vec15;

-regfield V[VecReg] is Vector128;

// Mask registers - 16 x 64-bit
// Encoding: 4-bit register index (0-15)
-regclass MaskReg is
    mask0, mask1, mask2, mask3, mask4, mask5, mask6, mask7,
    mask8, mask9, mask10, mask11, mask12, mask13, mask14, mask15;

-regfield M[MaskReg] is bits(64);

// =============================================================================
// Flags Register Layout
// =============================================================================

struct Flags {
    Z : bits(1);   // Zero flag
    N : bits(1);   // Negative flag
    C : bits(1);   // Carry flag
    V : bits(1);   // Overflow flag
    RES : bits(60); // Reserved (must be 0)
};

// =============================================================================
// Instruction Encoding Formats
// =============================================================================

// All instructions are 32 bits wide

struct R_type {
    opcode : bits(4);    // [31:28]
    func   : bits(4);    // [27:24]
    Rd     : bits(5);    // [23:19] - destination register
    Rs1    : bits(5);    // [18:14] - source register 1
    Rs2    : bits(5);    // [13:9]  - source register 2
    res0   : bits(1);    // [8]     - reserved
    res1   : bits(7);    // [7:0]   - reserved
};

struct I_type {
    opcode : bits(4);    // [31:28]
    func   : bits(4);    // [27:24]
    Rd     : bits(5);    // [23:19] - destination register
    Rs1    : bits(5);    // [18:14] - source register 1
    imm9   : sint9;      // [13:5]  - signed 9-bit immediate
    res    : bits(4);    // [4:0]   - reserved
};

struct S_type {
    opcode : bits(4);    // [31:28]
    func   : bits(4);    // [27:24]
    Rs2    : bits(5);    // [23:19] - source register 2 (data)
    Rs1    : bits(5);    // [18:14] - source register 1 (base)
    imm9   : sint9;      // [13:5]  - signed 9-bit immediate
    res    : bits(4);    // [4:0]   - reserved
};

struct B_type {
    opcode : bits(4);    // [31:28]
    cond   : bits(4);    // [27:24] - condition code
    imm12  : sint12;     // [23:12] - signed 12-bit PC-relative offset
    Rs1    : bits(5);    // [11:7]  - source register
    res0   : bits(5);    // [6:2]   - reserved
    res1   : bits(2);    // [1:0]   - reserved
};

struct U_type {
    opcode : bits(4);    // [31:28]
    func   : bits(4);    // [27:24]
    Rd     : bits(5);    // [23:19] - destination register
    imm19  : bits(19);   // [18:0]  - unsigned 19-bit immediate
};

struct J_type {
    opcode : bits(4);    // [31:28]
    func   : bits(4);    // [27:24]
    imm24  : sint24;     // [23:0]  - signed 24-bit PC-relative offset
};

struct V_type {
    opcode : bits(4);    // [31:28]
    func   : bits(4);    // [27:24]
    Vd     : bits(4);    // [23:19] - vector destination
    Vs1    : bits(4);    // [18:15] - vector source 1
    Vs2    : bits(4);    // [14:11] - vector source 2
    Ms     : bits(4);    // [10:7]  - mask register (0 = no mask)
    res    : bits(7);    // [6:0]   - reserved
};

// =============================================================================
// Opcode Definitions
// =============================================================================

enum Opcode : bits(4) {
    NOP     = 0b0000,
    ALU     = 0b0001,
    MEM     = 0b0010,
    BR      = 0b0011,
    JMP     = 0b0100,
    LUI     = 0b0101,
    SYSCALL = 0b0110,
    VEC     = 0b0111,
    CMP     = 0b1000,
    MOV     = 0b1001,
    LDI     = 0b1010,
    MASK    = 0b1011
};

// =============================================================================
// ALU Function Codes (opcode = ALU)
// =============================================================================

enum ALU_func : bits(4) {
    ADD = 0b0000,
    SUB = 0b0001,
    MUL = 0b0010,
    DIV = 0b0011,
    MOD = 0b0100,
    AND = 0b0101,
    OR  = 0b0110,
    XOR = 0b0111,
    SHL = 0b1000,
    SHR = 0b1001,
    SAR = 0b1010,
    NEG = 0b1011,
    NOT = 0b1100
};

// =============================================================================
// Memory Function Codes (opcode = MEM)
// =============================================================================

enum MEM_func : bits(4) {
    LB  = 0b0000,   // Load byte (zero-extend)
    LH  = 0b0001,   // Load half (zero-extend)
    LW  = 0b0010,   // Load word (zero-extend)
    LD  = 0b0011,   // Load dword
    LBS = 0b0100,   // Load byte (sign-extend)
    LHS = 0b0101,   // Load half (sign-extend)
    LWS = 0b0110,   // Load word (sign-extend)
    LFS = 0b0111,   // Load single float
    LFD = 0b1000,   // Load double float
    SB  = 0b1001,   // Store byte
    SH  = 0b1010,   // Store half
    SW  = 0b1011,   // Store word
    SD  = 0b1100,   // Store dword
    SFS = 0b1101,   // Store single float
    SFD = 0b1110    // Store double float
};

// =============================================================================
// Branch Condition Codes (opcode = BR)
// =============================================================================

enum Branch_cond : bits(4) {
    EQ  = 0b0000,   // flags.Z == 1
    NE  = 0b0001,   // flags.Z == 0
    LT  = 0b0010,   // flags.N != flags.V
    GE  = 0b0011,   // flags.N == flags.V
    LTU = 0b0100,   // flags.C == 0
    GEU = 0b0101,   // flags.C == 1
    LE  = 0b0110,   // flags.Z == 1 || flags.N != flags.V
    GT  = 0b0111,   // flags.Z == 0 && flags.N == flags.V
    BZS = 0b1000,   // Rs1 == 0
    BNZ = 0b1001    // Rs1 != 0
};

// =============================================================================
// Jump Function Codes (opcode = JMP)
// =============================================================================

enum JMP_func : bits(4) {
    J    = 0b0000,  // Unconditional jump
    JAL  = 0b0001,  // Jump and link (call)
    JR   = 0b0010,  // Jump register
    JALR = 0b0011,  // Jump and link register
    RET  = 0b0100   // Return
};

// =============================================================================
// Comparison Function Codes (opcode = CMP)
// =============================================================================

enum CMP_func : bits(4) {
    CMP = 0b0000,   // Compare (subtract)
    TST = 0b0001    // Test (AND)
};

// =============================================================================
// MOV Function Codes (opcode = MOV)
// =============================================================================

enum MOV_func : bits(4) {
    MV  = 0b0000,   // Register move
    SEB = 0b0001,   // Sign-extend byte
    SEH = 0b0010,   // Sign-extend half
    SEW = 0b0011,   // Sign-extend word
    ZLB = 0b0100,   // Zero-extend byte
    ZLH = 0b0101,   // Zero-extend half
    ZLW = 0b0110,   // Zero-extend word
    MFPC= 0b0111,   // Read PC
    MTPC= 0b1000,   // Write PC (privileged)
    MFLR= 0b1001,   // Read LR
    MTLR= 0b1010,   // Write LR
    MFSP= 0b1011,   // Read SP
    MTSP= 0b1100,   // Write SP
    MFFP= 0b1101,   // Read FP
    MTFP= 0b1110    // Write FP
};

// =============================================================================
// Vector Function Codes (opcode = VEC)
// =============================================================================

enum VEC_func : bits(4) {
    VADD  = 0b0000,  // Vector add
    VSUB  = 0b0001,  // Vector subtract
    VMUL  = 0b0010,  // Vector multiply
    VDIV  = 0b0011,  // Vector divide
    VAND  = 0b0100,  // Vector AND
    VOR   = 0b0101,  // Vector OR
    VXOR  = 0b0110,  // Vector XOR
    VNOT  = 0b0111,  // Vector NOT
    VLD_B = 0b1000,  // Load vector (byte)
    VST_B = 0b1001,  // Store vector (byte)
    VLD_W = 0b1010,  // Load vector (word)
    VST_W = 0b1011,  // Store vector (word)
    VDUP_B= 0b1100,  // Broadcast byte
    VDUP_H= 0b1101,  // Broadcast half
    VDUP_W= 0b1110,  // Broadcast word
    VDUP_D= 0b1111   // Broadcast dword
};

// =============================================================================
// Mask Function Codes (opcode = MASK)
// =============================================================================

enum MASK_func : bits(4) {
    MSET = 0b0000,   // Set mask from immediate
    MAND = 0b0001,   // Mask AND
    MOR  = 0b0010,   // Mask OR
    MXOR = 0b0011,   // Mask XOR
    MNOT = 0b0100,   // Mask NOT
    MRD  = 0b0101,   // Read mask to integer
    MWR  = 0b0110,   // Write integer to mask
    MBS  = 0b0111,   // Set bit in mask
    MBC  = 0b1000,   // Clear bit in mask
    MBT  = 0b1001    // Test bit in mask
};

// =============================================================================
// Instruction Decoder
// =============================================================================

instruction Decode(bits(32) encoding) is
    let opcode = encoding[31:28];

    case opcode of
        Opcode.NOP     => { NOP }
        Opcode.ALU     => decode_ALU(encoding)
        Opcode.MEM     => decode_MEM(encoding)
        Opcode.BR      => decode_BR(encoding)
        Opcode.JMP     => decode_JMP(encoding)
        Opcode.LUI     => decode_LUI(encoding)
        Opcode.SYSCALL => decode_SYSCALL(encoding)
        Opcode.VEC     => decode_VEC(encoding)
        Opcode.CMP     => decode_CMP(encoding)
        Opcode.MOV     => decode_MOV(encoding)
        Opcode.LDI     => decode_LDI(encoding)
        Opcode.MASK    => decode_MASK(encoding)
        _              => { UNDEFINED }

// =============================================================================
// ALU Instruction Semantics
// =============================================================================

instruction decode_ALU(bits(32) enc) is
    let r = R_type{enc};
    let func = ALU_func{r.func};
    let rd  = r.Rd;
    let rs1 = r.Rs1;
    let rs2 = r.Rs2;
    let val1 = R[rs1];
    let val2 = R[rs2];
    let result : bits(64);

    case func of
        ALU_func.ADD => { result = val1 + val2; SetFlags(result); R[rd] = result; }
        ALU_func.SUB => { result = val1 - val2; SetFlags(result); R[rd] = result; }
        ALU_func.MUL => { result = val1 * val2; SetFlags(result); R[rd] = result; }
        ALU_func.DIV => { result = val1 / val2; SetFlags(result); R[rd] = result; }
        ALU_func.MOD => { result = val1 % val2; SetFlags(result); R[rd] = result; }
        ALU_func.AND => { result = val1 AND val2; SetFlags(result); R[rd] = result; }
        ALU_func.OR  => { result = val1 OR val2; SetFlags(result); R[rd] = result; }
        ALU_func.XOR => { result = val1 XOR val2; SetFlags(result); R[rd] = result; }
        ALU_func.SHL => { result = val1 << val2[5:0]; SetFlags(result); R[rd] = result; }
        ALU_func.SHR => { result = val1 >> val2[5:0]; SetFlags(result); R[rd] = result; }
        ALU_func.SAR => { result = SInt(val1) >> val2[5:0]; SetFlags(result); R[rd] = result; }
        ALU_func.NEG => { result = 0 - val1; SetFlags(result); R[rd] = result; }
        ALU_func.NOT => { result = NOT val1; SetFlags(result); R[rd] = result; }

// ALU with immediate (I-type)
instruction decode_ALU_imm(bits(32) enc) is
    let r = I_type{enc};
    let func = ALU_func{r.func};
    let rd  = r.Rd;
    let rs1 = r.Rs1;
    let imm = SignExtend(r.imm9, 64);
    let val1 = R[rs1];
    let result : bits(64);

    case func of
        ALU_func.ADD => { result = val1 + imm; SetFlags(result); R[rd] = result; }
        ALU_func.SUB => { result = val1 - imm; SetFlags(result); R[rd] = result; }
        ALU_func.MUL => { result = val1 * imm; SetFlags(result); R[rd] = result; }
        ALU_func.DIV => { result = val1 / imm; SetFlags(result); R[rd] = result; }
        ALU_func.MOD => { result = val1 % imm; SetFlags(result); R[rd] = result; }
        ALU_func.AND => { result = val1 AND ZeroExtend(r.imm9, 64); SetFlags(result); R[rd] = result; }
        ALU_func.OR  => { result = val1 OR ZeroExtend(r.imm9, 64); SetFlags(result); R[rd] = result; }
        ALU_func.XOR => { result = val1 XOR ZeroExtend(r.imm9, 64); SetFlags(result); R[rd] = result; }
        ALU_func.SHL => { result = val1 << r.imm9[4:0]; SetFlags(result); R[rd] = result; }
        ALU_func.SHR => { result = val1 >> r.imm9[4:0]; SetFlags(result); R[rd] = result; }
        ALU_func.SAR => { result = SInt(val1) >> r.imm9[4:0]; SetFlags(result); R[rd] = result; }

// =============================================================================
// Memory Instruction Semantics
// =============================================================================

instruction decode_MEM(bits(32) enc) is
    let func = enc[27:24];

    case func of
        MEM_func.LB => {
            let r = I_type{enc};
            let addr = R[r.Rs1] + SignExtend(r.imm9, 64);
            R[r.Rd] = ZeroExtend(Mem8[addr], 64);
        }
        MEM_func.LH => {
            let r = I_type{enc};
            let addr = R[r.Rs1] + SignExtend(r.imm9, 64);
            R[r.Rd] = ZeroExtend(Mem16[addr], 64);
        }
        MEM_func.LW => {
            let r = I_type{enc};
            let addr = R[r.Rs1] + SignExtend(r.imm9, 64);
            R[r.Rd] = ZeroExtend(Mem32[addr], 64);
        }
        MEM_func.LD => {
            let r = I_type{enc};
            let addr = R[r.Rs1] + SignExtend(r.imm9, 64);
            R[r.Rd] = Mem64[addr];
        }
        MEM_func.LBS => {
            let r = I_type{enc};
            let addr = R[r.Rs1] + SignExtend(r.imm9, 64);
            R[r.Rd] = SignExtend(Mem8[addr], 64);
        }
        MEM_func.LHS => {
            let r = I_type{enc};
            let addr = R[r.Rs1] + SignExtend(r.imm9, 64);
            R[r.Rd] = SignExtend(Mem16[addr], 64);
        }
        MEM_func.LWS => {
            let r = I_type{enc};
            let addr = R[r.Rs1] + SignExtend(r.imm9, 64);
            R[r.Rd] = SignExtend(Mem32[addr], 64);
        }
        MEM_func.LFS => {
            let r = I_type{enc};
            let addr = R[r.Rs1] + SignExtend(r.imm9, 64);
            R[r.Rd] = Mem32[addr];  // IEEE 754 single
        }
        MEM_func.LFD => {
            let r = I_type{enc};
            let addr = R[r.Rs1] + SignExtend(r.imm9, 64);
            R[r.Rd] = Mem64[addr];  // IEEE 754 double
        }
        MEM_func.SB => {
            let r = S_type{enc};
            let addr = R[r.Rs1] + SignExtend(r.imm9, 64);
            Mem8[addr] = r.Rs2[7:0];
        }
        MEM_func.SH => {
            let r = S_type{enc};
            let addr = R[r.Rs1] + SignExtend(r.imm9, 64);
            Mem16[addr] = r.Rs2[15:0];
        }
        MEM_func.SW => {
            let r = S_type{enc};
            let addr = R[r.Rs1] + SignExtend(r.imm9, 64);
            Mem32[addr] = r.Rs2[31:0];
        }
        MEM_func.SD => {
            let r = S_type{enc};
            let addr = R[r.Rs1] + SignExtend(r.imm9, 64);
            Mem64[addr] = r.Rs2;
        }
        MEM_func.SFS => {
            let r = S_type{enc};
            let addr = R[r.Rs1] + SignExtend(r.imm9, 64);
            Mem32[addr] = r.Rs2[31:0];  // IEEE 754 single
        }
        MEM_func.SFD => {
            let r = S_type{enc};
            let addr = R[r.Rs1] + SignExtend(r.imm9, 64);
            Mem64[addr] = r.Rs2;  // IEEE 754 double
        }

// =============================================================================
// Branch Instruction Semantics
// =============================================================================

instruction decode_BR(bits(32) enc) is
    let r = B_type{enc};
    let cond = Branch_cond{r.cond};
    let rs1_val = R[r.Rs1];
    let offset = SignExtend(r.imm12, 64) << 2;
    let taken = FALSE;

    case cond of
        Branch_cond.EQ  => { taken = (flags.Z == 1); }
        Branch_cond.NE  => { taken = (flags.Z == 0); }
        Branch_cond.LT  => { taken = (flags.N != flags.V); }
        Branch_cond.GE  => { taken = (flags.N == flags.V); }
        Branch_cond.LTU => { taken = (flags.C == 0); }
        Branch_cond.GEU => { taken = (flags.C == 1); }
        Branch_cond.LE  => { taken = (flags.Z == 1) || (flags.N != flags.V); }
        Branch_cond.GT  => { taken = (flags.Z == 0) && (flags.N == flags.V); }
        Branch_cond.BZS => { taken = (rs1_val == 0); }
        Branch_cond.BNZ => { taken = (rs1_val != 0); }

    if taken then
        PC = PC + offset;

// =============================================================================
// Jump Instruction Semantics
// =============================================================================

instruction decode_JMP(bits(32) enc) is
    let r = J_type{enc};
    let func = JMP_func{r.func};
    let offset = SignExtend(r.imm24, 64) << 2;

    case func of
        JMP_func.J    => { PC = PC + offset; }
        JMP_func.JAL  => { LR = PC + 4; PC = PC + offset; }
        JMP_func.JR   => { PC = R[r.func]; }  // Rs1 encoded in func field
        JMP_func.JALR => { LR = PC + 4; PC = R[r.func]; }
        JMP_func.RET  => { PC = LR; }

// =============================================================================
// Load Upper Immediate Semantics
// =============================================================================

instruction decode_LUI(bits(32) enc) is
    let r = U_type{enc};
    R[r.Rd] = ZeroExtend(r.imm19, 64) << 12;

// =============================================================================
// System Call Semantics
// =============================================================================

instruction decode_SYSCALL(bits(32) enc) is
    let imm9 = enc[13:5];
    let syscall_num = R[ret0];

    // Skyscraper ABI syscall numbers
    // Platform-specific backends map these to OS syscalls
    case syscall_num of
        0 => { // exit - terminate process
            Syscall_exit(R[arg0]);
        }
        1 => { // read - read from fd
            R[ret0] = Syscall_read(R[arg0], R[arg1], R[arg2]);
        }
        2 => { // write - write to fd
            R[ret0] = Syscall_write(R[arg0], R[arg1], R[arg2]);
        }
        3 => { // open - open file
            R[ret0] = Syscall_open(R[arg0], R[arg1], R[arg2]);
        }
        4 => { // close - close fd
            R[ret0] = Syscall_close(R[arg0]);
        }
        5 => { // seek - seek in file
            R[ret0] = Syscall_seek(R[arg0], R[arg1], R[arg2]);
        }
        6 => { // stat - get file status
            R[ret0] = Syscall_stat(R[arg0], R[arg1]);
        }
        7 => { // mmap - map memory
            R[ret0] = Syscall_mmap(R[arg0], R[arg1], R[arg2], R[arg3], R[arg4], R[arg5]);
        }
        8 => { // munmap - unmap memory
            R[ret0] = Syscall_munmap(R[arg0], R[arg1]);
        }
        9 => { // brk - set/clear heap break
            R[ret0] = Syscall_brk(R[arg0]);
        }
        10 => { // clock - monotonic clock (ns)
            R[ret0] = Syscall_clock();
        }
        11 => { // yield - yield to scheduler
            Syscall_yield();
        }
        12 => { // getpid - get process ID
            R[ret0] = Syscall_getpid();
        }
        13 => { // fork - fork process
            R[ret0] = Syscall_fork();
        }
        14 => { // exec - execute program
            R[ret0] = Syscall_exec(R[arg0], R[arg1], R[arg2]);
        }
        15 => { // pipe - create pipe
            R[ret0] = Syscall_pipe(R[arg0]);
        }
        16 => { // dup - duplicate fd
            R[ret0] = Syscall_dup(R[arg0]);
        }
        17 => { // dup2 - duplicate to specific fd
            R[ret0] = Syscall_dup2(R[arg0], R[arg1]);
        }
        18 => { // ioctl - device I/O control
            R[ret0] = Syscall_ioctl(R[arg0], R[arg1], R[arg2]);
        }
        19 => { // time - wall clock time
            R[ret0] = Syscall_time(R[arg0]);
        }
        20 => { // sleep - sleep for duration
            Syscall_sleep(R[arg0]);
        }
        21 => { // mprotect - change memory protection
            R[ret0] = Syscall_mprotect(R[arg0], R[arg1], R[arg2]);
        }
        22 => { // getdents - read directory entries
            R[ret0] = Syscall_getdents(R[arg0], R[arg1], R[arg2]);
        }
        23 => { // unlink - delete file
            R[ret0] = Syscall_unlink(R[arg0]);
        }
        24 => { // rename - rename file
            R[ret0] = Syscall_rename(R[arg0], R[arg1]);
        }
        25 => { // mkdir - create directory
            R[ret0] = Syscall_mkdir(R[arg0], R[arg1]);
        }
        26 => { // rmdir - remove directory
            R[ret0] = Syscall_rmdir(R[arg0]);
        }
        27 => { // chdir - change working directory
            R[ret0] = Syscall_chdir(R[arg0]);
        }
        28 => { // getcwd - get current working directory
            R[ret0] = Syscall_getcwd(R[arg0], R[arg1]);
        }
        _ => { // reserved - undefined
            UNDEFINED;
        }

// =============================================================================
// Comparison Instruction Semantics
// =============================================================================

instruction decode_CMP(bits(32) enc) is
    let func = CMP_func{enc[27:24]};

    case func of
        CMP_func.CMP => {
            let r = R_type{enc};
            let result = R[r.Rs1] - R[r.Rs2];
            SetFlags(result);
            R[r.Rd] = result;
        }
        CMP_func.TST => {
            let r = R_type{enc};
            let result = R[r.Rs1] AND R[r.Rs2];
            SetFlags(result);
            R[r.Rd] = result;
        }

// CMP with immediate (I-type)
instruction decode_CMP_imm(bits(32) enc) is
    let func = CMP_func{enc[27:24]};

    case func of
        CMP_func.CMP => {
            let r = I_type{enc};
            let result = R[r.Rs1] - SignExtend(r.imm9, 64);
            SetFlags(result);
            R[r.Rd] = result;
        }
        CMP_func.TST => {
            let r = I_type{enc};
            let result = R[r.Rs1] AND ZeroExtend(r.imm9, 64);
            SetFlags(result);
            R[r.Rd] = result;
        }

// =============================================================================
// MOV Instruction Semantics
// =============================================================================

instruction decode_MOV(bits(32) enc) is
    let func = MOV_func{enc[27:24]};
    let r = R_type{enc};

    case func of
        MOV_func.MV   => { R[r.Rd] = R[r.Rs1]; }
        MOV_func.SEB  => { R[r.Rd] = SignExtend(R[r.Rs1][7:0], 64); }
        MOV_func.SEH  => { R[r.Rd] = SignExtend(R[r.Rs1][15:0], 64); }
        MOV_func.SEW  => { R[r.Rd] = SignExtend(R[r.Rs1][31:0], 64); }
        MOV_func.ZLB  => { R[r.Rd] = ZeroExtend(R[r.Rs1][7:0], 64); }
        MOV_func.ZLH  => { R[r.Rd] = ZeroExtend(R[r.Rs1][15:0], 64); }
        MOV_func.ZLW  => { R[r.Rd] = ZeroExtend(R[r.Rs1][31:0], 64); }
        MOV_func.MFPC => { R[r.Rd] = PC; }
        MOV_func.MTPC => { PC = R[r.Rs1]; }  // Privileged
        MOV_func.MFLR => { R[r.Rd] = LR; }
        MOV_func.MTLR => { LR = R[r.Rs1]; }
        MOV_func.MFSP => { R[r.Rd] = SP; }
        MOV_func.MTSP => { SP = R[r.Rs1]; }
        MOV_func.MFFP => { R[r.Rd] = FP; }
        MOV_func.MTFP => { FP = R[r.Rs1]; }

// =============================================================================
// Load Immediate Semantics
// =============================================================================

instruction decode_LDI(bits(32) enc) is
    let func = enc[27:24];
    let r = I_type{enc};

    case func of
        0b0000 => { R[r.Rd] = SignExtend(r.imm9, 64); }   // ldi
        0b0001 => { R[r.Rd] = ZeroExtend(r.imm9, 64); }   // ldiu

// =============================================================================
// Vector Instruction Semantics
// =============================================================================

instruction decode_VEC(bits(32) enc) is
    let r = V_type{enc};
    let func = VEC_func{r.func};
    let vd  = r.Vd;
    let vs1 = r.Vs1;
    let vs2 = r.Vs2;
    let ms  = r.Ms;

    case func of
        VEC_func.VADD => { V[vd] = V[vs1] + V[vs2]; }
        VEC_func.VSUB => { V[vd] = V[vs1] - V[vs2]; }
        VEC_func.VMUL => { V[vd] = V[vs1] * V[vs2]; }
        VEC_func.VDIV => { V[vd] = V[vs1] / V[vs2]; }
        VEC_func.VAND => { V[vd] = V[vs1] AND V[vs2]; }
        VEC_func.VOR  => { V[vd] = V[vs1] OR V[vs2]; }
        VEC_func.VXOR => { V[vd] = V[vs1] XOR V[vs2]; }
        VEC_func.VNOT => { V[vd] = NOT V[vs1]; }
        VEC_func.VLD_B => {
            let addr = R[enc[18:14]] + SignExtend(enc[13:5], 64);
            V[vd] = Mem128[addr];
        }
        VEC_func.VST_B => {
            let addr = R[enc[18:14]] + SignExtend(enc[13:5], 64);
            Mem128[addr] = V[vs2];
        }
        VEC_func.VLD_W => {
            let addr = R[enc[18:14]] + SignExtend(enc[13:5], 64);
            V[vd] = Mem128[addr];
        }
        VEC_func.VST_W => {
            let addr = R[enc[18:14]] + SignExtend(enc[13:5], 64);
            Mem128[addr] = V[vs2];
        }
        VEC_func.VDUP_B => {
            let scalar = R[enc[18:14]][7:0];
            V[vd] = Replicate(scalar, 16);
        }
        VEC_func.VDUP_H => {
            let scalar = R[enc[18:14]][15:0];
            V[vd] = Replicate(scalar, 8);
        }
        VEC_func.VDUP_W => {
            let scalar = R[enc[18:14]][31:0];
            V[vd] = Replicate(scalar, 4);
        }
        VEC_func.VDUP_D => {
            let scalar = R[enc[18:14]];
            V[vd] = Replicate(scalar, 2);
        }

// =============================================================================
// Mask Instruction Semantics
// =============================================================================

instruction decode_MASK(bits(32) enc) is
    let func = MASK_func{enc[27:24]};

    case func of
        MASK_func.MSET => {
            let rd = enc[23:19];
            let imm9 = enc[13:5];
            M[rd] = ZeroExtend(imm9, 64);
        }
        MASK_func.MAND => {
            let md  = enc[23:19];
            let ms1 = enc[18:14];
            let ms2 = enc[13:9];
            M[md] = M[ms1] AND M[ms2];
        }
        MASK_func.MOR => {
            let md  = enc[23:19];
            let ms1 = enc[18:14];
            let ms2 = enc[13:9];
            M[md] = M[ms1] OR M[ms2];
        }
        MASK_func.MXOR => {
            let md  = enc[23:19];
            let ms1 = enc[18:14];
            let ms2 = enc[13:9];
            M[md] = M[ms1] XOR M[ms2];
        }
        MASK_func.MNOT => {
            let md  = enc[23:19];
            let ms1 = enc[18:14];
            M[md] = NOT M[ms1];
        }
        MASK_func.MRD => {
            let rd  = enc[23:19];
            let ms1 = enc[18:14];
            R[rd] = M[ms1];
        }
        MASK_func.MWR => {
            let rd  = enc[23:19];
            let ms1 = enc[18:14];
            M[ms1] = R[rd];
        }
        MASK_func.MBS => {
            let ms1 = enc[23:19];
            let bit = enc[8:5];
            M[ms1][bit] = '1';
        }
        MASK_func.MBC => {
            let ms1 = enc[23:19];
            let bit = enc[8:5];
            M[ms1][bit] = '0';
        }
        MASK_func.MBT => {
            let rd  = enc[23:19];
            let ms1 = enc[18:14];
            let bit = enc[8:5];
            R[rd] = ZeroExtend(M[ms1][bit], 64);
        }

// =============================================================================
// Helper Functions
// =============================================================================

function SetFlags(bits(64) result) => void is
    flags.Z = if result == 0 then '1' else '0';
    flags.N = result[63];
    // C and V set by specific operations (ADD, SUB)

function SignExtend(bits(N) val, integer M) => bits(M) is
    // Sign-extend N-bit value to M bits
    return Replicate(val[N-1], M-N) :: val;

function ZeroExtend(bits(N) val, integer M) => bits(M) is
    // Zero-extend N-bit value to M bits
    return Replicate('0', M-N) :: val;

function Replicate(bits(N) val, integer count) => bits(N*count) is
    // Replicate value count times
    let result : bits(N*count);
    for i = 0 to count-1
        result[i*N +: N] = val;
    return result;

// =============================================================================
// Memory Interface
// =============================================================================

// Byte-addressable, little-endian memory
memfunc Mem8[bits(64) addr] => bits(8);
memfunc Mem16[bits(64) addr] => bits(16);
memfunc Mem32[bits(64) addr] => bits(32);
memfunc Mem64[bits(64) addr] => bits(64);
memfunc Mem128[bits(64) addr] => bits(128);

// =============================================================================
// System Interface - Skyscraper ABI
// =============================================================================
//
// These are architecture-independent syscalls defined by Skyscraper.
// Platform-specific backends (isa/x86-64/linux, isa/aarch64/linux, etc.)
// map these to the appropriate OS syscall interface.
//
// Syscall numbers:
//   0  = exit
//   1  = read
//   2  = write
//   3  = open
//   4  = close
//   5  = seek
//   6  = stat
//   7  = mmap
//   8  = munmap
//   9  = brk
//   10 = clock
//   11 = yield
//   12 = getpid
//   13 = fork
//   14 = exec
//   15 = pipe
//   16 = dup
//   17 = dup2
//   18 = ioctl
//   19 = time
//   20 = sleep
//   21 = mprotect
//   22 = getdents
//   23 = unlink
//   24 = rename
//   25 = mkdir
//   26 = rmdir
//   27 = chdir
//   28 = getcwd

// Process control
syscall Syscall_exit(bits(64) code) => void;
syscall Syscall_fork() => bits(64);
syscall Syscall_exec(bits(64) path, bits(64) argv, bits(64) envp) => bits(64);
syscall Syscall_getpid() => bits(64);
syscall Syscall_yield() => void;

// File I/O
syscall Syscall_read(bits(64) fd, bits(64) buf, bits(64) count) => bits(64);
syscall Syscall_write(bits(64) fd, bits(64) buf, bits(64) count) => bits(64);
syscall Syscall_open(bits(64) path, bits(64) flags, bits(64) mode) => bits(64);
syscall Syscall_close(bits(64) fd) => bits(64);
syscall Syscall_seek(bits(64) fd, bits(64) offset, bits(64) whence) => bits(64);
syscall Syscall_stat(bits(64) path, bits(64) buf) => bits(64);
syscall Syscall_dup(bits(64) fd) => bits(64);
syscall Syscall_dup2(bits(64) oldfd, bits(64) newfd) => bits(64);
syscall Syscall_ioctl(bits(64) fd, bits(64) request, bits(64) arg) => bits(64);

// Directory operations
syscall Syscall_getdents(bits(64) fd, bits(64) buf, bits(64) count) => bits(64);
syscall Syscall_unlink(bits(64) path) => bits(64);
syscall Syscall_rename(bits(64) oldpath, bits(64) newpath) => bits(64);
syscall Syscall_mkdir(bits(64) path, bits(64) mode) => bits(64);
syscall Syscall_rmdir(bits(64) path) => bits(64);
syscall Syscall_chdir(bits(64) path) => bits(64);
syscall Syscall_getcwd(bits(64) buf, bits(64) size) => bits(64);

// Memory management
syscall Syscall_mmap(bits(64) addr, bits(64) len, bits(64) prot, bits(64) flags, bits(64) fd, bits(64) offset) => bits(64);
syscall Syscall_munmap(bits(64) addr, bits(64) len) => bits(64);
syscall Syscall_brk(bits(64) addr) => bits(64);
syscall Syscall_mprotect(bits(64) addr, bits(64) len, bits(64) prot) => bits(64);

// Time
syscall Syscall_clock() => bits(64);
syscall Syscall_time(bits(64) buf) => bits(64);
syscall Syscall_sleep(bits(64) nanoseconds) => void;

// IPC
syscall Syscall_pipe(bits(64) fds) => bits(64);
