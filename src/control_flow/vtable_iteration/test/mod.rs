//! Tests for vtable_iteration module.

use super::code::get_variants;

pub fn verify_all() -> Result<(), String> {
    const TEST_SIZE: usize = 100;
    const SEED: u64 = 0x12345678;

    for variant in get_variants() {
        let mut pool = (variant.create)(TEST_SIZE, SEED);
        let result = pool.iterate();
        
        if result == 0 {
            return Err(format!("{} returned 0", variant.name));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::code::*;

    #[test]
    fn test_verify_all() {
        verify_all().expect("Verification failed");
    }

    #[test]
    fn test_vtable_consistency() {
        let obj_a = ObjectA::new(42);
        let obj_b = ObjectB::new(42);
        
        assert_eq!(obj_a.vtable as *const VTable, &VTABLE_A as *const VTable);
        assert_eq!(obj_b.vtable as *const VTable, &VTABLE_B as *const VTable);
    }

    #[test]
    fn test_process_modifies_data() {
        let mut obj = ObjectA::new(0);
        let initial_data = obj.data.clone();
        
        unsafe {
            (obj.vtable.process)(&mut obj as *mut ObjectA as *mut u8);
        }
        
        assert_ne!(obj.data, initial_data);
    }
}
