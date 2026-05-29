use crate::mem::{Assert, InputType, IsTrue};
use core::arch::x86_64::*;
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
use super::Sqlite3Memory;

impl<T> Sqlite3Memory<T>
where
    T: InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    // the caller must ensure that T points to at least Size_of Self, and aligned 32!
    pub fn load_from_u8_aligned_array_ref(t: &[u8; 32]) -> Self {
        Self(
            unsafe { _mm256_load_si256(t.as_ptr().cast()) },
            core::marker::PhantomData {},
        )
    }
}

/// here the default implementation of partialeq
/// assumes that you are operating bytewise (u8),
/// as such it perfomrs mm256_cmpeq_epi8.
/// if this is not the desired implementation,
/// make sure you aslo check the other methods on this type

impl PartialEq for Sqlite3Memory<u8>
where
    u8: InputType,
    Assert<{ core::mem::size_of::<u8>() > 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<u8>() > 0 }>: IsTrue,
{
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            let eq = _mm256_cmpeq_epi8(self.0, other.0);
            _mm256_testc_si256(eq, _mm256_set1_epi8(-1)) == 0
        }
    }
}
impl PartialEq for Sqlite3Memory<i16>
where
    i16: InputType,
    Assert<{ core::mem::size_of::<i16>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<i16>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<i16>() > 0 }>: IsTrue,
{
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            let eq = _mm256_cmpeq_epi16(self.0, other.0);
            _mm256_testc_si256(eq, _mm256_set1_epi16(-1)) == 0
        }
    }
}

impl PartialEq for Sqlite3Memory<u16>
where
    u16: InputType,
    Assert<{ core::mem::size_of::<u16>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<u16>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<u16>() > 0 }>: IsTrue,
{
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            let eq = _mm256_cmpeq_epi8(self.0, other.0);
            _mm256_testc_si256(eq, _mm256_set1_epi8(-1)) == 0
        }
    }
}






impl PartialEq for Sqlite3Memory<i8>
where
    i8: InputType,
    Assert<{ core::mem::size_of::<i8>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<i8>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<i8>() > 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<i8>() > 0 }>: IsTrue,
{
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            let eq = _mm256_cmpeq_epi8(self.0, other.0);
            _mm256_testc_si256(eq, _mm256_set1_epi8(-1)) == 0
        }
    }
}

// impl<T> PartialEq for Sqlite3Memory<T> {
//     #[inline(always)]
//     fn eq(&self, other: &Self) -> bool {
//         unsafe {
//             let eq = _mm256_cmpeq_epi8(self.0, other.0);
//             _mm256_testc_si256(eq, _mm256_set1_epi8(-1)) == 0
//         }
//     }
// }
