use core::{marker::PhantomData, num::NonZeroUsize, ptr::NonNull};

#[cfg(target_arch = "aarch64")]
pub mod aarch64;
#[allow(non_camel_case_types)]
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_os = "windows")]
pub mod windows;

// sqlite3_memory.rs

// I decided to abandon core alloc layout because it discards all type information

pub struct Assert<const CHECK: bool>;

pub trait IsTrue {}

impl IsTrue for Assert<true> {}

pub unsafe trait InputType: Sized
where
    Assert<{ core::mem::size_of::<Self>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<Self>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<Self>() > 0 }>: IsTrue,
{
    fn size_of_nonzero() -> NonZeroUsize {
        unsafe { NonZeroUsize::new_unchecked(core::mem::size_of::<Self>()) }
    }
    fn algin_of_nonzero() -> NonZeroUsize {
        unsafe { NonZeroUsize::new_unchecked(core::mem::size_of::<Self>()) }
    }
}

//
// Fundamental invariant:
//
// Every Sqlite3Memory value is EXACTLY 32 bytes.
//
// The backend representation changes per target,
// but the semantic size stays fixed.
//

// ============================================================
// x86/x86_64 AVX2
// ============================================================

// sqlite3Memory stores as vector type, but also keeps its associated type
// so we can specialize on it
#[derive(Debug)]
pub struct Sqlite3Memory<T>(core::arch::x86_64::__m256i, core::marker::PhantomData<T>)
where
    T: InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue;
#[derive(Debug)]
pub struct Sqlite3MemorySlice<'a, T>
where
    T: InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    ptr: NonNull<Sqlite3Memory<T>>,
    len: NonZeroUsize,
    _marker: PhantomData<&'a mut Sqlite3Memory<T>>,
}
impl<T> Sqlite3MemorySlice<'_, T>
where
    T: InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    pub fn as_ptr(&self) -> NonNull<Sqlite3Memory<T>> {
        self.ptr
    }
    pub fn as_mut_ptr(&mut self) -> NonNull<Sqlite3Memory<T>>{
        self.ptr
    }
    pub fn len(&self) -> NonZeroUsize {
        self.len
    }
}

// iterator
pub struct Sqlite3MemoryIter<'a, T>
where
    T: InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    ptr: *const T,
    remaining: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Iterator for Sqlite3MemoryIter<'a, T>
where
    T: InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    type Item = &'a T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        unsafe {
            let current = self.ptr;

            self.ptr = self.ptr.add(1);
            self.remaining -= 1;

            Some(&*current)
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<'a, T> Sqlite3MemorySlice<'a, T>
where
    T: InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    #[inline(always)]
    pub fn iter(&'a self) -> Sqlite3MemoryIter<'a, T> {
        let elements_per_block = 32 / core::mem::size_of::<T>();

        let total_elements = self.len.get() * elements_per_block;

        Sqlite3MemoryIter {
            ptr: self.ptr.as_ptr().cast::<T>(),
            remaining: total_elements,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> ExactSizeIterator for Sqlite3MemoryIter<'a, T>
where
    T: InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
}

impl<T> Sqlite3MemorySlice<'_, T>
where
    T: InputType + Default,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    pub fn fill_default(&mut self) {
        let elements_per_block =
            Sqlite3Memory::<T>::nonzero_size_of().get() / <T as InputType>::size_of_nonzero();

        unsafe {
            let ptr = self.ptr.as_ptr().cast::<T>();

            let total_elements = self.len.get() * elements_per_block;

            for index in 0..total_elements {
                ptr.add(index).write(T::default());
            }
        }

        todo!()
    }
}

// ============================================================
// Compile-time assertions
// ============================================================
//  these assert at the type level and trait level that you cannot
// physically construct an instance of sqlite3memory unless the unerlying
// representation passes these checks
//
// because T: Input Type, and input type has these assertions,
// you cannot physically place a T inside this collection unless it is well formed

unsafe impl InputType for u8 {}
unsafe impl InputType for [u8; 32] {}
unsafe impl InputType for u16 {}
unsafe impl InputType for [u16; 16] {}
unsafe impl InputType for u32 {}
unsafe impl InputType for [u32; 8] {}
unsafe impl InputType for u64 {}
unsafe impl InputType for [u64; 4] {}
unsafe impl InputType for u128 {}
unsafe impl InputType for [u128; 2] {}

unsafe impl InputType for i8 {}
unsafe impl InputType for [i8; 32] {}

unsafe impl InputType for i16 {}
unsafe impl InputType for [i16; 16] {}
unsafe impl InputType for i32 {}
unsafe impl InputType for [i32; 8] {}
unsafe impl InputType for i64 {}
unsafe impl InputType for [i64; 4] {}
unsafe impl InputType for i128 {}
unsafe impl InputType for [i128; 2] {}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2",
))]
#[repr(transparent)]
#[derive(Clone)]

pub struct Sqlite3MemoryExactSizedSlice<'a, T, const N: usize>
where
    T: InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
    Assert<{ N > 0 }>: IsTrue,
{
    ptr: NonNull<Sqlite3Memory<T>>,
    _marker: PhantomData<&'a mut Sqlite3Memory<T>>,
}

// ============================================================
// x86/x86_64 SSE2 fallback
// 2 x 128-bit vectors = 32 bytes
// ============================================================

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    not(target_feature = "avx2"),
    target_feature = "sse2",
))]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Sqlite3Memory<LaneType>(
    pub core::arch::x86_64::__m128i,
    pub core::arch::x86_64::__m128i,
    core::marker::PhantomData<T>,
);

// ============================================================
// wasm32 SIMD128
// 2 x v128 = 32 bytes
// ============================================================

#[cfg(all(target_arch = "wasm32", target_feature = "simd128",))]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Sqlite3Memory(pub core::arch::wasm32::v128, pub core::arch::wasm32::v128);

// ============================================================
// AArch64 NEON
// 2 x 128-bit vectors = 32 bytes
// ============================================================

#[cfg(target_arch = "aarch64")]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Sqlite3Memory(
    pub core::arch::aarch64::uint8x16_t,
    pub core::arch::aarch64::uint8x16_t,
);

// ============================================================
// Portable fallback
// Preserves:
// - exact size
// - exact alignment
//
// Does NOT preserve LLVM vector semantics.
// ============================================================

#[cfg(not(any(
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
    ),
    all(target_arch = "wasm32", target_feature = "simd128",),
    target_arch = "aarch64",
)))]
#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct Sqlite3Memory(pub [u8; 32]);

impl<T> Sqlite3Memory<T>
where
    T: InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    // the above compile time invariants prove that this can never be zero
    /// returns the alignment of this type. note that the alignment of this type can
    /// never be zero and is proven at compile time, so this will never cause UB
    pub const fn nonzero_align_of() -> NonZeroUsize {
        let val = core::mem::align_of::<Self>();
        unsafe { NonZeroUsize::new_unchecked(val) }
    }
    // the above compile time invariants prove that this can never be zero
    /// returns the size of this type. note that the size of this type can
    /// never be zero and is proven at compile time, so this will never cause UB
    pub const fn nonzero_size_of() -> NonZeroUsize {
        let val = core::mem::size_of::<Self>();
        unsafe { NonZeroUsize::new_unchecked(val) }
    }
    // the above compile time invariants prove that this unchecked layout upholds
    // layouts internal invairants and as such can never cause ub, making
    // the layout of this type completely known at compile time (on top of its own internal layout)
    pub const fn layout() -> core::alloc::Layout {
        let size = Self::nonzero_size_of();
        let align_of = Self::nonzero_align_of();
        unsafe { core::alloc::Layout::from_size_align_unchecked(size.get(), align_of.get()) }
    }
    // the above compile time invariants prove that this can never be zero
}

// ============================================================
// Allocation error
// ============================================================

#[derive(Copy, Clone, Debug)]
pub struct AllocError;

// ============================================================
// SIMD-native allocator API
// ============================================================

pub trait Sqlite3Allocator {
    unsafe fn allocate<'a, T: InputType>(
        &'a self,
        element_count: NonZeroUsize,
    ) -> Result<Sqlite3MemorySlice<'a, T>, AllocError>
    where
        T: InputType,
        Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
        Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
        Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue;

    unsafe fn allocate_exact_sized<'a, T: InputType, const N: usize>()
    -> Result<Sqlite3MemoryExactSizedSlice<'a, T, N>, AllocError>
    where
        T: InputType,
        Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
        Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
        Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
        Assert<{ N > 0 }>: IsTrue;

    // there is no zeroed because zeroed is the default
    // unsafe fn allocate_zeroed<'a, T: InputType>(
    //     &'a self,
    //     element_count: NonZeroUsize,
    // ) -> Result<&'a mut [Sqlite3Memory<T>], AllocError>;

    unsafe fn deallocate<'a, T>(&'a self, ptr: Sqlite3MemorySlice<'a, T>) -> Result<(), AllocError>
    where
        T: InputType,
        Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
        Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
        Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue;
    unsafe fn deallocate_exact_sized<'a, T: InputType, const N: usize>(
        ptr: Sqlite3MemoryExactSizedSlice<'a, T, N>,
    ) -> Result<(), AllocError>
    where
        T: InputType,
        Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
        Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
        Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
        Assert<{ N > 0 }>: IsTrue;

    unsafe fn grow<'a, T: InputType>(
        &'a self,
        ptr: Sqlite3MemorySlice<'a, T>,
        element_count: NonZeroUsize,
    ) -> Result<Sqlite3MemorySlice<'a, T>, AllocError>
    where
        Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
        Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
        Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue;

    unsafe fn shrink<'a, T: InputType>(
        &self,
        ptr: &'a mut [Sqlite3Memory<T>],
        new_len: NonZeroUsize,
    ) -> Result<(), AllocError>
    where
        Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
        Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
        Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue;
}

#[cfg(target_os = "windows")]
static SQLITE3_ALLOCATOR: windows::WindowsVirtualMemoryAllocator =
    windows::WindowsVirtualMemoryAllocator::new();

pub fn alloc<'a, T: InputType>(
    element_count: NonZeroUsize,
) -> Result<Sqlite3MemorySlice<'a, T>, AllocError>
where
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    unsafe { SQLITE3_ALLOCATOR.allocate::<T>(element_count) }
}
// // pub fn alloc_zeroed<T:InputType>(
// //     element_count: NonZeroUsize,
// // ) -> Result<*mut Sqlite3Memory, AllocError> {
// //     unsafe { SQLITE3_ALLOCATOR.allocate::<InputType>(element_count) }
// // }
pub fn dealloc<'a, T>(
    ptr: &mut Sqlite3MemorySlice<'_, T>,
    // layout: core::alloc::Layout,
) -> Result<(), AllocError>
where
    T: InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    unsafe { SQLITE3_ALLOCATOR.deallocate::<T>(ptr) }
}
