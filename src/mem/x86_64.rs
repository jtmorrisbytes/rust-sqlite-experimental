pub mod prelude {
    pub use super::Sqlite3Memory;
}
use core::arch::x86_64::*;
use std::{
    alloc::{Layout, alloc_zeroed, dealloc, handle_alloc_error, realloc},
    fmt::{Binary, Debug, Display},
    intrinsics::const_allocate,
    num::NonZeroUsize,
    ops::{Index, IndexMut},
    ptr::NonNull,
    u32,
};
/// @Copy Jordan Morris
/// sqlite3 memory.rs
/// because the database needed this :)

/// the single unit of memory in the sqlite '3r' engine.
/// all above memory operations are required to work with this
/// and understand it.

/// it may not make sense, but doing it this way
/// enforces alignment and perfomance guarantees not available
/// with random memory allocations
///
/// the compiler intrinsic type was chosen to give llvm
/// the best chance of treating this type as the register
/// when the compiler can detect that the type was sourced from
/// memory or stack as well as coordinate memory loads
///
/// this also means that avx2 is now a hard requirement
/// for x86_64. NO RUNTIME CHECKS will occur.
/// YOU HAVE BEEN WARNED
#[repr(transparent)]
#[derive(Clone)]
pub struct Sqlite3Memory(__m256i);

// this compile time invariant ensures that Sqlite3Memory can NEVER be a zero sized type
// this allows for some crazy NonZero optimizations
// --- The Compile-Time Size Assertion Guard ---
const _: () = {
    // Explicitly enforce that it matches your architecture's 32-byte target requirement
    let _assert_is_32_bytes = [(); 0 - (std::mem::size_of::<Sqlite3Memory>() != 32) as usize];
};

// This evaluates entirely at compile time.
// If the expression is false, the array size becomes -1 (negative),
// which triggers an immediate, un-passable compilation error.
// --- Core Physical Layout Invariant Guarantees ---
const _: () = {
    let _assert_not_zst = [(); 0 - (std::mem::size_of::<Sqlite3Memory>() == 0) as usize];
    // 1. Enforce that size is exactly 32 bytes (Never Zero, matches __m256i / v128x2)
    let _assert_size_is_32 = [(); 0 - (std::mem::size_of::<Sqlite3Memory>() != 32) as usize];

    const ALIGN_OF_REGISTER: usize = std::mem::align_of::<Sqlite3Memory>();
    // 2. Enforce that alignment is exactly 32 bytes (Never Zero, matches CPU Cache line splits)
    let _assert_align_is_32 = [(); 0 - (ALIGN_OF_REGISTER != 32) as usize];

    // 3. Mathematical Proof of Size-Overflow Bounds
    // We must prove that no matter how many elements we ask for up to the machine's
    // theoretical maximum allocation limits, our rounding math will never wrap around memory.
    // Since we round to a multiple of 32, the worst-case maximum valid index is bounded by isize::MAX.
    const max_allocatable_bytes: usize = isize::MAX as usize;
    const size_of_register: usize = std::mem::size_of::<Sqlite3Memory>();
    // Prove that rounding a value right at the cliff edge of isize::MAX doesn't overflow
    const worst_case_bytes: usize =
        max_allocatable_bytes - (max_allocatable_bytes % size_of_register);
    let _assert_math_bounds_never_overflow_isize =
        [(); 0 - (worst_case_bytes > max_allocatable_bytes) as usize];

    //4: layouts power of two invariant
    const IS_NOT_POWER_OF_TWO: bool = (ALIGN_OF_REGISTER & (ALIGN_OF_REGISTER - 1)) != 0;
    let _assert_align_is_power_of_two = [(); 0 - IS_NOT_POWER_OF_TWO as usize];
};

impl Sqlite3Memory {
    // the caller must ensure that T points to at least Size_of Self, and aligned 32!
    pub fn load_from_u8_aligned_array_ref(t: &[u8; 32]) -> Self {
        Self(unsafe { _mm256_load_si256(t.as_ptr().cast()) })
    }
    // the above compile time invariants prove that this can never be zero
    /// returns the alignment of this type. note that the alignment of this type can
    /// never be zero and is proven at compile time, so this will never cause UB
    pub const fn nonzero_align_of() -> NonZeroUsize {
        let val = std::mem::align_of::<Self>();
        unsafe { NonZeroUsize::new_unchecked(val) }
    }
    // the above compile time invariants prove that this can never be zero
    /// returns the size of this type. note that the size of this type can
    /// never be zero and is proven at compile time, so this will never cause UB
    pub const fn nonzero_size_of() -> NonZeroUsize {
        let val = std::mem::size_of::<Self>();
        unsafe { NonZeroUsize::new_unchecked(val) }
    }
    // the above compile time invariants prove that this unchecked layout upholds
    // layouts internal invairants and as such can never cause ub, making
    // the layout of this type completely known at compile time (on top of its own internal layout)
    pub const fn layout() -> std::alloc::Layout {
        let size = Self::nonzero_size_of();
        let align_of = Self::nonzero_align_of();
        unsafe { std::alloc::Layout::from_size_align_unchecked(size.get(), align_of.get()) }
    }
    // the above compile time invariants prove that this can never be zero
}

/// here the default implementation of partialeq
/// assumes that you are operating bytewise (u8),
/// as such it perfomrs mm256_cmpeq_epi8.
/// if this is not the desired implementation,
/// make sure you aslo check the other methods on this type
impl PartialEq for Sqlite3Memory {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            let eq = _mm256_cmpeq_epi8(self.0, other.0);
            _mm256_testc_si256(eq, _mm256_set1_epi8(-1)) == 0
        }
    }
}
