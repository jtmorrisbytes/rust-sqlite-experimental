// use std::{
//     cell::{Cell, UnsafeCell},
//     fs::File,
//     io::{Read, Seek},
// };
// use crate::SqliteError;
// // Jordan to Richard: your going to love this:
// // your magic header is exactly 16 bytes.
// // this means that if we size this exactly at compile time
// // we can tell the cpu to compare on accelerated platforms in only 1-2 cycles
// #[derive(Debug, Default)]
// #[repr(C, align(16))]
// pub struct Sqlite3MagicSequence([u8; 16]);

// impl Sqlite3MagicSequence {
//     pub const BYTES: &[u8; 16] = b"SQLite format 3\0";
//     pub const MAGIC: Self = Self(*Self::BYTES);
// }
// // NOTICE: sse2 has been availible since like 2010.
// // I dont perform runtime checking here because I believe that
// // sse2 should be a minimum requirement in 2025
// // if this causes problems we could make this a feature flag,
// // but optimizations like this are what saves milliseconds file load etc
// //
// // here I attempt to check if this has been compiled with sse 4.1 available.
// // if so the compoiler should evlauate the target feature 4.1 block before sse2.
// // this ensures that the best instructions possible are available for this type.
// #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
// impl PartialEq for Sqlite3MagicSequence {
//     /// Compares two buffers to check if it maches the magic sequence.
//     /// uses a single load along with a in memory fusion when possible
//     fn eq(&self, other: &Self) -> bool {
//         #[cfg(target_arch = "x86_64")]
//         use core::arch::x86_64::*;
//         #[cfg(target_arch = "x86")]
//         use core::arch::x86_64::*;
//         if cfg!(target_feature = "sse4.1") {
//             // Path A: The 1-instruction SSE4.1 PTEST branch
//             unsafe {
//                 let lhs = _mm_load_si128(self.0.as_ptr().cast());
//                 let rhs = _mm_load_si128(other.0.as_ptr().cast());
//                 let diff = _mm_xor_si128(lhs, rhs);
//                 _mm_testz_si128(diff, diff) == 1
//             }
//         } else {
//             // Path B: The highly compatible 2-instruction SSE2 fallback
//             unsafe {
//                 let lhs = _mm_load_si128(self.0.as_ptr().cast());
//                 let rhs = _mm_load_si128(other.0.as_ptr().cast());
//                 let eq = _mm_cmpeq_epi8(lhs, rhs);
//                 _mm_movemask_epi8(eq) == 0xFFFF
//             }
//         }
//     }
// }
// impl Sqlite3MagicSequence {

// }

// #[repr(C, align(16))]
// // NOTE MAKE SURE YOU DONT CAST THIS TYPE DIRECTLY
// // AS THE 100 byte, only cast the data field as the 100 byte
// pub struct Sqlite3FileHeader {
//     data: [u8; 100],
//     _pad: [u8; 12],
// }

// // here we use rusts machinery to allocate a per thread magic byte buffer
// // to read into so we can perform the check at the vfs level without reallocating
// thread_local! {
//     static THREAD_LOCAL_MAGIC_HEADER: UnsafeCell<Sqlite3MagicSequence> = UnsafeCell::new(Sqlite3MagicSequence::default());
// }

// impl Sqlite3FileHeader {
//     pub fn read(f: &mut File) -> crate::SqliteResult<Self> {
//         // f.seek(std::io::SeekFrom::Start(0))?;
//         // read the magic byte first

//         let mut magic = Sqlite3MagicSequence::default();
//         // let r:crate::SqliteResult<bool> = THREAD_LOCAL_MAGIC_HEADER.with(|cell|{
//         //     let magic = unsafe {cell.as_mut_unchecked()};
//         //     Ok(*magic == Sqlite3MagicSequence::MAGIC)
//         // });
//         let r = magic == Sqlite3MagicSequence::MAGIC;
//         // magic.read(&mut *f)?;
//         let is_match = r;
//         if !is_match {
//             return Err(crate::SqliteError::NotADb);
//         }
//         Ok(Self {
//             data: [0u8; 100],
//             _pad: [0; 12],
//         })
//     }
// }
