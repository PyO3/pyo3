use crate::pyport::Py_ssize_t;
use core::sync::atomic::{AtomicIsize, Ordering};

// skipped _Py_atomic_add_int
// skipped _Py_atomic_add_int8
// skipped _Py_atomic_add_int16
// skipped _Py_atomic_add_int32
// skipped _Py_atomic_add_int64
// skipped _Py_atomic_add_intptr
// skipped _Py_atomic_add_uint
// skipped _Py_atomic_add_uint8
// skipped _Py_atomic_add_uint16
// skipped _Py_atomic_add_uint32
// skipped _Py_atomic_add_uint64
// skipped _Py_atomic_add_uintptr
// skipped _Py_atomic_add_ssize

// skipped _Py_atomic_compare_exchange_int
// skipped _Py_atomic_compare_exchange_int8
// skipped _Py_atomic_compare_exchange_int16
// skipped _Py_atomic_compare_exchange_int32
// skipped _Py_atomic_compare_exchange_int64
// skipped _Py_atomic_compare_exchange_intptr
// skipped _Py_atomic_compare_exchange_uint
// skipped _Py_atomic_compare_exchange_uint8
// skipped _Py_atomic_compare_exchange_uint16
// skipped _Py_atomic_compare_exchange_uint32
// skipped _Py_atomic_compare_exchange_uint64
// skipped _Py_atomic_compare_exchange_uintptr
// skipped _Py_atomic_compare_exchange_ssize
// skipped _Py_atomic_compare_exchange_ptr

// skipped _Py_atomic_exchange_int
// skipped _Py_atomic_exchange_int8
// skipped _Py_atomic_exchange_int16
// skipped _Py_atomic_exchange_int32
// skipped _Py_atomic_exchange_int64
// skipped _Py_atomic_exchange_intptr
// skipped _Py_atomic_exchange_uint
// skipped _Py_atomic_exchange_uint8
// skipped _Py_atomic_exchange_uint16
// skipped _Py_atomic_exchange_uint32
// skipped _Py_atomic_exchange_uint64
// skipped _Py_atomic_exchange_uintptr
// skipped _Py_atomic_exchange_ssize
// skipped _Py_atomic_exchange_ptr

// skipped _Py_atomic_and_uint8
// skipped _Py_atomic_and_uint16
// skipped _Py_atomic_and_uint32
// skipped _Py_atomic_and_uint64
// skipped _Py_atomic_and_uintptr

// skipped _Py_atomic_or_uint8
// skipped _Py_atomic_or_uint16
// skipped _Py_atomic_or_uint32
// skipped _Py_atomic_or_uint64
// skipped _Py_atomic_or_uintptr

// skipped _Py_atomic_load_int
// skipped _Py_atomic_load_int8
// skipped _Py_atomic_load_int16
// skipped _Py_atomic_load_int32
// skipped _Py_atomic_load_int64
// skipped _Py_atomic_load_intptr
// skipped _Py_atomic_load_uint8
// skipped _Py_atomic_load_uint16
// skipped _Py_atomic_load_uint32
// skipped _Py_atomic_load_uint64
// skipped _Py_atomic_load_uintptr
// skipped _Py_atomic_load_uint
// skipped _Py_atomic_load_ssize
// skipped _Py_atomic_load_ptr

// skipped _Py_atomic_load_int_relaxed
// skipped _Py_atomic_load_char_relaxed
// skipped _Py_atomic_load_uchar_relaxed
// skipped _Py_atomic_load_short_relaxed
// skipped _Py_atomic_load_ushort_relaxed
// skipped _Py_atomic_load_long_relaxed
// skipped _Py_atomic_load_double_relaxed
// skipped _Py_atomic_load_llong_relaxed
// skipped _Py_atomic_load_int8_relaxed
// skipped _Py_atomic_load_int16_relaxed
// skipped _Py_atomic_load_int32_relaxed
// skipped _Py_atomic_load_int64_relaxed
// skipped _Py_atomic_load_intptr_relaxed
// skipped _Py_atomic_load_uint8_relaxed
// skipped _Py_atomic_load_uint16_relaxed
// skipped _Py_atomic_load_uint32_relaxed
// skipped _Py_atomic_load_uint64_relaxed
// skipped _Py_atomic_load_uintptr_relaxed
// skipped _Py_atomic_load_uint_relaxed

#[inline]
pub(crate) unsafe fn _Py_atomic_load_ssize_relaxed(value: *const Py_ssize_t) -> Py_ssize_t {
    AtomicIsize::from_ptr(value.cast_mut()).load(Ordering::Relaxed)
}

// skipped _Py_atomic_load_ptr_relaxed
// skipped _Py_atomic_load_ullong_relaxed

// skipped _Py_atomic_store_int
// skipped _Py_atomic_store_int8
// skipped _Py_atomic_store_int16
// skipped _Py_atomic_store_int32
// skipped _Py_atomic_store_int64
// skipped _Py_atomic_store_intptr
// skipped _Py_atomic_store_uint8
// skipped _Py_atomic_store_uint16
// skipped _Py_atomic_store_uint32
// skipped _Py_atomic_store_uint64
// skipped _Py_atomic_store_uintptr
// skipped _Py_atomic_store_uint
// skipped _Py_atomic_store_ptr
// skipped _Py_atomic_store_ssize

// skipped _Py_atomic_store_int_relaxed
// skipped _Py_atomic_store_int8_relaxed
// skipped _Py_atomic_store_int16_relaxed
// skipped _Py_atomic_store_int32_relaxed
// skipped _Py_atomic_store_int64_relaxed
// skipped _Py_atomic_store_intptr_relaxed
// skipped _Py_atomic_store_uint8_relaxed
// skipped _Py_atomic_store_uint16_relaxed
// skipped _Py_atomic_store_uint32_relaxed
// skipped _Py_atomic_store_uint64_relaxed
// skipped _Py_atomic_store_uintptr_relaxed
// skipped _Py_atomic_store_uint_relaxed
// skipped _Py_atomic_store_ptr_relaxed
// skipped _Py_atomic_store_ssize_relaxed
// skipped _Py_atomic_store_ullong_relaxed
// skipped _Py_atomic_store_char_relaxed
// skipped _Py_atomic_store_uchar_relaxed
// skipped _Py_atomic_store_short_relaxed
// skipped _Py_atomic_store_ushort_relaxed
// skipped _Py_atomic_store_long_relaxed
// skipped _Py_atomic_store_float_relaxed
// skipped _Py_atomic_store_double_relaxed
// skipped _Py_atomic_store_llong_relaxed

// skipped _Py_atomic_load_ptr_acquire
// skipped _Py_atomic_load_uintptr_acquire
// skipped _Py_atomic_store_ptr_release
// skipped _Py_atomic_store_uintptr_release
// skipped _Py_atomic_store_ssize_release
// skipped _Py_atomic_store_int8_release
// skipped _Py_atomic_store_int_release
// skipped _Py_atomic_load_int_acquire
// skipped _Py_atomic_store_uint_release
// skipped _Py_atomic_store_uint32_release
// skipped _Py_atomic_store_uint64_release
// skipped _Py_atomic_load_uint64_acquire
// skipped _Py_atomic_load_uint32_acquire
// skipped _Py_atomic_load_ssize_acquire

// skipped _Py_atomic_fence_seq_cst
// skipped _Py_atomic_fence_acquire
// skipped _Py_atomic_fence_release

// skipped _Py_atomic_load_ptr_consume
// skipped _Py_atomic_load_ulong
// skipped _Py_atomic_load_ulong_relaxed
// skipped _Py_atomic_store_ulong
// skipped _Py_atomic_store_ulong_relaxed
