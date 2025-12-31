# Call vs Branch (Inline) Comparison

## Overview

This module demonstrates the performance difference between **function calls** (`CALL`/`RET` instructions) and **inline code** in x86_64 assembly.

## The Problem

Function calls have inherent overhead:
1. **CALL instruction**: Pushes 8-byte return address onto stack (~1 cycle)
2. **RET instruction**: Pops return address and jumps (~1 cycle)
3. **Stack pointer manipulation**: Sub/Add to RSP for local variables
4. **Register saving**: Callee-saved registers must be preserved
5. **Return address prediction**: Uses Return Stack Buffer (RSB)

For small, frequently-called functions, this overhead can dominate execution time.

## Implementations

### 1. Original (Rust with `#[inline(never)]`)
```rust
#[inline(never)]
fn double(x: u32) -> u32 { x * 2 }

#[inline(never)]
fn add_ten(x: u32) -> u32 { x + 10 }

#[inline(never)]
fn square(x: u32) -> u32 { x * x }

pub fn process(value: u32) -> u32 {
    square(add_ten(double(value)))
}
```

Each function call generates: `CALL` → function body → `RET`

### 2. x86_64-asm-call (Explicit CALL/RET)
```asm
process_with_calls:
    mov eax, edi
    call double
    call add_ten
    call square
    ret

double:
    add eax, eax
    ret

add_ten:
    add eax, 10
    ret

square:
    imul eax, eax
    ret
```

**Overhead per call:**
- CALL: Push return address (~1 cycle)
- RET: Pop and jump (~1-2 cycles)
- Total: ~3-5 cycles per call/ret pair

### 3. x86_64-asm-inline (No CALL)
```asm
process_inline:
    mov eax, edi
    add eax, eax      ; double
    add eax, 10       ; add_ten
    imul eax, eax     ; square
    ret
```

**Overhead:** Zero call overhead, just sequential execution.

## Performance Analysis

| Metric | CALL/RET | Inline |
|--------|----------|--------|
| Instructions | More (CALL/RET pairs) | Fewer |
| Cycles overhead | ~3-5 per call | 0 |
| Stack operations | Yes | No |
| Code size | Smaller (shared) | Larger (duplicated) |
| I-cache pressure | Lower | Higher |

## When to Use Each

### Use Function Calls When:
- Function is large (>20 instructions)
- Called from many places (reduces code size)
- Performance is not critical
- Debugging/profiling benefits from stack traces

### Use Inline When:
- Function is small (<5 instructions)
- Called in hot loops
- Performance is critical
- Function is called from few places

## Return Stack Buffer (RSB)

Modern CPUs predict return addresses using a hardware stack:

```
CALL foo    → RSB.push(return_addr)
  ...
RET         → predicted_addr = RSB.pop()
```

**Misprediction scenarios:**
1. RSB overflow (deep recursion)
2. Mismatched CALL/RET (longjmp, exceptions)
3. Spectre-style attacks (RSB poisoning)

Misprediction penalty: ~15-20 cycles

## Compiler Inlining Heuristics

Compilers automatically inline based on:
- Function size (small → likely inline)
- Call frequency (hot → likely inline)
- `#[inline]` hints
- Optimization level (-O2, -O3)
- LTO (Link-Time Optimization)

Force behavior with:
- `#[inline(always)]` - Force inline
- `#[inline(never)]` - Force call
- `#[cold]` - Hint that function is rarely called

## Expected Results

For the 3-operation chain:
- **CALL version**: ~9-15 cycles overhead (3 calls × 3-5 cycles)
- **Inline version**: ~0 cycles overhead

Total speedup from inlining: **significant for small functions in hot paths**
