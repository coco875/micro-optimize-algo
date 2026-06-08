#![allow(clippy::missing_safety_doc)]
//! Object definitions with manual C repr vtables.
//!
//! Each object has a `vtable` field at the beginning that points to a struct
//! containing function pointers.

/// Manual vtable - list of function pointers
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VTable {
    /// Process function that performs computation on the object
    pub process: unsafe extern "C" fn(*mut u8) -> u64,
}

// ============================================================================
// Diverse objects with different sizes
// ============================================================================

// to help get the vtable from the object else same as other objects
#[repr(C)]
pub struct ObjectVTable {
    pub vtable: &'static VTable,
}

/// Object A - Very small (12 bytes of data)
#[repr(C)]
pub struct ObjectA {
    pub vtable: &'static VTable,
    pub data: [u8; 12],
}

/// Object B - Small (16 bytes of data)
#[repr(C)]
pub struct ObjectB {
    pub vtable: &'static VTable,
    pub data: [u8; 16],
}

/// Object C - Medium (32 bytes of data)
#[repr(C)]
pub struct ObjectC {
    pub vtable: &'static VTable,
    pub data: [u8; 32],
}

/// Object D - Medium with complex computation (24 bytes + f32)
#[repr(C)]
pub struct ObjectD {
    pub vtable: &'static VTable,
    pub data: [u8; 24],
    pub multiplier: f32,
}

/// Object E - Large (48 bytes + counter)
#[repr(C)]
pub struct ObjectE {
    pub vtable: &'static VTable,
    pub data: [u8; 48],
    pub counter: u32,
}

/// Object F - Very large (64 bytes)
#[repr(C)]
pub struct ObjectF {
    pub vtable: &'static VTable,
    pub data: [u8; 64],
}

/// Object G - Tiny (8 bytes)
#[repr(C)]
pub struct ObjectG {
    pub vtable: &'static VTable,
    pub data: [u8; 8],
}

// ============================================================================
// Process function implementations for each object
// ============================================================================

pub unsafe extern "C" fn process_a(ptr: *mut u8) -> u64 {
    let obj = &mut *(ptr as *mut ObjectA);
    let mut sum: u64 = 0;
    for b in obj.data.iter_mut() {
        *b = b.wrapping_add(1);
        sum = sum.wrapping_add(*b as u64);
    }
    sum
}

pub unsafe extern "C" fn process_b(ptr: *mut u8) -> u64 {
    let obj = &mut *(ptr as *mut ObjectB);
    let mut sum: u64 = 0;
    for b in obj.data.iter_mut() {
        *b = b.wrapping_add(2);
        sum = sum.wrapping_add(*b as u64);
    }
    sum
}

pub unsafe extern "C" fn process_c(ptr: *mut u8) -> u64 {
    let obj = &mut *(ptr as *mut ObjectC);
    let mut sum: u64 = 0;
    for b in obj.data.iter_mut() {
        *b = b.wrapping_add(3);
        sum = sum.wrapping_add(*b as u64);
    }
    sum
}

pub unsafe extern "C" fn process_d(ptr: *mut u8) -> u64 {
    let obj = &mut *(ptr as *mut ObjectD);
    let mut sum: u64 = 0;
    for b in obj.data.iter_mut() {
        *b = b.wrapping_add(4);
        sum = sum.wrapping_add(*b as u64);
    }
    // Computation with multiplier
    (sum as f32 * obj.multiplier) as u64
}

pub unsafe extern "C" fn process_e(ptr: *mut u8) -> u64 {
    let obj = &mut *(ptr as *mut ObjectE);
    obj.counter = obj.counter.wrapping_add(1);
    let mut sum: u64 = obj.counter as u64;
    for b in obj.data.iter_mut() {
        *b = b.wrapping_add(5);
        sum = sum.wrapping_add(*b as u64);
    }
    sum
}

pub unsafe extern "C" fn process_f(ptr: *mut u8) -> u64 {
    let obj = &mut *(ptr as *mut ObjectF);
    let mut sum: u64 = 0;
    for b in obj.data.iter_mut() {
        *b = b.wrapping_add(6);
        sum = sum.wrapping_add(*b as u64);
    }
    sum
}

pub unsafe extern "C" fn process_g(ptr: *mut u8) -> u64 {
    let obj = &mut *(ptr as *mut ObjectG);
    let mut sum: u64 = 0;
    for b in obj.data.iter_mut() {
        *b = b.wrapping_add(7);
        sum = sum.wrapping_add(*b as u64);
    }
    sum
}

// ============================================================================
// Static vtables for each object type
// ============================================================================

pub static VTABLE_A: VTable = VTable { process: process_a };
pub static VTABLE_B: VTable = VTable { process: process_b };
pub static VTABLE_C: VTable = VTable { process: process_c };
pub static VTABLE_D: VTable = VTable { process: process_d };
pub static VTABLE_E: VTable = VTable { process: process_e };
pub static VTABLE_F: VTable = VTable { process: process_f };
pub static VTABLE_G: VTable = VTable { process: process_g };

pub static VTABLE_PTRS: [&VTable; 7] = [
    &VTABLE_A, &VTABLE_B, &VTABLE_C, &VTABLE_D, &VTABLE_E, &VTABLE_F, &VTABLE_G,
];

// ============================================================================
// Constructors
// ============================================================================

impl ObjectA {
    pub fn new(seed: u8) -> Self {
        Self {
            vtable: &VTABLE_A,
            data: [seed; 12],
        }
    }
}

impl ObjectB {
    pub fn new(seed: u8) -> Self {
        Self {
            vtable: &VTABLE_B,
            data: [seed; 16],
        }
    }
}

impl ObjectC {
    pub fn new(seed: u8) -> Self {
        Self {
            vtable: &VTABLE_C,
            data: [seed; 32],
        }
    }
}

impl ObjectD {
    pub fn new(seed: u8, multiplier: f32) -> Self {
        Self {
            vtable: &VTABLE_D,
            data: [seed; 24],
            multiplier,
        }
    }
}

impl ObjectE {
    pub fn new(seed: u8) -> Self {
        Self {
            vtable: &VTABLE_E,
            data: [seed; 48],
            counter: 0,
        }
    }
}

impl ObjectF {
    pub fn new(seed: u8) -> Self {
        Self {
            vtable: &VTABLE_F,
            data: [seed; 64],
        }
    }
}

impl ObjectG {
    pub fn new(seed: u8) -> Self {
        Self {
            vtable: &VTABLE_G,
            data: [seed; 8],
        }
    }
}

// ============================================================================
// Helper to call process via vtable
// ============================================================================

/// Calls the process function via the object's vtable
///
/// # Safety
/// Pointer must point to a valid object with a valid vtable
#[inline]
pub unsafe fn call_process(ptr: *mut u8) -> u64 {
    let vtable = *(ptr as *const *const VTable);
    ((*vtable).process)(ptr)
}

// ============================================================================
// Size and alignment helpers
// ============================================================================

/// Number of object types
pub const OBJECT_TYPE_COUNT: usize = 7;

/// Array of all object sizes
pub const OBJECT_SIZES: [usize; OBJECT_TYPE_COUNT] = [
    std::mem::size_of::<ObjectA>(),
    std::mem::size_of::<ObjectB>(),
    std::mem::size_of::<ObjectC>(),
    std::mem::size_of::<ObjectD>(),
    std::mem::size_of::<ObjectE>(),
    std::mem::size_of::<ObjectF>(),
    std::mem::size_of::<ObjectG>(),
];

/// Array of all object alignments
pub const OBJECT_ALIGNS: [usize; OBJECT_TYPE_COUNT] = [
    std::mem::align_of::<ObjectA>(),
    std::mem::align_of::<ObjectB>(),
    std::mem::align_of::<ObjectC>(),
    std::mem::align_of::<ObjectD>(),
    std::mem::align_of::<ObjectE>(),
    std::mem::align_of::<ObjectF>(),
    std::mem::align_of::<ObjectG>(),
];

/// Helper to find max in array at compile time
const fn const_array_max(arr: &[usize]) -> usize {
    let mut max = 0;
    let mut i = 0;
    while i < arr.len() {
        if arr[i] > max {
            max = arr[i];
        }
        i += 1;
    }
    max
}

/// Maximum size of any object type
pub const MAX_OBJECT_SIZE: usize = const_array_max(&OBJECT_SIZES);

/// Maximum alignment of any object type
pub const MAX_OBJECT_ALIGN: usize = const_array_max(&OBJECT_ALIGNS);

/// Padded size that can hold any object type (aligned)
pub const PADDED_OBJECT_SIZE: usize =
    (MAX_OBJECT_SIZE + MAX_OBJECT_ALIGN - 1) & !(MAX_OBJECT_ALIGN - 1);
