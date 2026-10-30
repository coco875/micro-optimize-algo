# VTable Iteration Benchmark

## Overview

Compares different iteration strategies over a heterogeneous pool of polymorphic objects. Objects use manual `repr(C)` vtables. The scenario owns each pool through `Box<dyn ObjectPool + Send>`, so the timed workload includes one call through the pool trait before its iteration strategy runs.

## Objects

7 object types with different sizes, each with a `vtable` field as the first member:

| Object | Data Size | Notes |
|--------|-----------|-------|
| `ObjectA` | 12 bytes | Very small |
| `ObjectB` | 16 bytes | Small |
| `ObjectC` | 32 bytes | Medium |
| `ObjectD` | 24 bytes + f32 | With multiplier |
| `ObjectE` | 48 bytes + u32 | With counter |
| `ObjectF` | 64 bytes | Very large |
| `ObjectG` | 8 bytes | Tiny |

## Variants

| Registered CLI ID | Implementation | Description |
|---|---|---|
| `VTableContiguous` | `vtable_contiguous` | All objects in a contiguous buffer, dispatch via vtable pointer |
| `SegmentedPools` | `segmented_pools` | Separate `Vec<ObjectX>` per type, direct function calls |
| `OneTypeAtTime` | `one_type_at_time` | Contiguous buffer, process all objects of one vtable type before moving to next |

## Historical results (size 16384, 1000 runs)

These results predate the runner refactor and have not been revalidated with the current setup and timing boundaries. The machine, seed, and compiler flags were not recorded here. Rerun the benchmark before using these numbers for current performance comparisons.

| Variant | Average | Speedup |
|---------|---------|---------|
| `segmented_pools` | 288K cycles | **2.43x** |
| `vtable_contiguous` | 533K cycles | 1.31x |
| `one_type_at_time` | 699K cycles | 1.00x |

## Iteration strategies

1. **Segmented pools**:
   - Better cache locality (objects of same type are contiguous)
   - Direct function calls instead of indirect vtable dispatch
   - Single pass over data

2. **One-type-at-time**:
   - Makes 7 passes over the entire buffer (one per vtable type)
   - Poor cache utilization from repeated traversals
   - Comparison overhead for each object

3. **Vtable contiguous**:
   - Single pass over data
   - Indirect dispatch overhead via vtable pointer

## Workload and usage

The `mixed` scenario prepares one logical sequence of object types and initial values for each sample, then builds each implementation's pool before timing. One pool iteration is measured. Dedicated validation cases compare the outputs of four successive iterations against the reference, using fresh pools.

```bash
cargo run --release -- --verify --scenario vtable_iteration --seed 42
cargo run --release -- --scenario vtable_iteration --runs 30 --seed 42
```

Set benchmark sizes with `[vtable_iteration] sizes = [...]` in a TOML configuration, using the table syntax shown in the root [README.md](../../../README.md). See [BENCHMARKING.md](../../../BENCHMARKING.md) for timing units and replay behavior.

## Structure

```
vtable_iteration/
├── README.md           # This file
├── mod.rs              # System contract and discovered modules
├── code/
│   ├── mod.rs          # Module exports
│   ├── objects.rs      # Object definitions with manual vtables
│   └── variants.rs     # Pool implementations (ObjectPool trait)
├── scenarios/
│   └── mixed.rs        # Shared object fixtures, timing, and typed validation
└── test/
    └── mod.rs          # Verification tests
```
