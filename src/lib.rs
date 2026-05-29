// #![no_std]
#![feature(generic_const_exprs)]
// #![feature(alloc_error_handler)] // Enable explicit out-of-band alloc tracking if on nightly
// #![feature(nonzero_ops)]
// #![feature(allocator_api)]
#![feature(core_intrinsics)]
// #![feature(const_cmp)]
// #![feature(generic_const_exprs)]
#![feature(const_trait_impl)]
// #![feature(const_convert)]
#![feature(ptr_cast_slice)]
#![feature(ptr_cast_array)]
// #![feature(explicit_tail_calls)]
// #![feature(const_heap)]
// #![feature(const_ops)]
// #![feature(unsafe_cell_access)]
// #![feature(stmt_expr_attributes)]
// #![feature(min_specialization)]
pub mod ffi;
// pub mod mem;
pub mod pcache;
pub mod fs;
pub mod vec;
pub mod mem;
pub mod varint;





// pub type SqliteResult<T> = core::result::Result<T,SqliteError>;

// /// Strict, zero-allocation mirror of original SQLite3 primary result codes.
// #[derive(Copy, Clone, Debug, PartialEq, Eq)]
// #[repr(i32)]
// pub enum SqliteError {
//     // Ok = 0,           // SQLITE_OK
//     Error = 1,
//     Busy = 5,                   // SQLITE_BUSY
//     IoError = 10,               // SQLITE_IOERR
//     Corrupt = 11,               // SQLITE_CORRUPT
//     Full = 13,                  // SQLITE_FULL
//     CantOpen = 14,              // SQLITE_CANTOPEN
//     Misuse = 21,                // SQLITE_MISUSE
//     NotADb = 26,      // SQLITE_NOTADB: File opened that is not a database file (Invalid Magic!)
//     // ... all other 31 SQLITE_codes
//     Row = 100,        // SQLITE_ROW
//     Done = 101,       // SQLITE_DONE

//     IoErrRead = 10 | (1 << 8),       // 266: SQLITE_IOERR_READ
//     IoErrWrite = 10 | (3 << 8),      // 778: SQLITE_IOERR_WRITE
//     IoErrShortRead = 10 | (2 << 8),  // 522: SQLITE_IOERR_SHORT_READ

//     // --- Extended CANTOPEN Subcodes ---
//     // Maps to ErrorKind::NotFound (File doesn't exist, cannot open)
//     CantOpenNoFile = 14 | (1 << 8),     // 270: SQLITE_CANTOPEN_ISDIR or custom missing hook

//        // Maps to ErrorKind::PermissionDenied (Path is blocked by OS security)
//     CantOpenNoPerm = 14 | (5 << 8),     // 1294: SQLITE_CANTOPEN_EPERM
    
// }
