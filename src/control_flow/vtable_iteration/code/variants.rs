#![allow(clippy::missing_safety_doc)]
//! Iteration variants to compare.

use super::objects::*;
use std::alloc::{alloc, dealloc, Layout};

// ============================================================================
// Common trait for all pool variants
// ============================================================================

/// Trait for object pool implementations
pub trait ObjectPool: Send {
    /// Create a new pool with the given size and seed
    fn new(objects: &[(u8, u8)]) -> Self
    where
        Self: Sized;
    /// Iterate over all objects and return a sum
    fn iterate(&mut self) -> u64;
}
use crate::control_flow::vtable_iteration::VTableIterationVariant;
use crate::variant;

// ============================================================================
// Common data generator
// ============================================================================

#[inline]
pub unsafe fn write_object_at(ptr: *mut u8, type_idx: u8, seed_byte: u8) {
    match type_idx {
        0 => std::ptr::write(ptr as *mut ObjectA, ObjectA::new(seed_byte)),
        1 => std::ptr::write(ptr as *mut ObjectB, ObjectB::new(seed_byte)),
        2 => std::ptr::write(ptr as *mut ObjectC, ObjectC::new(seed_byte)),
        3 => std::ptr::write(ptr as *mut ObjectD, ObjectD::new(seed_byte, 1.5)),
        4 => std::ptr::write(ptr as *mut ObjectE, ObjectE::new(seed_byte)),
        5 => std::ptr::write(ptr as *mut ObjectF, ObjectF::new(seed_byte)),
        _ => std::ptr::write(ptr as *mut ObjectG, ObjectG::new(seed_byte)),
    }
}

// ============================================================================
// Variant 1: Contiguous buffer with vtable dispatch
// ============================================================================

pub struct RawVTablePool {
    buffer: *mut u8,
    layout: Layout,
    entries: Vec<usize>,
}

unsafe impl Send for RawVTablePool {}

impl ObjectPool for RawVTablePool {
    fn new(objects: &[(u8, u8)]) -> Self {
        let count = objects.len();
        let padded_size = PADDED_OBJECT_SIZE;
        let total_size = padded_size * count;
        let layout = Layout::from_size_align(total_size, MAX_OBJECT_ALIGN).unwrap();

        let buffer = unsafe { alloc(layout) };
        let mut entries = Vec::with_capacity(count);
        let generator = objects.iter().copied();

        for (i, (type_idx, seed_byte)) in generator.enumerate() {
            let offset = i * padded_size;
            let ptr = unsafe { buffer.add(offset) };
            unsafe { write_object_at(ptr, type_idx, seed_byte) };
            entries.push(offset);
        }

        Self {
            buffer,
            layout,
            entries,
        }
    }

    fn iterate(&mut self) -> u64 {
        let mut sum: u64 = 0;
        for &offset in &self.entries {
            let ptr = unsafe { self.buffer.add(offset) };
            sum = sum.wrapping_add(unsafe { call_process(ptr) });
        }
        sum
    }
}

impl Drop for RawVTablePool {
    fn drop(&mut self) {
        unsafe { dealloc(self.buffer, self.layout) };
    }
}

// ============================================================================
// Variant 2: Segmented pools with direct calls
// ============================================================================

pub struct SegmentedPool {
    pool_a: Vec<ObjectA>,
    pool_b: Vec<ObjectB>,
    pool_c: Vec<ObjectC>,
    pool_d: Vec<ObjectD>,
    pool_e: Vec<ObjectE>,
    pool_f: Vec<ObjectF>,
    pool_g: Vec<ObjectG>,
}

impl ObjectPool for SegmentedPool {
    fn new(objects: &[(u8, u8)]) -> Self {
        let mut pool = Self {
            pool_a: Vec::new(),
            pool_b: Vec::new(),
            pool_c: Vec::new(),
            pool_d: Vec::new(),
            pool_e: Vec::new(),
            pool_f: Vec::new(),
            pool_g: Vec::new(),
        };

        let generator = objects.iter().copied();
        for (type_idx, seed_byte) in generator {
            match type_idx {
                0 => pool.pool_a.push(ObjectA::new(seed_byte)),
                1 => pool.pool_b.push(ObjectB::new(seed_byte)),
                2 => pool.pool_c.push(ObjectC::new(seed_byte)),
                3 => pool.pool_d.push(ObjectD::new(seed_byte, 1.5)),
                4 => pool.pool_e.push(ObjectE::new(seed_byte)),
                5 => pool.pool_f.push(ObjectF::new(seed_byte)),
                _ => pool.pool_g.push(ObjectG::new(seed_byte)),
            }
        }
        pool
    }

    fn iterate(&mut self) -> u64 {
        let mut sum: u64 = 0;

        for obj in &mut self.pool_a {
            sum = sum.wrapping_add(unsafe { process_a(obj as *mut ObjectA as *mut u8) });
        }
        for obj in &mut self.pool_b {
            sum = sum.wrapping_add(unsafe { process_b(obj as *mut ObjectB as *mut u8) });
        }
        for obj in &mut self.pool_c {
            sum = sum.wrapping_add(unsafe { process_c(obj as *mut ObjectC as *mut u8) });
        }
        for obj in &mut self.pool_d {
            sum = sum.wrapping_add(unsafe { process_d(obj as *mut ObjectD as *mut u8) });
        }
        for obj in &mut self.pool_e {
            sum = sum.wrapping_add(unsafe { process_e(obj as *mut ObjectE as *mut u8) });
        }
        for obj in &mut self.pool_f {
            sum = sum.wrapping_add(unsafe { process_f(obj as *mut ObjectF as *mut u8) });
        }
        for obj in &mut self.pool_g {
            sum = sum.wrapping_add(unsafe { process_g(obj as *mut ObjectG as *mut u8) });
        }
        sum
    }
}

// ============================================================================
// 3. only process one type at a time
// ============================================================================

pub struct OneTimeVTablePool {
    buffer: *mut u8,
    layout: Layout,
    entries: Vec<usize>,
}

unsafe impl Send for OneTimeVTablePool {}

impl ObjectPool for OneTimeVTablePool {
    fn new(objects: &[(u8, u8)]) -> Self {
        let count = objects.len();
        let padded_size = PADDED_OBJECT_SIZE;
        let total_size = padded_size * count;
        let layout = Layout::from_size_align(total_size, MAX_OBJECT_ALIGN).unwrap();

        let buffer = unsafe { alloc(layout) };
        let mut entries = Vec::with_capacity(count);
        let generator = objects.iter().copied();

        for (i, (type_idx, seed_byte)) in generator.enumerate() {
            let offset = i * padded_size;
            let ptr = unsafe { buffer.add(offset) };
            unsafe { write_object_at(ptr, type_idx, seed_byte) };
            entries.push(offset);
        }

        Self {
            buffer,
            layout,
            entries,
        }
    }

    fn iterate(&mut self) -> u64 {
        let mut sum: u64 = 0;
        for &vtable in &VTABLE_PTRS {
            for &offset in &self.entries {
                let ptr = unsafe { self.buffer.add(offset) };
                let obj = unsafe { &*(ptr as *const ObjectVTable) };
                if !std::ptr::eq(obj.vtable, vtable) {
                    continue;
                }
                sum = sum.wrapping_add(unsafe { (vtable.process)(ptr) });
            }
        }
        sum
    }
}

impl Drop for OneTimeVTablePool {
    fn drop(&mut self) {
        unsafe { dealloc(self.buffer, self.layout) };
    }
}

// ============================================================================
// Variant registry
// ============================================================================

#[variant(group = "vtable_iteration")]
pub struct VTableContiguous;

impl VTableIterationVariant for VTableContiguous {
    fn name() -> &'static str {
        "vtable_contiguous"
    }
    fn create(objects: &[(u8, u8)]) -> Box<dyn ObjectPool + Send> {
        Box::new(RawVTablePool::new(objects))
    }
}

#[variant(group = "vtable_iteration")]
pub struct SegmentedPools;

impl VTableIterationVariant for SegmentedPools {
    fn name() -> &'static str {
        "segmented_pools"
    }
    fn create(objects: &[(u8, u8)]) -> Box<dyn ObjectPool + Send> {
        Box::new(SegmentedPool::new(objects))
    }
}

#[variant(group = "vtable_iteration")]
pub struct OneTypeAtTime;

impl VTableIterationVariant for OneTypeAtTime {
    fn name() -> &'static str {
        "one_type_at_time"
    }
    fn create(objects: &[(u8, u8)]) -> Box<dyn ObjectPool + Send> {
        Box::new(OneTimeVTablePool::new(objects))
    }
}
