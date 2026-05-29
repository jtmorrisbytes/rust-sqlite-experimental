use core::{
    alloc::Layout,
    fmt::{Binary, Display},
    num::NonZeroUsize,
    ops::{Index, IndexMut},
    ptr::NonNull,
};

use crate::mem::{Assert, InputType, IsTrue, Sqlite3Memory, Sqlite3MemorySlice};

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

pub struct Sqlite3Vec<'a, T>
where
    T: InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    ///stores the element count of T, regardless of the byte count
    // len: NonZeroUsize,
    /// stores the byte size, in multpiles of Sqlite3Memory
    /// regardless of the len of the elements
    // size: NonZeroUsize,
    ptr: Sqlite3MemorySlice<'a, T>,
}

impl<'a, T> Sqlite3Vec<'a, T>
where
    T: InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    // I decided to ban zsts because its kind of akward. you cant compare or reason about them
    // at the memory level without logical fallicies. for example () == () but has no memory representation.

    /// Returns the physical size of the inner type `T`.
    ///
    /// This method explicitly enforces at compile time that `T` cannot be a zero-sized type (ZST).
    /// Because ZSTs are banned, this operation is guaranteed to never cause UB or return zero.
    #[inline(always)]
    pub fn size_of_inner() -> NonZeroUsize {
        <T as InputType>::size_of_nonzero()
    }
}

impl<'a, T> Sqlite3Vec<'a, T>
where
    T: Sized + Default + InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    /// initalizes the collection with exactly one element, using the default implementation
    /// to guarentee that T is valid, after this call it is valid to do Self[0] ... whatever
    /// and push is not required
    pub fn new() -> Self {
        let len: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1) };
        // let layout = Self::layout(len);
        let mut raw_ptr = unsafe { crate::mem::alloc::<T>(len).unwrap() };
        // if raw_ptr.is_null() {
        //     panic!("Sqlite3Vec: Fatal Invaraint violation: Allocation failed");
        // }
        // unsafe { core::ptr::write_bytes(raw_ptr, 0, layout.size()) };
        // let default = T::default();
        // unsafe { raw_ptr.ptr().cast::<T>().write(default) };
        raw_ptr.fill_default();
        // let raw_ptr = unsafe {core::intrinsics::const_make_global(raw_ptr) as *mut u8};
        // let ptr = unsafe { raw_ptr.cast::<T>() };
        // let size = unsafe { NonZeroUsize::new_unchecked(1) };
        Self { ptr: raw_ptr }
    }
}
impl<'a, T> Sqlite3Vec<'a, T>
where
    T: Default + Clone + InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    pub fn with_initial_size(elements: NonZeroUsize, value: Option<T>) -> Self {
        // let layout = Self::layout(elements);
        let ptr = unsafe { crate::mem::alloc(elements).unwrap() };
        // after this call, layout.size should be a multiple of 32 and no math should be required
        // let size = unsafe { NonZeroUsize::new_unchecked(layout.size()) };
        // let ptr = unsafe { NonNull::new_unchecked(ptr).cast::<T>() };
        // then, after this we initialize N elements using T Default
        // let default:T = T::default();
        // for i in 0..elements.get() {
        //     unsafe { ptr.as_ptr().add(i).write(value.clone().unwrap_or_default()) };
        // }
        Self {
            ptr,
            // size,
            // len: elements,
        }
    }
    // creates an instance of this collection, but requires that
    // it always be initalized with at least one valid element,
    // even if zero or default
    pub fn from_slice<B: AsRef<[T]>>(b: &B) -> Result<Self, &'static str> {
        let bytes = b.as_ref();
        // let len = bytes.len();
        // let size_of_t_nonzero = Self::size_of_inner();
        if bytes.len() == 0 {
            return Err(
                "Sqlite3Vec:Invaraint Violation: This collection must always be initalized with at least one element",
            );
        }
        // let len = unsafe { NonZeroUsize::new_unchecked(bytes.len()) };
        // Allocate using our absolute 32-byte multiple boundary function
        // let layout = Self::layout(len);
        let mut ptr = unsafe { crate::mem::alloc::<T>(bytes.len().try_into().unwrap()).unwrap() };
        // if ptr.is_null() {
        //     core::alloc::handle_alloc_error(layout);
        // }
        unsafe {
            // Copy the exact elements preserving typed constraints
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.as_mut_ptr().cast::<T>().as_ptr(), ptr.len().get());
        }

        // Record 'size' as the absolute count of 32-byte blocks
        // let size = unsafe { NonZeroUsize::new_unchecked(layout.size()) };
        // let ptr = unsafe { NonNull::new_unchecked(ptr).cast() };

        Ok(Self { ptr })
    }
}
// impl<T> Sqlite3Vec<T>
// where
//     T: Sized,
// {
//     /// Constant evaluated Greatest Common Divisor loop for compile-time structural proofs
//     const fn const_gcd(a: NonZeroUsize, b: NonZeroUsize) -> NonZeroUsize {
//         let mut val_a = a.get();
//         let mut val_b = b.get();
//         while val_b != 0 {
//             let temp = val_b;
//             val_b = val_a % val_b;
//             val_a = temp;
//         }
//         #[cfg(debug_assertions)]
//         {
//             NonZeroUsize::new(val_a).expect(
//                 "Sqlite3Vec: FATAL INVARIANT VIOLATION during gcd calculation:\
//              lcm_bytes / size_of_t == 0. STRIDE CAN NEVER BE ZERO",
//             )
//         }
//         #[cfg(not(debug_assertions))]
//         {
//             unsafe { NonZeroUsize::new_unchecked(val_a) }
//         }
//     }
// }

// impl<T> Sqlite3Vec<T>
// where
//     T: Sized,
// {
//     // const ASSERT1:() {
//     //     if Self::size_of_inner().get() /
//     // }
//     // calculates how many elements of T will fit into Sqlite3Memory, otherwise gets the LCM
//     pub const fn stride() -> NonZeroUsize {
//         // because we ban zsts we can enforce that this cann never be null
//         let size_of_t: NonZeroUsize = Self::size_of_inner();
//         // the above compile time invariants prove that this can never be zero, and as such
//         // should never cause ub
//         let size_of_register: NonZeroUsize = Sqlite3Memory::nonzero_size_of();

//         // If the element size divides cleanly into 32 bytes (e.g., u8, u16, u32, u64, u128),
//         // the calculation is simple: how many items fit in a single 32-byte block.
//         if size_of_register.get() % size_of_t.get() == 0 {
//             return unsafe { NonZeroUsize::new_unchecked(size_of_register.get() / size_of_t) };
//         }
//         // For arbitrary or odd-sized structs (e.g., a custom 5-byte SQL record cell):
//         // 1. Find the Least Common Multiple (LCM) of bytes using Great Common Divisor (GCD)
//         let gcd = Self::const_gcd(size_of_t, size_of_register);

//         let mul = size_of_t.checked_mul(size_of_register).expect(
//             "Sqlite3Vec: FATAL INVARIANT VIOLATION: size_of T * size_of register overflowed",
//         );
//         let lcm_bytes = mul.get() / gcd;

//         if lcm_bytes == 0 {
//             panic!(
//                 "Sqlite3Vec: FATAL INVARIANT VIOLATION: stride calculation produced a byte count of zero"
//             );
//         }
//         let lcm_bytes = unsafe { NonZeroUsize::new_unchecked(lcm_bytes) };

//         let element_count = lcm_bytes.get() / size_of_t;
//         let element_count = NonZeroUsize::new(element_count).expect(
//             "Sqlite3Vec: FATAL INVARIANT VIOLATION during stride calculation:\
//              lcm_bytes / size_of_t == 0. STRIDE CAN NEVER BE ZERO",
//         );
//         element_count
//     }
//     // builds a layout based on the size of T
//     // that guarentees that T is a multiple of 32
// }

// impl<T> Sqlite3Vec<T>
// where
//     T: Sized,
// {

//     pub const fn layout(elements: NonZeroUsize) -> Layout {
//         let size_of_t = Self::size_of_inner();
//         let size_of_register = Sqlite3Memory::nonzero_size_of(); // fuck gemini its trying to get me to put 32 here
//         let align_of = Sqlite3Memory::nonzero_align_of();
//         // because size_of T can never be zero, we remove this case

//         // if size_of_t == 0 {
//         //     return unsafe { Layout::from_size_align_unchecked(size_of_register, align_of) };
//         // }

//         // 1. Calculate the raw exact bytes your elements actually occupy
//         let exact_bytes = elements
//             .saturating_mul(size_of_t);
//             // .expect("Sqlite3Vec: FATAL INVARIANT VIOLATION: overflow when calculating exact_bytes");

//         // 2. Round the BYTE count up to the next highest multiple of 32 bytes
//         let remainder = exact_bytes.get() % size_of_register.get();
//         let final_allocated_bytes = if remainder == 0 {
//             exact_bytes
//         } else {
//             exact_bytes.checked_add(size_of_register.s - remainder)
//             .expect("Sqlite3Vec: FATAL INVARIANT Violation: Overflow when calculating final byte allocation size")
//         };
//         if final_allocated_bytes.get() >= isize::MAX as usize {
//             panic!("Sqlite3Vec: Fatal Invariant Violation: Requested allocation size > isize::MAX")
//         }

//         // 3. Build a 32-byte hardware-aligned allocation token
//         unsafe { Layout::from_size_align_unchecked(final_allocated_bytes.get(), align_of.get()) }
//     }
//     /// casts ptr as &[T;N] without checking. be exra careful here with the ptr offset as it in T elements not u8
//     pub unsafe fn view_as_array_unchecked<const N: usize>(&self, offset: usize) -> &[T; N] {
//         unsafe { self.ptr.add(offset).cast_array().as_ref() }
//     }
//     // pub unsafe fn view_as_simd_slice(&self, offset: usize) -> &[Sqlite3Memory] {
//     //     unsafe {
//     //         self.ptr
//     //             .cast::<Sqlite3Memory>()
//     //             .add(offset)
//     //             .cast_slice(self.size.get())
//     //             .as_ref()
//     //     }
//     // }
// }

impl<'a, T> core::fmt::Display for Sqlite3Vec<'a, T>
where
    T: Display + InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // let s = unsafe { self.ptr.as_ptrcast_slice(self.len().get()).as_ref() };
        let mut iter = self.ptr.iter();
        write!(f, "[")?;
        if let Some(val) = iter.next() {
            write!(f, "{val}")?;
        }
        for el in iter {
            write!(f, ",{el}")?;
        }
        write!(f, "]").unwrap();
        Ok(())
    }
}
impl<'a, T> Sqlite3Vec<'a, T>
where
    T: Sized + InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    //// views this collection as a slice of bytes regardless of what T is
    // pub unsafe fn as_raw_byte_slice(&self) -> &[u8] {
    //     unsafe { self.ptr.cast::<u8>().cast_slice(self.size.get()).as_ref() }
    // }
    // pub fn print_binary_as_u8(&self) {
    //     let slice = unsafe { self.as_raw_byte_slice() };
    //     let mut s =  String::with_capacity(slice.len());
    //     s.push_str("sqlite3Vec binary dump:[");
    //     for el in slice {
    //         let f = format!("{el:08b},");
    //         s.push_str(&f);
    //     }
    //     s.push(']');
    //     println!("{s}");
    // }
}

impl<'a, T> Sqlite3Vec<'a, T>
where
    T: Sized + Default + Clone + InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    #[inline]
    pub fn from_iterator<I>(i: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut iter = i.into_iter();
        let mut _self: Self = Sqlite3Vec::with_initial_size(NonZeroUsize::new(1).unwrap(), None);
        while let Some(element) = iter.next() {
            // _self.push(element).unwrap()
            todo!()
        }
        _self
    }
}

// impl<'a, T> Sqlite3Vec<'a, T>
// where T: crate::mem::InputType + Sized,
//     Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
//     Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
//     Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,

// {
//     #[inline]
//     pub fn cast_into_memory(&self) -> Sqlite3Vec<Sqlite3Memory<T>> {
//         Sqlite3Vec {
//             ptr: self.ptr.cast::<Sqlite3Memory<T>>(),
//             // I guess this becomes the byte size where the last valid byte is
//             len:self.len.checked_mul(Self::size_of_inner()).unwrap(),
//             size:self.size
//         }
//     }
//     #[inline]

//     pub fn cast_from_memory(mem: Sqlite3Vec<Sqlite3Memory<T>>) -> Self {
//         Self {
//             ptr: mem.ptr.cast::<T>(),
//             len:NonZeroUsize::new(mem.len().get() / Self::size_of_inner()).unwrap(),
//             size:mem.size
//         }
//     }
// }

// for this I decided it was safer to just call T debug impl for each T
impl<'a, T> core::fmt::Debug for Sqlite3Vec<'a, T>
where
    T: core::fmt::Debug + InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // 1. Calculate the total capacity in elements of T
        let slice = unsafe { self.ptr.as_ptr().cast_slice(self.len().get()).as_ref() };
        write!(
            f,
            "Sqlite3Vec<{}>{{ ptr: {:?} }}\n Data: [",
            core::any::type_name::<T>(),
            // self.len,
            // self.size,
            self.ptr
        )?;
        for el in slice {
            write!(f, "{el:?},")?;
        }
        write!(f, "];")
    }
}

// 3. CLEANUP ENGINE: Avoids any platform memory leak anomalies
impl<'a, T> Drop for Sqlite3Vec<'a, T>
where
    T: InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    fn drop(&mut self) {
        // Build the correct tracking signature matching our initial allocation specifications
        // let layout = Self::layout(self.len);
        unsafe {
            // Drop instances sequentially so deep memory traits release properly
            // let mut current_ptr = self.ptr.as_ptr();
            // for _ in 0..self.len().get() {
            //     core::ptr::drop_in_place(current_ptr);
            //     current_ptr = current_ptr.add(1);
            // }
            // Deallocate the underlying memory page structure
            crate::mem::dealloc::<T>(&mut self.ptr);
        }
    }
}

/// treats t like a bag of bytes and performs direct comparisons
impl<'a, T> PartialEq for Sqlite3Vec<'a, T>
where
    T: PartialEq + InputType,
    Sqlite3Memory<T>: PartialEq,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    // Performs eq byte wise on underlying memory, ignoring T's special eq implementation
    // ignores any len and leverages the alignment and zeroing guarentees by this primitive to perform the comparison
    // does not perform any len precondition checks.
    // does not 'check' if there is any valid memory becaue it as asserted at compile time
    // by invariants
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        /* we dont check len here because:
        1: we guarentee at compile time that slot zero always refers to some valid form of T
           (unless you manually constructed it with zeroed with is invalid)
        2: it doesnt matter if the len is different, only the size at the byte level
        */
        if self.ptr.as_ptr().eq(&other.ptr.as_ptr()) {
            return true;
        }
        if self.ptr.len() != other.ptr.len() {
            return false;
        }
        // 3. COMPILE-TIME ROUTING VIA INVERTED LOOKUP:
        // If T doesn't have a destructor, it is structurally a plain old data bag of bytes.
        // This if condition evaluates entirely at compile time!
        if !core::mem::needs_drop::<T>() {
            unsafe {
                // Calculate exactly how many 32-byte blocks are allocated
                let block_count = self.ptr.len().get() / 32;

                // Cast your raw heap pointers into perfectly aligned slices of 32-byte chunks
                let lhs: &[[u8; 32]] =
                    core::slice::from_raw_parts(self.ptr.as_ptr().as_ptr().cast::<[u8; 32]>(), block_count);
                let rhs: &[[u8; 32]] =
                    core::slice::from_raw_parts(other.ptr.as_ptr().as_ptr().cast::<[u8; 32]>(), block_count);

                // Invoke raw_eq on the 32-byte block slices!
                // LLVM compiles this branch down into pure inline AVX lane register sweeps
                core::intrinsics::raw_eq(&lhs, &rhs)
            }
        } else {
            // Fallback routing: Only compiles if T manages complex heap resources
            for i in 0..self.ptr.len().get() {
                unsafe {
                    if self.ptr.as_ptr().add(i).as_ref() != other.ptr.as_ptr().add(i).as_ref() {
                        return false;
                    }
                }
            }
            true
        }
    }
}

impl<'a,T> Eq for Sqlite3Vec<'a,T> where T: Eq + InputType,
    Sqlite3Memory<T>:PartialEq,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,

{}

impl<'a,T> core::fmt::Binary for Sqlite3Vec<'a,T>

where
    T: Binary + InputType,
    Sqlite3Memory<T>:Binary,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let slice = unsafe { self.ptr.as_ptr().cast_slice(self.len().get()).as_ref() };
        write!(f, "[")?;
        for el in slice {
            write!(f, "{el:b},")?;
        }
        write!(f, "]")
    }
}

impl<'a,T> Sqlite3Vec<'a,T>
where
    T: Sized + InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    pub fn len(&self) -> NonZeroUsize {
        self.ptr.len()
    }
    // pub fn size(&self) -> NonZeroUsize {
    //     self.size
    // }
    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr().cast().as_ptr()
    }
    //// resizes the underlying vector so that it holds at least count of elements.
    //// if the vector is already large enough, this does nothing
    // pub fn push(&mut self, t: T) -> Result<(), &'static str> {
    //     let current_size_bytes = self.size;
    //     let size_of_t_nonzero = Self::size_of_inner();
    //     let required_bytes = size_of_t_nonzero.checked_mul(self.len().checked_add(1).unwrap())
    //     .ok_or("Sqlite3Vec: Invariant Violation: Overflow when calulating the required bytes for push")?;

    //     let next_len = self
    //         .len
    //         .checked_add(1)
    //         .ok_or("Sqlite3Vec: Invariant Violation: Length overflowed usize")?;

    //     // 2. Growth path: Triggered only if the new item spills past the current 32-byte aligned window
    //     if required_bytes.get() > current_size_bytes.get() {
    //         let stride_nonzero = Self::stride();

    //         let double = unsafe { NonZeroUsize::new_unchecked(2) };
    //         // Amortized growth: Double the current item length to minimize allocator hammering
    //         let double_len = next_len
    //             .checked_mul(double)
    //             .ok_or("Sqlite3: Invaraint Violation. overflow when calulating next len")?;

    //         // Align the doubled length upward to a clean multiple of your calculated element stride
    //         let stride_elements = stride_nonzero;

    //         let step_a =
    //             (double_len.checked_add(stride_elements.get())).ok_or("overflow step a")?;
    //         // its actually really important for at least one of these to be divided on nonzerousize
    //         // especially the divisor
    //         let step_b = step_a.get() / stride_elements;
    //         // both inputs to step b can never be zero so result cannot be zero either
    //         let step_b = unsafe { NonZeroUsize::new_unchecked(step_b) };

    //         let next_multiple_of_stride = step_b.checked_mul(stride_elements)
    //         .ok_or("Sqlite3Vec: Invaraint Violation: Overflow while calulating next multiple of stride")?;

    //         // Ensure the new element count capacity is mathematically sufficient for this push
    //         let new_size = if next_multiple_of_stride >= next_len {
    //             next_multiple_of_stride
    //         } else {
    //             next_len
    //         };

    //         // let new_size_token = unsafe { NonZeroUsize::new_unchecked(target_elements) };

    //         // Execute your safe zero-initialized reallocation function
    //         self.realloc_zeroed(new_size);
    //     }
    //     unsafe {
    //         self.ptr.add(self.len().get()).write(t);
    //         self.len = next_len;
    //     }
    //     Ok(())
    //     // todo!()
    // }
}

// impl<T> Sqlite3Vec<T>
// where
//     T: Sized + Default,
// {
//     pub fn pop(&mut self) -> T {
//         // if the len is one, we copy T onto the stack, call drop in place
//         // then leave the zeroeth element in a valid default state.
//         // this kinda turns into a factory method
//         unsafe {
//             if self.len == NonZeroUsize::new_unchecked(1) {
//                 let last_item = core::ptr::read(self.ptr.as_ptr());
//                 if core::mem::needs_drop::<T>() {
//                     core::ptr::drop_in_place(self.ptr.as_ptr());
//                 }
//                 core::ptr::write(self.ptr.as_ptr(), T::default());
//                 return last_item;
//             }
//         }
//         unsafe {
//             // this was already checked and self.len cant be zero so
//             // all values here are greater than or equal to one.
//             // HMM this is paying of nicely (nonzerousize) for len
//             // this also means that its impossible for this to unferflow and cause ub
//             let id = self.len().get().unchecked_sub(1);
//             let target_ptr = self.ptr.add(id).as_ptr();
//             let popped_item = core::ptr::read(target_ptr);
//             if core::mem::needs_drop::<T>() {
//                 core::ptr::drop_in_place(target_ptr);
//             }
//             core::ptr::write_bytes(target_ptr, 0, 1);
//             self.len = NonZeroUsize::new_unchecked(id);
//             return popped_item;
//         }
//     }
// }

// impl PartialEq for Sqlite3Vec<>

/// Indexing into this vector is inherently safe as long
/// as you make sure to bound the len at the call site,
/// otherwise this does not check bounds at all except
/// for a debug assert during development.
///
/// The full responsibility goes on the developer to ensure
/// that the index is always bounded!
impl<'a,T> Index<usize> for Sqlite3Vec<'a,T>
    where T:InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,
{
    type Output = T;
    #[inline(always)]
    /// 'safely' uses unsafe to address the underlying pointer in elements of T.
    /// as per the docs for the impl, this function DOES NOT bounds check,
    /// except for development. it is the responsibility of the developer to ensure
    /// that the design of the program upholds this invariant
    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(
            index < self.ptr.len().into(),
            "Sqlite3Vec: Index out of bounds (Debug): len is {} but index is {}",
            self.ptr.len(),
            index
        );
        unsafe { self.ptr.as_ptr().cast::<T>().add(index).as_ref() }
    }
}
/// Indexing into this vector is inherently safe as long
/// as you make sure to bound the len at the call site,
/// otherwise this does not check bounds at all except
/// for a debug assert during development.
///
/// The full responsibility goes on the developer to ensure
/// that the index is always bounded!
impl<'a, T> IndexMut<usize> for Sqlite3Vec<'a, T>
    where T: InputType,
    Assert<{ core::mem::size_of::<T>() > 0 }>: IsTrue,
    Assert<{ 32 % core::mem::size_of::<T>() == 0 }>: IsTrue,
    Assert<{ core::mem::align_of::<T>() > 0 }>: IsTrue,


{
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        debug_assert!(
            index < self.ptr.len().into(),
            "Sqlite3Vec: IndexMut out of bounds (Debug): len is {} but index is {}",
            self.ptr.len(),
            index
        );
        unsafe { self.ptr.as_ptr().cast::<T>().add(index).as_mut() }
    }
}

// impl Sqlite3Vec<u8> {
//     // treats the underlying type as valid utf8
//     // and perfroms the utf8 -> 16 encoding directly
//     pub fn encode_as_wide_text_unchecked(&mut self, _buf: &mut Sqlite3Vec<u16>) {
//         let _offset = 0;
//         self[0] = 123;
//     }
// }

pub fn index_mut_with_eq_asm_output_check(
    lhs: &mut Sqlite3Vec<u64>,
    rhs: &mut Sqlite3Vec<u64>,
) -> bool {
    lhs[0] = 64;
    rhs[20] = 128;
    // let _ = rhs.pop();
    // lhs.push(256).ok();
    // rhs.push(987).ok();
    // rhs.push(654).ok();
    // lhs == rhs
    true
}

#[cfg_attr(test, test)]
pub fn test_vec() -> () {
    let vec: Sqlite3Vec<u8> = Sqlite3Vec::from_slice(&[u8::MAX, 1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
    if vec[0] == 2{return}
    // println!("{vec}");
    println!("{vec:?}");
    // assert_eq!(vec, vec);
    // let _ = vec == vec;
    // let expected_len = unsafe { NonZeroUsize::new_unchecked(9) };
    // assert_eq!(vec.len, expected_len);
    // let expected_size = unsafe { NonZeroUsize::new_unchecked(32) };
    // assert_eq!(vec.size, expected_size);
    // vec.print_binary_as_u8();
    // vec[3] = 6;
    // assert_eq!(vec[3], 6);
    // vec.push(2).ok();
    // let two = vec.pop();
    // assert_eq!(two, 2);

    // let vec: Sqlite3Vec<u8> = Sqlite3Vec::new();
    // println!("{vec}");
    // println!("{vec:?}");
    // assert_eq!(vec, vec);
    // let expected_len = unsafe { NonZeroUsize::new_unchecked(1) };
    // assert_eq!(vec.len, expected_len);
    // let expected_size = unsafe { NonZeroUsize::new_unchecked(32) };
    // assert_eq!(vec.size, expected_size);
    // assert_eq!(vec[0], u8::default());

    // let mut vec: Sqlite3Vec<u16> = Sqlite3Vec::from_slice(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]).unwrap();
    // println!("{vec}");
    // println!("{vec:?}");
    // assert_eq!(vec, vec);
    // vec[4] = 8;
    // assert_eq!(vec[4], 8);
    // let mut VEC3: Sqlite3Vec<u16> =
    //     Sqlite3Vec::from_slice(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16])
    //         .unwrap();
    // assert_eq!(VEC3.len.get(), 17);
    // assert_eq!(VEC3.size.get(), 64);
    // println!("{VEC3:?}");
    // VEC3.print_binary_as_u8();

    // let mut vec: Sqlite3Vec<u32> = Sqlite3Vec::new();
    // assert_eq!(vec.len.get(), 1);
    // assert_eq!(vec.size, Sqlite3Memory::nonzero_size_of());
    // vec[0] = u32::MAX;
    // assert_eq!(vec[0], u32::MAX);
    // println!("{vec:?}");
    // vec.print_binary_as_u8();

    // let mut vec = Sqlite3Vec::with_initial_size(255.try_into().unwrap(), Some(-1_isize));

    // for i in 0..vec.len.get() {
    //     if vec[i] != -1_isize {
    //         panic!("i:{i} val{}", vec[i])
    //     }
    // }
    // println!("{vec:?}");
    // vec.print_binary_as_u8();

    // let mut vec: Sqlite3Vec<AwkwardCell> =
    //     Sqlite3Vec::with_initial_size(14.try_into().unwrap(), None);
    // println!("{vec:b}");
    // assert_eq!(vec.len.get(), 14);
    // vec.print_binary_as_u8();

    // let rainbow_cell = AwkwardCell {
    //     id: 0x55AA_FF00,
    //     flags: 0xF00F,
    //     category: 0xA5,
    // };
    // vec.push(rainbow_cell).unwrap();
    // let old = vec.pop();
    // assert_eq!(old,rainbow_cell);

    // println!("{vec:?}");
    // // this should crash in debug mode
    // // let _ = vec[14];

    // // VEC3.resize_with_default_value(NonZeroUsize::new(255).unwrap());
    // // println!("{VEC3}");
    // // println!("{VEC3:?}");
}
// the wierd size type case
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(packed)] // Forces the compiler to skip padding holes, ensuring size_of is exactly 7 bytes
pub struct AwkwardCell {
    pub id: u32,      // 4 bytes
    pub flags: u16,   // 2 bytes
    pub category: u8, // 1 byte
}
impl Default for AwkwardCell {
    fn default() -> Self {
        Self {
            id: u32::MAX,
            flags: 0xDEAD,
            category: 128,
        }
    }
}
impl core::fmt::Binary for AwkwardCell {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "AwkwardCell{{{:032b},{:016b},{:08b}}}",
            unsafe { (&raw const self.id).read_unaligned() },
            unsafe { (&raw const self.flags).read_unaligned() },
            unsafe { (&raw const self.category).read_unaligned() }
        )
    }
}

// impl<T> Sqlite3Vec<T> where T: PartialEq {}

// impl<T> Sqlite3Vec<T> {
//     // this collection only grows, never shrinks, writes zeroes to new memory
//     // does not add new elements to T yet, only increases the ability to hold more T elements
//     pub fn realloc_zeroed(&mut self, new_size: NonZeroUsize) {
//         let old_len = self.len;
//         let old_layout = Self::layout(old_len);
//         let new_layout = Self::layout(new_size);
//         let new_ptr = unsafe {
//             core::alloc::realloc(self.ptr.as_ptr().cast(), old_layout, new_layout.size())
//         };
//         let new_ptr = NonNull::new(new_ptr)
//             .expect("Sqlite3Memory: Invariant Violation: Allcoation failed Or OOM reached");
//         unsafe {
//             let new_start = new_ptr.add(old_layout.size());
//             let new_count = new_layout.size() - old_layout.size();
//             core::ptr::write_bytes(new_start.as_ptr(), 0, new_count);
//         }
//         self.ptr = new_ptr.cast();
//         self.size = NonZeroUsize::new(new_layout.size())
//             .expect("Sqlite3Memory: Zero Size encountered while reallocating");
//     }
// }
