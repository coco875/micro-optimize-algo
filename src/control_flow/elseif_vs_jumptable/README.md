# Branch vs Jump Table vs Branchless Comparison

## Overview

This module compares three fundamental approaches to multi-way dispatch in assembly:

1. **Branch (Jcc)**: Conditional jumps based on comparisons
2. **Jump Table**: Indexed lookup for O(1) dispatch
3. **Branchless (CMOV)**: Conditional moves without branching

## The Problem

When implementing a switch/match with many cases:
- **Branches** are subject to misprediction (10-20 cycle penalty)
- **Jump tables** have constant time but indirect branch overhead
- **Branchless** executes all code but never mispredicts

## Implementations

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
- Each branch can mispredict (10-20 cycles penalty)
- Early cases are faster than late cases
- Good for predictable patterns (e.g., always opcode 0)

### 3. x86_64-asm-jumptable (Lookup Table)
```asm
dispatch_jumptable:
    cmp edi, 7
    ja .invalid
    mov eax, [multiplier_table + rdi*4]  ; Load multiplier
    imul eax, esi                         ; Compute result
```

**Characteristics:**
- O(1) time complexity
- Same time for all valid opcodes
- One indirect memory access
- Good for uniform distribution

### 4. x86_64-asm-branchless (CMOV)
```asm
dispatch_branchless:
    ; Compute result speculatively
    mov eax, [table + rdi*4]
    imul eax, esi
    ; Use CMOV to handle invalid case
    xor ecx, ecx
    cmp edi, 7
    cmova eax, ecx    ; If invalid, result = 0
```

**Characteristics:**
- O(1) time complexity
- NO branch misprediction possible
- Always executes all instructions
- Best for random/unpredictable data

## Performance Comparison

| Metric | Branch (Jcc) | Jump Table | Branchless (CMOV) |
|--------|--------------|------------|-------------------|
| Best case | Very fast | Constant | Constant |
| Worst case | Slow (mispredictions) | Constant | Constant |
| Random data | Poor | Good | Best |
| Predictable data | Best | Good | Good |
| Code size | Medium | Small + table | Small |
| Instructions | Variable | Few | Fixed |

## Branch Prediction

Modern CPUs use sophisticated predictors:
- **2-bit saturating counter**: Predicts based on recent history
- **Pattern-based**: Learns sequences (TTNTTNT...)
- **Neural predictors**: ML-based prediction (modern Intel/AMD)

Even the best predictors fail on truly random data → ~50% misprediction rate.

## When to Use Which?

| Data Pattern | Best Choice |
|--------------|-------------|
| Always same opcode | Branch (first position) |
| Mostly one opcode (90%+) | Branch (put hot case first) |
| Uniform random | Jump table or Branchless |
| Multiple hot cases | Jump table |
| Security-critical (timing) | Branchless (constant time) |

## x86_64 Instructions Reference

| Instruction | Type | Description |
|------------|------|-------------|
| `JE`, `JNE`, `JG`, `JL` | Branch | Conditional jump (can mispredict) |
| `JMP` | Jump | Unconditional jump |
| `CMOVE`, `CMOVNE`, `CMOVG` | CMOV | Conditional move (no branch) |
| `CMP`, `TEST` | Compare | Set flags for Jcc/CMOVcc |

## Expected Results

With uniformly random opcodes (0-7):
- **Branch**: Variable, depends on position and prediction
- **Jump table**: Consistent, ~3-5 cycles for lookup + multiply
- **Branchless**: Most consistent, no variance from misprediction

With skewed distribution (90% opcode 0):
- **Branch**: Very fast (opcode 0 checked first)
- **Jump table**: Same as before
- **Branchless**: Same as before
