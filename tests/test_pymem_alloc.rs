#![cfg(any(Py_3_13, not(Py_LIMITED_API)))]

use pyo3::pymem_alloc::PyMemRawAllocator;

#[global_allocator]
static GLOBAL_ALLOCATOR: PyMemRawAllocator = PyMemRawAllocator;

#[test]
fn allocations_work_through_global_allocator() {
    let mut vec: Vec<u8> = Vec::with_capacity(1024);
    vec.extend_from_slice(&[1, 2, 3, 4]);
    assert_eq!(vec, vec![1, 2, 3, 4]);

    let s = String::from("hello");
    assert_eq!(s.len(), 5);

    #[repr(align(64))]
    struct Aligned([u8; 128]);
    let boxed = Box::new(Aligned([7_u8; 128]));
    assert_eq!(boxed.0[0], 7);

    // TODO: use `.addr()` directly on MSRV 1.84 (`ptr_addr(ptr)` -> `ptr.addr()`)
    #[inline(always)]
    fn ptr_addr<T>(ptr: *const T) -> usize {
        // SAFETY: Pointer-to-integer transmutes are valid.
        unsafe { core::mem::transmute(ptr.cast::<()>()) }
    }

    assert_eq!(ptr_addr(&*boxed as *const Aligned) % 64, 0);
}
