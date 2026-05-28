use std::path::Path;
#[cfg(target_os="windows")]
pub mod windows;
#[cfg(target_os="windows")]
pub use windows::prelude::*;




#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct SqliteGlobalHeader {
    /// Must contain the exact 16 bytes: b"SQLite format 3\0"
    pub magic: [u8; 16],
    
    /// Database page size in bytes (Big Endian). Must be a power of two between 512 and 65536.
    /// Value 1 represents 65536. In your architecture, this is locked to 4096 (0x1000).
    pub page_size: u16,
    
    /// File format write version: 1 for legacy journal, 2 for WAL
    pub write_version: u8,
    
    /// File format read version: 1 for legacy journal, 2 for WAL
    pub read_version: u8,
    
    /// Bytes of unused "reserved" space at the end of each page (for extensions/extensions)
    pub reserved_space: u8,
    
    /// Maximum embedded payload fraction. Must be 64.
    pub max_payload_fraction: u8,
    
    /// Minimum embedded payload fraction. Must be 32.
    pub min_payload_fraction: u8,
    
    /// Leaf payload fraction. Must be 32.
    pub leaf_payload_fraction: u8,
    
    /// File change counter. Incremented by 1 on every committed write transaction.
    pub file_change_counter: u32,
    
    /// In-header database size in pages (Big Endian). Valid if the change counter matches 
    /// the version valid-for number below.
    pub db_size_pages: u32,
    
    /// Page number of the first freelist trunk page (Big Endian), or 0 if empty
    pub first_freelist_trunk_page: u32,
    
    /// Total number of freelist pages in the file (Big Endian)
    pub total_freelist_pages: u32,
    
    /// The schema cookie (Big Endian). Incremented whenever the database schema changes.
    pub schema_cookie: u32,
    
    /// The schema format number. Valid versions are 1, 2, 3, and 4.
    pub schema_format: u32,
    
    /// Default page cache size in pages (Big Endian). Sets default cache bounds.
    pub default_pcache_size: u32,
    
    /// Page number of the largest root b-tree page when in auto-vacuum/incremental mode.
    pub largest_root_btree_page: u32,
    
    /// Database text encoding: 1 for UTF-8, 2 for UTF-16LE, 3 for UTF-16BE
    pub text_encoding: u32,
    
    /// User version: Application-specific integer token (Big Endian)
    pub user_version: u32,
    
    /// True for incremental-vacuum mode, false for full auto-vacuum
    pub incremental_vacuum: u32,
    
    /// Application ID: Application-specific integer token (Big Endian)
    pub application_id: u32,
    
    /// Reserved for expansion. Must be entirely zeroed out.
    pub reserved_padding: [u8; 20],
    
    /// Version-valid-for number (Big Endian). 
    /// If this matches the file_change_counter, the db_size_pages field is considered valid.
    pub version_valid_for: u32,
    
    /// SQLITE_VERSION_NUMBER value indicating which engine build wrote the file (Big Endian)
    pub write_library_version: u32,
}
pub enum BTreePageType {
    InteriorIndex = 0x02,
    InteriorTable = 0x05,
    LeafIndex     = 0x0a,
    LeafTable     = 0x0d,
}

/// The Layout header present at the start of EVERY B-Tree page.
/// Note: On Page 1, this header begins exactly at byte 100, right after the Global Header.
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct BTreePageHeader {
    /// Flag byte defining the node type (0x02, 0x05, 0x0a, or 0x0d)
    pub page_type: u8,
    
    /// Byte offset pointing to the first block of free space on the page (Big Endian), or 0
    pub first_freeblock_offset: u16,
    
    /// Total number of data cells packed into this page sector (Big Endian)
    pub cell_count: u16,
    
    /// Byte offset pointing to the start of the cell content area (Big Endian).
    /// If this reads 0, it means the content area begins exactly at byte 65536.
    pub cell_content_start_offset: u16,
    
    /// Total number of fragmented free bytes sitting inside released cells
    pub fragmented_free_bytes: u8,
    
    /* 
     * THE CONDITIONAL ELEMENT:
     * If page_type is 0x02 or 0x05 (Interior Nodes), the header is immediately 
     * appended with a 4-byte big-endian integer pointing to the rightmost child page number.
     * For leaf pages, this field does not exist, and the Cell Pointer Array starts immediately.
     */
}
/// A zero-cost hardware viewport cast over a single 4096-byte memory-mapped page sector.
pub struct PageViewport {
    pub ptr: *mut u8,
    pub is_page_1: bool,
}

impl PageViewport {
    /// Computes where the B-Tree header begins inside this specific page
    #[inline(always)]
    pub unsafe fn header_offset(&self) -> usize {
        if self.is_page_1 { 100 } else { 0 }
    }

    /// Fetches the raw big-endian cell pointer array slice
    #[inline(always)]
    pub unsafe fn cell_pointers(&self) -> &[u16] {
        let base_offset = self.header_offset();
        let header = &*(self.ptr.add(base_offset) as *const BTreePageHeader);
        let cell_count = u16::from_be(header.cell_count) as usize;
        
        // Interior nodes have a 4-byte rightmost pointer, shifting the array down
        let array_start_offset = base_offset + 8 + if header.page_type == 0x02 || header.page_type == 0x05 { 4 } else { 0 };
        
        std::slice::from_raw_parts(self.ptr.add(array_start_offset) as *const u16, cell_count)
    }
}



pub fn open<P: AsRef<std::path::Path>>(p:P) {
    let path = p.as_ref();
    let path = path.canonicalize().unwrap();
    // std::fs::OpenOptions::new().read(true).write(true).create(true)
}