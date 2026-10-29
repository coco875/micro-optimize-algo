# VTable Iteration Benchmark

## Overview

Compares different iteration strategies over a heterogeneous pool of polymorphic objects using manual C repr vtables (no `dyn` traits, no enums).

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

| Variant | Description |
|---------|-------------|
| `vtable_contiguous` | All objects in a contiguous buffer, dispatch via vtable pointer |
| `segmented_pools` | Separate `Vec<ObjectX>` per type, direct function calls |
| `one_type_at_time` | Contiguous buffer, process all objects of one vtable type before moving to next |

## Results (Size 16384, 1000 runs)

| Variant | Average | Speedup |
|---------|---------|---------|
| `segmented_pools` | 288K cycles | **2.43x** |
| `vtable_contiguous` | 533K cycles | 1.31x |
| `one_type_at_time` | 699K cycles | 1.00x |

## Key Findings

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
