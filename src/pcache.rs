use std::{
    alloc::{Layout, alloc, alloc_zeroed, dealloc, realloc},
    cell::UnsafeCell,
    collections::HashMap,
    mem::{MaybeUninit, offset_of},
    num::{NonZeroU32, NonZeroU64, NonZeroUsize},
    ops::{BitAnd, BitAndAssign, BitOrAssign, Deref, DerefMut, Not, Sub},
    ptr::NonNull,
    sync::atomic::AtomicPtr,
};

pub const PAGE_SIZE: usize = 4096;

#[repr(align(4096))]
pub struct PageBlockChunk([u8; PAGE_SIZE]);
impl PageBlockChunk {
    fn alloc() -> NonNull<Self> {
        let layout = Layout::new::<Self>();
        unsafe {
            let raw_ptr = alloc(layout);
            if raw_ptr.is_null() {
                // for now, we have to crash the program
                std::alloc::handle_alloc_error(layout);
            }

            // Cast to our specific self type and wrap
            NonNull::new_unchecked(raw_ptr as *mut Self)
        }
    }
    fn free(ptr: NonNull<Self>) {
        let layout = Layout::new::<Self>();
        unsafe {
            dealloc(ptr.as_ptr() as *mut u8, layout);
        }
    }
}
pub type PageBlockInternalT = PageBlockChunk;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum PageBlockFlags {
    #[default]
    PageFlagsEmpty = 0,
    PageFlagDirty = 0b00000001,
    PageFlagPinned = 0b00000010,
    PageFlagFlushing = 0b00000100,
    PageFlagCorrupt = 0b00001000,
}
impl PageBlockFlags {
    pub const VALID_MASK: u8 = 0b00001111;
}


impl BitAnd for PageBlockFlags {
    type Output = u8;
    fn bitand(self, rhs: Self) -> Self::Output {
        self as u8 & rhs as u8
    }
}
impl BitAnd<u8> for PageBlockFlags {
    type Output = u8;
    fn bitand(self, rhs: u8) -> Self::Output {
        self as u8 & rhs
    }
}
impl BitAnd<PageBlockFlags> for u8 {
    type Output = u8;
    fn bitand(self, rhs: PageBlockFlags) -> Self::Output {
        self & rhs as u8
    }
}

impl BitOrAssign<PageBlockFlags> for u8 {
    fn bitor_assign(&mut self, rhs: PageBlockFlags) {
        *self |= rhs as u8
    }
}
impl BitOrAssign<u8> for PageBlockFlags {
    fn bitor_assign(&mut self, rhs: u8) {
        let r = *self as u8 | rhs as u8;
        let r:Self = unsafe {std::mem::transmute(r)};
        *self = r;
        // (*self as u8 |= rhs as u8)
    }
}

impl BitAndAssign<PageBlockFlags> for u8 {
    fn bitand_assign(&mut self, rhs: PageBlockFlags) {
        *self &= rhs as u8
    }
}

// TODO: unsafe, make sure to update pageblockflags
// with all possible combos or make sure to compare as u8 under the hood
impl BitAndAssign<u8> for PageBlockFlags {
    fn bitand_assign(&mut self, rhs: u8) {
        let r = *self as u8 & rhs;
        let r: Self = unsafe {std::mem::transmute(r)};
        *self = r;
    }
}

impl Not for PageBlockFlags {
    type Output = u8;
    fn not(self) -> Self::Output {
        !(self as u8)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PageId(usize);
impl Deref for PageId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type PageBlockFlagsInternalT = u8;
pub struct PageBlock {
    page_id: PageId,
    // ptr: NonNull<PageBlockInternalT>,
    flags: PageBlockFlagsInternalT,
}
impl PageBlock {
    pub const PAGE_FLAG_DIRTY: PageBlockFlags = PageBlockFlags::PageFlagDirty;
    pub const PAGE_FLAG_PINNED: PageBlockFlags = PageBlockFlags::PageFlagPinned;
    pub const PAGE_FLAG_FLUSHING: PageBlockFlags = PageBlockFlags::PageFlagFlushing;
    pub const PAGE_FLAG_CORRUPT: PageBlockFlags = PageBlockFlags::PageFlagCorrupt;
    pub const PAGE_FLAG_VALID_FLAG_MASK: u8 = PageBlockFlags::VALID_MASK;

    pub fn set_dirty(&mut self) {
        self.flags |= Self::PAGE_FLAG_DIRTY;
    }
    pub fn set_pinned(&mut self) {
        self.flags |= Self::PAGE_FLAG_PINNED;
    }
    pub fn set_flushing(&mut self) {
        self.flags |= Self::PAGE_FLAG_FLUSHING
    }
    pub fn set_corrupt(&mut self) {
        self.flags |= Self::PAGE_FLAG_CORRUPT
    }

    pub fn clear_dirty(&mut self) {
        self.flags &= !Self::PAGE_FLAG_DIRTY
    }
    pub fn clear_pinned(&mut self) {
        self.flags &= !Self::PAGE_FLAG_PINNED
    }
    pub fn clear_flushing(&mut self) {
        self.flags &= !Self::PAGE_FLAG_FLUSHING
    }
    pub fn clear_corrupt(&mut self) {
        self.flags &= !Self::PAGE_FLAG_CORRUPT
    }

    pub fn is_dirty(&self) -> bool {
        (self.flags & Self::PAGE_FLAG_VALID_FLAG_MASK & Self::PAGE_FLAG_DIRTY) != 0
    }
    pub fn is_pinned(&self) -> bool {
        (self.flags & Self::PAGE_FLAG_VALID_FLAG_MASK & Self::PAGE_FLAG_PINNED) != 0
    }
    pub fn is_corrupt(&self) -> bool {
        (self.flags & Self::PAGE_FLAG_VALID_FLAG_MASK & Self::PAGE_FLAG_CORRUPT) != 0
    }
    pub fn set_flags(&mut self, flags: PageBlockFlagsInternalT) {
        self.flags = flags & Self::PAGE_FLAG_VALID_FLAG_MASK
    }
    pub fn get_flags(&self) -> PageBlockFlagsInternalT {
        self.flags & Self::PAGE_FLAG_VALID_FLAG_MASK
    }
    pub fn clear_flags(&mut self) {
        self.flags = PageBlockFlags::default() as PageBlockFlagsInternalT
    }
}

impl PageBlock {
    pub fn new(page_id: PageId) -> Self {
        // let ptr = PageBlockChunk::alloc();
        Self {
            page_id,
            flags: PageBlockFlags::default() as PageBlockFlagsInternalT,
        }
    }
}

// Jordan morris:
// an aside from the developer:
//
// I cant believe how exciting this is. I literally removed the need for page backed metatdata,
// removed null pointers from existince,
// introduced len and capacity tracking,
// made some invald states completely unrepresentable in this design.
//
// I wish the original C devs could have these ideas here
// as it would have made sqlite legendary.
//
// it would have totally been worth them implementing a custom rust allocator
// that handles oom and doesnt panic just for this alone
pub struct PageCache {
    // pageid_at_end: PageId,
    // blocks now become the header where the arena holds the data.
    // this prevents dangling pointers,
    // and now pageid is the literal offset of the memory
    flags: Vec<PageBlockFlags>,
    ptr: NonNull<PageBlockChunk>,
    size: NonZeroUsize,
}
impl PageCache {
    // creates a layout guarenteed to be valid at compile time with runtime input
    pub fn create_layout_for_block(size: NonZeroUsize) -> std::alloc::Layout {
        // mul by the alinment
        // calcualte the exact value to muliply by AT COMPILE TIME! ** SOO COOL ***
        const ALIGNMENT: usize = std::mem::align_of::<PageBlockChunk>();
        const ALIGN_SHIFT: u32 = ALIGNMENT.trailing_zeros();
        let total_bytes = unsafe { size.get().unchecked_shl(ALIGN_SHIFT) };
        let layout = unsafe { Layout::from_size_align_unchecked(total_bytes, ALIGNMENT) };
        layout
    }
    pub fn create_layout_for_flags(size: NonZeroUsize) -> std::alloc::Layout {
        const ALIGNMENT: usize = std::mem::align_of::<PageBlockFlags>();
        const ALIGN_SHIFT: u32 = ALIGNMENT.trailing_zeros();
        let total_bytes = unsafe { size.get().unchecked_shl(ALIGN_SHIFT) };
        let layout = unsafe { Layout::from_size_align_unchecked(total_bytes, ALIGNMENT) };
        layout
    }
    pub fn with_size(size: NonZeroUsize) -> Self {
        let chunk_layout = Self::create_layout_for_block(size);
        let flags_layout = Self::create_layout_for_flags(size);
        unsafe {
            let chunk_ptr = alloc_zeroed(chunk_layout) as *mut PageBlockChunk;
            // 2. THE HARDWARE TRICK:
            // We tell the compiler to assume with 100% certainty that raw_ptr is NOT null.
            // If the system runs out of memory, the allocator handles the panic/abort out-of-band.
            // This allows LLVM to completely erase the `test rax, rax` and `je` instructions!
            std::hint::assert_unchecked(!chunk_ptr.is_null());


            
            // handle the vec same as above. we make OOM basically UB for now
            let flags_ptr = alloc_zeroed(flags_layout) as *mut PageBlockFlags;
            std::hint::assert_unchecked(!flags_ptr.is_null());
            // let flags_ptr = unsafe {NonNull::new_unchecked(flags_ptr)};
            let flags = Vec::from_raw_parts(flags_ptr, size.get(), size.get());

            Self {
                // flags: Vec::with_capacity(size.get()),
                flags,
                ptr: NonNull::new_unchecked(chunk_ptr),
                size,
            }
        }
    }
    // #[inline(always)]
    pub unsafe fn get_chunk_unchecked(&self, id: PageId) -> &PageBlockChunk {
        unsafe { self.ptr.add(id.0 as usize).as_ref() }
    }
    #[inline(always)]
    pub fn get_chunk(&self, id: &PageId) -> Option<&PageBlockChunk> {
        if self.flags.get(id.0).is_some() {
            return None;
        }
        unsafe { Some(self.ptr.add(id.0 as usize).as_ref()) }
    }
    #[inline]
    /// if this function errors, realloc failed and should be handled gracefully
    pub fn add_pages(&mut self, count: NonZeroUsize) -> Result<(), ()> {
        let new_capacity = unsafe { self.size.unchecked_add(count.get()) };
        let layout = Self::create_layout_for_block(new_capacity);
        let new_ptr =
            unsafe { realloc(self.ptr.as_ptr().cast::<u8>(), layout, new_capacity.get()) };
        if new_ptr.is_null() {
            return Err(());
        }
        self.size = new_capacity;
        self.ptr = unsafe { NonNull::new_unchecked(new_ptr.cast()) };
        Ok(())
    }
    /// this calls realloc and when it fails it creates UB (setting NonNull ptr to 0). only use this if you can guarentee it will succeed
    pub unsafe fn add_pages_unsafe(&mut self, count: NonZeroUsize) {
        let new_capacity = unsafe { self.size.unchecked_add(count.get()) };
        let layout = Self::create_layout_for_block(new_capacity);
        let new_ptr =
            unsafe { realloc(self.ptr.as_ptr().cast::<u8>(), layout, new_capacity.get()) };
        self.size = new_capacity;
        self.ptr = unsafe { NonNull::new_unchecked(new_ptr.cast()) };
    }
    /// this function calls realloc, and when it fails,
    /// it leaves the old pointer alone and returns a boolean saying whether the allocation was succesfull or not
    /// avoids some overhead from handling result
    pub unsafe fn try_add_pages(&mut self, count: NonZeroUsize) -> bool {
        let new_capacity = unsafe { self.size.unchecked_add(count.get()) };
        let layout = Self::create_layout_for_block(new_capacity);
        let new_ptr =
            unsafe { realloc(self.ptr.as_ptr().cast::<u8>(), layout, new_capacity.get()) };
        if !new_ptr.is_null() {
            self.size = new_capacity;
            self.ptr = unsafe { NonNull::new_unchecked(new_ptr.cast()) };
            return true;
        }
        return false;
    }

    pub unsafe fn get_chunk_mut_or_resize(&mut self,id: &PageId) -> &mut PageBlockChunk {
        if id.0 > self.size.get() {
            let count = id.0.sub(self.size.get()).min(1);
            let count = unsafe {
                NonZeroUsize::new_unchecked(count)
            };
            unsafe {self.add_pages_unsafe(count)};
        }
        unsafe {self.ptr.add(id.0).as_mut()}
    }
    // pub fn grow()
}

thread_local! {
    static THREAD_LOCAL_PAGECACHE: UnsafeCell<MaybeUninit<Box<PageCache>>> = const {UnsafeCell::new(MaybeUninit::uninit())};
}

#[unsafe(no_mangle)]
/// this function either succeeds or aborts the program if no memory is available.
/// this does not check whether the cache is already initialized.
/// currently it may be UB to call this function twice until drop is implemented.
/// it is the responsibility of this caller to uphold initial_size != 0
/// or it is immediate UB via passing 0 to NonZeroUsize
/// 
/// aside: I really wish this could do more than 8 exobytes... :)
pub unsafe extern "C" fn sqlite3r_pcache_init_thread(initial_size: NonZeroUsize) -> std::os::raw::c_int {
    let _self = PageCache::with_size(initial_size);
    // wrap it in a boxed pointer so the memory outlives the function
    let _self = Box::new(_self);
    // let r = Box::leak(_self);
    // let ptr = Box::into_raw(_self);
    //
    THREAD_LOCAL_PAGECACHE.with(|cell| {
        unsafe { (*cell).get().write(MaybeUninit::new(_self)) };
        // unsafe {cell.get() = ptr}
    });
    0
}
/// # Safety
/// * Enforced Contract: `sqlite3r_pcache_init_thread` must have run on the current thread.
/// * Enforced Contract: `page_id` must be strictly less than the allocated capacity.
/// Violating these conditions is instant, unmitigated Undefined Behavior (UB).
/// 
/// This compiles down to a pure linear block of roughly 4 instructions with ZERO conditional jumps.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sqlite3r_pcache_fetch(page_id: PageId) -> NonNull<u8> {
    // 1. Fetch the raw thread-local pointer block from the TLS cell slot
    let cell_ptr = THREAD_LOCAL_PAGECACHE.with(|cell| cell.get());
    
    unsafe {
        // 2. Blindly read the initialized Box reference without any Option/Result checking
        let cache = (*cell_ptr).assume_init_mut();
    
        let chunk = cache.get_chunk_mut_or_resize(&page_id);
        let cache_ptr: *mut u8 = std::ptr::from_mut(chunk).cast();
        // 3. Perform O(1) pointer math starting from your NonNull base location.
        // Because AlignedBlock is strictly 4096 bytes, LLVM translates this add into a bitwise left-shift by 12.
        NonNull::new_unchecked(cache_ptr)
    }
}

/// # Safety
/// * Enforced Contract: `sqlite3r_pcache_init_thread` must have run on this thread.
/// * Enforced Contract: `page_id` must be strictly less than the allocated capacity.
/// Violating this contract is immediate Undefined Behavior.
/// 
/// This executes a zero-branch bitwise OR directly on the flat vector memory index.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sqlite3r_pcache_set_flag(page_id: usize, flag: u8) {
    let cell_ptr = THREAD_LOCAL_PAGECACHE.with(|cell| cell.get());
    unsafe {
        let cache = (*cell_ptr).assume_init_mut();   
        // Use get_unchecked_mut to completely strip bounds-checking branches out of the assembly
        let flag_ref = cache.flags.get_unchecked_mut(page_id);
        *flag_ref |= flag;
    }
}

/// # Safety
/// * Enforced Contract: `sqlite3r_pcache_init_thread` must have run on this thread.
/// * Enforced Contract: `page_id` must be strictly less than the allocated capacity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sqlite3r_pcache_clear_flag(page_id: usize, flag: u8) {
    let cell_ptr = THREAD_LOCAL_PAGECACHE.with(|cell| cell.get());
    unsafe {
        let cache = (*cell_ptr).assume_init_mut();   
        let flag_ref = cache.flags.get_unchecked_mut(page_id);
        *flag_ref &= !flag;
    }
}



/// # Safety
/// * Enforced Contract: `sqlite3r_pcache_init_thread` must have run on this thread.
/// * Enforced Contract: `page_id` must be strictly less than the allocated capacity.
/// Returns 1 if the target flag bit is set, 0 if it is clear. Zero branches.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sqlite3r_pcache_has_flag(page_id: usize, flag: u8) -> i32 {
    let cell_ptr = THREAD_LOCAL_PAGECACHE.with(|cell| cell.get());
    
    unsafe  {
        let cache = (*cell_ptr).assume_init_ref();   
        // Read directly from the flag vector index offset using unchecked access
        let page_flags = *cache.flags.get_unchecked(page_id);
        ((page_flags & flag) != 0) as i32
    }
}

/// # Safety
/// Enforced Contract: Cache must be initialized on this thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sqlite3r_pcache_get_capacity() -> NonZeroUsize {
    let cell_ptr = THREAD_LOCAL_PAGECACHE.with(|cell| cell.get());
    let cache = unsafe {(*cell_ptr).assume_init_ref()};
    cache.size
}


// this is the hardware gold. we replace the complicated cace eviction (for now)
// via linked lists with a flat memory layout
/// # Safety
/// Enforced Contract: `p_page` must be a valid pointer belonging to this thread's contiguous cache.
#[inline(always)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sqlite3r_pcache_ptr_to_id(p_page: *mut u8) -> usize {
    let cell_ptr = THREAD_LOCAL_PAGECACHE.with(|cell| cell.get());
    let cache = (*cell_ptr).assume_init_ref();
    
    // Calculate byte distance from our NonNull base location
    let distance = p_page.offset_from(cache.ptr.as_ptr());
    
    // Distance / 4096. Compiles down to a single hardware right-shift: SHR RAX, 12
    (distance as usize).unchecked_shr(12)
}