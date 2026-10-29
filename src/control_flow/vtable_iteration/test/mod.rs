//! Tests for vtable_iteration module.

#[cfg(test)]
mod tests {
    use crate::control_flow::vtable_iteration::code::variants::{
        OneTypeAtTime, SegmentedPools, VTableContiguous,
    };
    use crate::control_flow::vtable_iteration::code::*;
    use crate::control_flow::vtable_iteration::VTableIterationVariant;

    fn verify_pool<V: VTableIterationVariant>(name: &str) {
        let mut pool = V::create(
            &(0..100)
                .map(|i| ((i % 7) as u8, i as u8))
                .collect::<Vec<_>>(),
        );
        let result = pool.iterate();
        assert!(result > 0, "{} returned 0", name);
    }

    #[test]
    fn test_verify_all() {
        verify_pool::<VTableContiguous>("vtable_contiguous");
        verify_pool::<SegmentedPools>("segmented_pools");
        verify_pool::<OneTypeAtTime>("one_type_at_time");
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
        let initial_data = obj.data;

        unsafe {
            (obj.vtable.process)(&mut obj as *mut ObjectA as *mut u8);
        }

        assert_ne!(obj.data, initial_data);
    }
}
