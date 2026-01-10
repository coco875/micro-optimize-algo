//! CPU affinity wrapper for cross-platform thread pinning.
//!
//! This module provides a unified API for pinning threads to CPU cores,
//! with proper support for unpinning (restoring original affinity).
//!
//! Implemented manually using platform-specific APIs (libc on Linux/macOS,
//! windows-sys on Windows) for full control and minimal dependencies.

// ============================================================================
// Linux implementation using libc
// ============================================================================

#[cfg(target_os = "linux")]
mod platform {
    use std::cell::RefCell;

    thread_local! {
        static ORIGINAL_AFFINITY: RefCell<Option<libc::cpu_set_t>> = const { RefCell::new(None) };
    }

    /// Get all available CPU core IDs
    pub fn get_core_ids() -> Option<Vec<usize>> {
        unsafe {
            let num_cpus = libc::sysconf(libc::_SC_NPROCESSORS_ONLN);
            if num_cpus <= 0 {
                return None;
            }
            Some((0..num_cpus as usize).collect())
        }
    }

    /// Get the current CPU core the thread is running on
    pub fn get_current_cpu() -> Option<usize> {
        unsafe {
            let cpu = libc::sched_getcpu();
            if cpu >= 0 {
                Some(cpu as usize)
            } else {
                None
            }
        }
    }

    /// Save the current CPU affinity mask
    pub fn save_affinity() -> bool {
        unsafe {
            let mut set: libc::cpu_set_t = std::mem::zeroed();
            if libc::sched_getaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &mut set) == 0 {
                ORIGINAL_AFFINITY.with(|cell| {
                    *cell.borrow_mut() = Some(set);
                });
                true
            } else {
                false
            }
        }
    }

    /// Pin to a specific core
    pub fn set_affinity(core_id: usize) -> bool {
        unsafe {
            let mut set: libc::cpu_set_t = std::mem::zeroed();
            libc::CPU_ZERO(&mut set);
            libc::CPU_SET(core_id, &mut set);
            libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &set) == 0
        }
    }

    /// Restore the original CPU affinity (unpin)
    pub fn restore_affinity() -> bool {
        unsafe {
            ORIGINAL_AFFINITY.with(|cell| {
                if let Some(set) = cell.borrow_mut().take() {
                    libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &set) == 0
                } else {
                    false
                }
            })
        }
    }
}

// ============================================================================
// macOS implementation using libc (thread affinity hints)
// ============================================================================

#[cfg(target_os = "macos")]
mod platform {
    use std::cell::RefCell;

    // macOS doesn't have true CPU affinity, only affinity hints via thread_policy
    // We'll use a simple flag to track if we were "pinned"
    thread_local! {
        static WAS_PINNED: RefCell<bool> = const { RefCell::new(false) };
    }

    pub fn get_core_ids() -> Option<Vec<usize>> {
        unsafe {
            let num_cpus = libc::sysconf(libc::_SC_NPROCESSORS_ONLN);
            if num_cpus <= 0 {
                return None;
            }
            Some((0..num_cpus as usize).collect())
        }
    }

    pub fn get_current_cpu() -> Option<usize> {
        // Not available on macOS without private APIs
        None
    }

    pub fn save_affinity() -> bool {
        WAS_PINNED.with(|cell| {
            *cell.borrow_mut() = false;
        });
        true
    }

    pub fn set_affinity(_core_id: usize) -> bool {
        // macOS doesn't support true CPU affinity
        // We could use thread_affinity_policy_data_t but it's just a hint
        WAS_PINNED.with(|cell| {
            *cell.borrow_mut() = true;
        });
        false // Return false to indicate it's not really pinned
    }

    pub fn restore_affinity() -> bool {
        WAS_PINNED.with(|cell| {
            *cell.borrow_mut() = false;
        });
        true
    }
}

// ============================================================================
// Windows implementation
// ============================================================================

#[cfg(target_os = "windows")]
mod platform {
    use std::cell::RefCell;

    // Windows API types
    type HANDLE = *mut std::ffi::c_void;
    type DWORD = u32;
    type DWORD_PTR = usize;
    type BOOL = i32;

    extern "system" {
        fn GetCurrentThread() -> HANDLE;
        fn SetThreadAffinityMask(hThread: HANDLE, dwThreadAffinityMask: DWORD_PTR) -> DWORD_PTR;
        fn GetSystemInfo(lpSystemInfo: *mut SYSTEM_INFO);
    }

    #[repr(C)]
    struct SYSTEM_INFO {
        wProcessorArchitecture: u16,
        wReserved: u16,
        dwPageSize: DWORD,
        lpMinimumApplicationAddress: *mut std::ffi::c_void,
        lpMaximumApplicationAddress: *mut std::ffi::c_void,
        dwActiveProcessorMask: DWORD_PTR,
        dwNumberOfProcessors: DWORD,
        dwProcessorType: DWORD,
        dwAllocationGranularity: DWORD,
        wProcessorLevel: u16,
        wProcessorRevision: u16,
    }

    thread_local! {
        static ORIGINAL_MASK: RefCell<Option<DWORD_PTR>> = const { RefCell::new(None) };
    }

    pub fn get_core_ids() -> Option<Vec<usize>> {
        unsafe {
            let mut info: SYSTEM_INFO = std::mem::zeroed();
            GetSystemInfo(&mut info);
            let num_cpus = info.dwNumberOfProcessors as usize;
            if num_cpus == 0 {
                return None;
            }
            Some((0..num_cpus).collect())
        }
    }

    pub fn get_current_cpu() -> Option<usize> {
        // GetCurrentProcessorNumber() would be needed
        None
    }

    pub fn save_affinity() -> bool {
        unsafe {
            let handle = GetCurrentThread();
            // Get current mask by setting to all cores then reading the return value
            let mut info: SYSTEM_INFO = std::mem::zeroed();
            GetSystemInfo(&mut info);
            let all_cores = info.dwActiveProcessorMask;

            let old_mask = SetThreadAffinityMask(handle, all_cores);
            if old_mask != 0 {
                // Restore immediately to get the value
                SetThreadAffinityMask(handle, old_mask);
                ORIGINAL_MASK.with(|cell| {
                    *cell.borrow_mut() = Some(old_mask);
                });
                true
            } else {
                false
            }
        }
    }

    pub fn set_affinity(core_id: usize) -> bool {
        unsafe {
            let handle = GetCurrentThread();
            let mask: DWORD_PTR = 1 << core_id;
            SetThreadAffinityMask(handle, mask) != 0
        }
    }

    pub fn restore_affinity() -> bool {
        unsafe {
            ORIGINAL_MASK.with(|cell| {
                if let Some(mask) = cell.borrow_mut().take() {
                    let handle = GetCurrentThread();
                    SetThreadAffinityMask(handle, mask) != 0
                } else {
                    false
                }
            })
        }
    }
}

// ============================================================================
// Fallback for unsupported platforms
// ============================================================================

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
mod platform {
    pub fn get_core_ids() -> Option<Vec<usize>> {
        None
    }
    pub fn get_current_cpu() -> Option<usize> {
        None
    }
    pub fn save_affinity() -> bool {
        true
    }
    pub fn set_affinity(_core_id: usize) -> bool {
        false
    }
    pub fn restore_affinity() -> bool {
        true
    }
}

// ============================================================================
// Public API
// ============================================================================

/// Get all available CPU core IDs
pub fn get_core_ids() -> Option<Vec<usize>> {
    platform::get_core_ids()
}

/// Get the current CPU core the thread is running on
pub fn get_current_cpu() -> Option<usize> {
    platform::get_current_cpu()
}

/// Pin the current thread to a specific core.
///
/// Saves the current affinity before pinning so it can be restored later.
///
/// # Returns
/// `true` if pinning was successful
pub fn pin_to_core(core_id: usize) -> bool {
    platform::save_affinity();
    platform::set_affinity(core_id)
}

/// Pin the current thread to the first available core.
///
/// # Returns
/// The core ID that was pinned to, or `None` if pinning failed.
pub fn pin_to_first_core() -> Option<usize> {
    let cores = get_core_ids()?;
    let core = *cores.first()?;
    if pin_to_core(core) {
        Some(core)
    } else {
        None
    }
}

/// Pin the current thread to the core it's currently running on.
///
/// This is ideal for timing measurements as it prevents migration
/// without forcing a specific core.
///
/// # Returns
/// The core ID that was pinned to, or `None` if pinning failed.
pub fn pin_to_current_core() -> Option<usize> {
    if let Some(current) = platform::get_current_cpu() {
        if pin_to_core(current) {
            return Some(current);
        }
    }
    // Fallback to first available core
    pin_to_first_core()
}

/// Unpin the current thread, restoring its original CPU affinity.
///
/// This allows the OS scheduler to freely migrate the thread between cores.
///
/// # Returns
/// `true` if unpinning was successful
pub fn unpin() -> bool {
    platform::restore_affinity()
}

// ============================================================================
// RAII Guard
// ============================================================================

/// RAII guard for CPU pinning - pins on creation, unpins on drop.
///
/// This ensures the thread is always unpinned when the guard goes out of scope,
/// even if the code panics.
///
/// # Example
/// ```ignore
/// {
///     let _pin = CpuPinGuard::new(); // Thread pinned
///     // ... do timing measurements ...
/// } // Thread automatically unpinned here
/// ```
pub struct CpuPinGuard {
    pinned_core: Option<usize>,
}

impl CpuPinGuard {
    /// Create a new guard that pins to the current CPU core.
    pub fn new() -> Self {
        Self {
            pinned_core: pin_to_current_core(),
        }
    }

    /// Create a new guard that pins to a specific core.
    pub fn with_core(core_id: usize) -> Self {
        platform::save_affinity();
        let success = platform::set_affinity(core_id);
        Self {
            pinned_core: if success { Some(core_id) } else { None },
        }
    }

    /// Create a new guard that pins to the first available core.
    pub fn first_core() -> Self {
        Self {
            pinned_core: pin_to_first_core(),
        }
    }

    /// Get the core ID this thread is pinned to, if any.
    pub fn core_id(&self) -> Option<usize> {
        self.pinned_core
    }

    /// Check if the thread was successfully pinned.
    pub fn is_pinned(&self) -> bool {
        self.pinned_core.is_some()
    }
}

impl Drop for CpuPinGuard {
    fn drop(&mut self) {
        if self.pinned_core.is_some() {
            unpin();
        }
    }
}

impl Default for CpuPinGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_core_ids() {
        let cores = get_core_ids();
        assert!(cores.is_some(), "Should be able to get core IDs");
        assert!(!cores.unwrap().is_empty(), "Should have at least one core");
    }

    #[test]
    fn test_pin_guard() {
        let guard = CpuPinGuard::new();
        // On most systems, pinning should succeed
        if guard.is_pinned() {
            assert!(guard.core_id().is_some());
        }
        drop(guard);
        // Thread should be unpinned now
    }

    #[test]
    fn test_pin_unpin_cycle() {
        let core = pin_to_first_core();
        if core.is_some() {
            assert!(unpin(), "Unpin should succeed after pin");
        }
    }
}
