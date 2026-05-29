use core::{num::NonZeroUsize, ptr::NonNull};

use crate::mem::{Assert, InputType, IsTrue, Sqlite3Memory, Sqlite3MemorySlice};
use windows_sys::Win32::System::Memory::{
    VirtualAlloc,
    VirtualFree,
    MEM_COMMIT,
    MEM_RELEASE,
    MEM_RESERVE,
    PAGE_READWRITE,
};

pub struct WindowsVirtualMemoryAllocator;
impl WindowsVirtualMemoryAllocator {
    pub const fn new() -> Self {Self{}}
}



// pub struct WindowsVirtualMemoryAllocator;

impl WindowsVirtualMemoryAllocator {
    /// Allocates the mathematically exact number of 32-byte SIMD blocks
    /// required to store `element_count` elements of `T`.
    pub unsafe fn allocate<'a, T>(
        &'a self,
        element_count: NonZeroUsize,
    ) -> Result<Sqlite3MemorySlice<'a, T>, super::AllocError>
    where
        T: InputType,
        Assert<{ size_of::<T>() > 0 }>: IsTrue,
        Assert<{ 32 % size_of::<T>() == 0 }>: IsTrue,
        Assert<{ align_of::<T>() > 0 }>: IsTrue,
    {
        const SIMD_BLOCK_SIZE: usize = 32;

        let element_size: NonZeroUsize = <T as InputType>::size_of_nonzero();

        // total bytes requested by caller
        let total_bytes =
            element_count
                .get()
                .checked_mul(element_size.get())
                .ok_or(super::AllocError)?;

        // exact number of 32-byte SIMD blocks required
        let block_count =
            total_bytes
                .div_ceil(SIMD_BLOCK_SIZE);

        let allocation_size =
            block_count
                .checked_mul(SIMD_BLOCK_SIZE)
                .ok_or(super::AllocError)?;

        let ptr = VirtualAlloc(
            core::ptr::null_mut(),
            allocation_size,
            MEM_RESERVE | MEM_COMMIT,
            PAGE_READWRITE,
        );

        if ptr.is_null() {
            return Err(super::AllocError);
        }

        Ok(Sqlite3MemorySlice {
            ptr: NonNull::new_unchecked(
                ptr.cast::<Sqlite3Memory<T>>()
            ),
            len: NonZeroUsize::new_unchecked(block_count),
            _marker: core::marker::PhantomData,
        })
    }

    pub unsafe fn deallocate<T>(
        &self,
        slice: &mut Sqlite3MemorySlice<T>,
    ) -> Result<(), super::AllocError>
    where
        T: InputType,
        Assert<{ size_of::<T>() > 0 }>: IsTrue,
        Assert<{ 32 % size_of::<T>() == 0 }>: IsTrue,
        Assert<{ align_of::<T>() > 0 }>: IsTrue,
    {
        let result = VirtualFree(
            slice.ptr.as_ptr().cast(),
            0,
            MEM_RELEASE,
        );

        if result == 0 {
            return Err(super::AllocError);
        }

        Ok(())
    }
}

// Sqlite3Allocator















// impl super::Sqlite3Allocator for WindowsVirtualMemoryAllocator {
//     unsafe fn allocate<InputType:Sized>(
//         &self,
//         element_count:NonZeroUsize,
//     ) -> Result<NonNull<Sqlite3Memory>, super::AllocError>
//     {
//         todo!()
//     }
//         unsafe fn allocate_zeroed<InputType:Sized>(
//         &self,
//         element_count:NonZeroUsize,
//     ) -> Result<NonNull<Sqlite3Memory>, super::AllocError>
//     {
//         todo!()
//     }
//     unsafe fn deallocate<InputType>(
//         &self,
//         ptr: NonNull<Sqlite3Memory>,
//     ) -> Result<(),super::AllocError>
//     {
//         todo!()
//     }
//     unsafe fn grow<InputType>(
//         &self,
//         ptr: *mut super::Sqlite3Memory,
//         old_blocks: usize,
//         new_blocks: usize,
//     ) -> Result<*mut super::Sqlite3Memory, super::AllocError>
//     {
//         todo!()    
//     }
//     unsafe fn shrink(
//         &self,
//         ptr: *mut super::Sqlite3Memory,
//         old_blocks: usize,
//         new_blocks: usize,
//     ) -> Result<*mut super::Sqlite3Memory, super::AllocError>
//     {
//         todo!()
//     }

// }