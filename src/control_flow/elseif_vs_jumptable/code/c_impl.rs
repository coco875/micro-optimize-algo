//! FFI bindings for C implementations of elseif_vs_jumptable.

#[cfg(c_implementation_active)]
mod ffi {
    extern "C" {
        pub fn dispatch_operation_c_elseif(opcode: u8, value: u32) -> u32;
        pub fn dispatch_operation_c_switch(opcode: u8, value: u32) -> u32;
    }
}

/// C if-else if chain implementation
#[cfg(c_implementation_active)]
pub fn dispatch_operation_c_elseif(opcode: u8, value: u32) -> u32 {
    unsafe { ffi::dispatch_operation_c_elseif(opcode, value) }
}

/// C switch implementation
#[cfg(c_implementation_active)]
pub fn dispatch_operation_c_switch(opcode: u8, value: u32) -> u32 {
    unsafe { ffi::dispatch_operation_c_switch(opcode, value) }
}

/// Check if C implementations are available
#[cfg(c_implementation_active)]
pub const C_IMPL_AVAILABLE: bool = true;

#[cfg(not(c_implementation_active))]
pub const C_IMPL_AVAILABLE: bool = false;

/// Name of the C compiler used
#[cfg(c_implementation_active)]
pub const COMPILER_NAME: Option<&str> = Some(env!("C_COMPILER_NAME"));

#[cfg(not(c_implementation_active))]
pub const COMPILER_NAME: Option<&str> = None;

// Stubs for missing C compiler
#[cfg(not(c_implementation_active))]
pub fn dispatch_operation_c_elseif(_opcode: u8, _value: u32) -> u32 {
    panic!("C implementation not compiled (requires GCC/Clang)")
}

#[cfg(not(c_implementation_active))]
pub fn dispatch_operation_c_switch(_opcode: u8, _value: u32) -> u32 {
    panic!("C implementation not compiled (requires GCC/Clang)")
}
