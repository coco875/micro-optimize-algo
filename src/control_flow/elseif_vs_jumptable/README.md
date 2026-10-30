# Branch vs Jump Table vs Branchless Comparison

## Overview

This module compares three fundamental approaches to multi-way dispatch in assembly:

1. **Branch (Jcc)**: Conditional jumps based on comparisons
2. **Jump Table**: Indexed lookup for O(1) dispatch
3. **Branchless (CMOV)**: Conditional moves without branching

## The Problem

When implementing a switch/match with many cases:

- **Branches** can incur conditional-branch mispredictions.
- **Jump tables** use a table lookup and an indirect jump to a case handler.
- **Branchless** uses conditional moves and a multiplier lookup instead of dispatch branches.

Cycle counts and relative performance depend on the machine and workload.

## Implementations

CLI filters use the registered IDs `Original`, `X86_64AsmBranch`, `X86_64AsmJumpTable`, and `X86_64AsmBranchless`. The assembly implementations are enabled on x86_64. When C compilation is available, `CElseIf` and `CSwitch` add C comparisons using an else-if chain and a switch statement.

The assembly snippets below illustrate the dispatch strategies; register allocation and labels differ in the inline assembly source.

### 1. Original (Rust Match)
```rust
match opcode {
    0 => value,
    1 => value * 2,
    2 => value * 3,
    // ...
    7 => value * 8,
    _ => 0,
}
```

### 2. x86_64-asm-branch (Conditional Branches)
```asm
dispatch_branch:
    cmp edi, 0
    je .case_0      ; BRANCH - may mispredict!
    cmp edi, 1
    je .case_1      ; BRANCH - may mispredict!
    cmp edi, 2
    je .case_2      ; BRANCH - may mispredict!
    ; ...
```

**Characteristics:**

- O(n) time complexity
- Each conditional branch can mispredict
- Early cases require fewer comparisons than late cases
- Good for predictable patterns (e.g., always opcode 0)

### 3. x86_64-asm-jumptable (Indirect Jump)

```asm
dispatch_jumptable:
    cmp edi, 7
    ja .invalid
    lea rax, [rip + jump_table]
    movsxd rcx, dword ptr [rax + rdi*4] ; Load signed case offset
    add rax, rcx
    jmp rax                           ; Jump to selected case handler

jump_table:
    .long .case_0 - jump_table
    .long .case_1 - jump_table
    ; ... eight case offsets in total

.case_0:
    mov eax, esi
    jmp .done
.case_1:
    lea eax, [rsi + rsi]
    jmp .done
    ; ... remaining handlers, invalid case, and return omitted
```

**Characteristics:**

- O(1) time complexity
- One bounds-check branch and an indirect jump
- A table of signed offsets selects separate arithmetic handlers
- Prediction and the selected handler affect measured time

### 4. x86_64-asm-branchless (CMOV)
```asm
dispatch_branchless:
    ; Clamp invalid opcodes before accessing the multiplier table
    mov ecx, edi
    cmp edi, 7
    mov edx, 0
    cmova ecx, edx
    lea rax, [rip + multiplier_table]
    mov eax, [rax + rcx*4]
    imul eax, esi
    ; IMUL changes flags, so compare again before selecting zero
    cmp edi, 7
    mov edx, 0
    cmova eax, edx
```

**Characteristics:**

- O(1) time complexity
- No conditional or indirect dispatch jump
- Clamps invalid opcodes to a valid table index before loading
- Uses conditional moves to return zero for invalid opcodes

## Performance Comparison

| Metric | Branch (Jcc) | Jump Table | Branchless (CMOV) |
|--------|--------------|------------|-------------------|
| Dispatch | Comparison chain | Offset lookup and indirect jump | Multiplier lookup and conditional moves |
| Prediction | Conditional branches | Bounds check and indirect jump | No dispatch branches |
| Arithmetic | Selected case handler | Selected case handler | Multiply by selected table value |
| Instruction path | Depends on opcode | Depends on selected handler | Same instruction sequence |

The table describes code structure. Benchmark the implementations on the target machine to compare performance.

## Workload and validation

The `random` scenario generates uniformly distributed valid opcodes (0–7) and values from 1 to 1000 before timing. Each sample measures dispatch over the entire input vector. Every implementation receives equivalent logical inputs.

Dedicated validation cases compare each output vector exactly against the Rust reference. Validation runs separately from timing. Skewed opcode distributions require a new scenario; they are not selected by the current CLI filters.

```bash
cargo run --release -- --verify --scenario elseif_vs_jumptable --seed 42
cargo run --release -- --scenario elseif_vs_jumptable --runs 30 --seed 42
```

Set benchmark sizes in an `[elseif_vs_jumptable]` TOML table, following the root [README.md](../../../README.md). See [BENCHMARKING.md](../../../BENCHMARKING.md) for timing units and replay behavior.

## x86_64 Instructions Reference

| Instruction | Type | Description |
|------------|------|-------------|
| `JE`, `JNE`, `JG`, `JL` | Branch | Conditional jump (can mispredict) |
| `JMP` | Jump | Unconditional jump |
| `CMOVE`, `CMOVNE`, `CMOVG` | CMOV | Conditional move (no branch) |
| `CMP`, `TEST` | Compare | Set flags for Jcc/CMOVcc |
