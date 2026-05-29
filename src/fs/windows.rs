// use std::os::windows::io::AsRawHandle;
// use std::{
//     cell::UnsafeCell,
//     fs::File,
//     io::{Read, Seek},
//     num::NonZeroUsize,
// };
// use windows_sys::Win32::Foundation::{ERROR_HANDLE_EOF, GetLastError};
// use windows_sys::Win32::{Foundation::HANDLE, Storage::FileSystem::ReadFile, System::IO::OVERLAPPED};

// use crate::vec::Sqlite3Vec;
// use crate::{SqliteError, SqliteResult};
// use crate::fs::Sqlite3Vfs;

// pub mod prelude {}


// // on windows, paths are stored as crate::mem::Sqlite3Vec<u16>
// // to prevent conversion overhead at the cost of some memory

// // Jordan: IM SO EXICTED WE GET TO USE THIS: PLEASE LOOK AT THE ASM FOR THE EQ FUNCTION!
// #[repr(transparent)]
// pub struct Sqlite3Path(crate::vec::Sqlite3Vec<crate::mem::Sqlite3Memory>);
// impl PartialEq for Sqlite3Path {
//     #[inline]
//     fn eq(&self, other: &Self) -> bool {
//         self.0 == other.0
//     }
// }

// impl Sqlite3Path {
//     #[inline]
//     /// copies a rust into utf16 then casts it into the in memory representation
//     pub fn from_str(s: &str) -> Self {
//         let i = s.encode_utf16().chain(std::iter::once(0));
//         Self(Sqlite3Vec::from_iterator(i).cast_into_memory())
//     }
//     /// patses the bytes directly into a Sqlite3Vec thens casts into the in mmeory inplementation
//     pub fn from_encoded_bytes_unchecked(slice:&[u16]) -> Self {
//         let v:Sqlite3Vec<u16> = Sqlite3Vec::from_slice(&slice).unwrap();
//         Self(v.cast_into_memory())
//     }
// }




// thread_local! {
//     static CURRENT_DIR_OS_PATH_BUFFER: UnsafeCell<crate::vec::Sqlite3Vec<u16>> = UnsafeCell::new(crate::vec::Sqlite3Vec::with_initial_size(unsafe {NonZeroUsize::new_unchecked(1024)},None));
// }

// type DWORD = u32;

// // unfortuaneltely the std lib was too slow here and injected too much extra asm
// // so Im talking directly to the os using modern bindings (windows-rs, windows-sys).
// // you should especially check out functions like read_exact_const
// // where when the exact size is known at compile time we can ask the os
// // for that and the llvm compiler backend loves Sized + exact sized at compile time

// impl super::Sqlite3Vfs {
//     #[cfg(target_os = "windows")]
//     pub fn open<P: AsRef<std::path::Path>>(p: P) -> Self {
//         let path = p.as_ref();
//         let _path = path.canonicalize().unwrap();
//         let mut options = std::fs::OpenOptions::new()
//             .read(true)
//             .write(true)
//             .create(true)
//             .to_owned();
//         use std::{num::NonZeroU32, os::windows::fs::OpenOptionsExt};
//         options = options.share_mode(0).to_owned();

//         let file = options.open(_path).unwrap();
//         let metadata = file.metadata().unwrap();
//         println!("{metadata:?}");
//         if metadata.len() == 0 {
//             println!(
//                 "this is a brand new file or its contents have been truncated. we should create a new one"
//             );
//             todo!()
//         }
//         // read the meatadata
//         let buf = [0u8; 100];
//         Self {
//             db_file: file,
//             page_size: NonZeroU32::new(1).unwrap(),
//         }
//     }
//     // this is required because rust tries to inject too much code into the hot path and we need
//     // an exact sized read implementaion anyways
//     #[inline]
//     pub fn read_exact_const<const N: usize, T>(
//         &mut self,
//         input: &mut [T; N],
//         offset: u64,
//     ) -> crate::SqliteResult<()> {
//         // #[rustfmt(skip)]
//         #[rustfmt::skip]
//     const {
//         assert!(N > 0, "VFS Compile Error: Cannot execute a zero-length hardware read!");
//         assert!(std::mem::size_of::<T>() > 0, "VFS Compile Error: Zero-Sized Types (ZSTs) are outlawed!");
//     }
//         // 2. Compute the exact total byte allocation count at compile time
//         // let total_bytes_to_read = (N * std::mem::size_of::<T>()) as u32;
//         // 3. Extract the raw OS handle integer from your exclusive file wrapper
//         let raw_handle =self.db_file.as_raw_handle();
//         // 4. Set up the native Win32 OVERLAPPED layout to target the precise file offset branchlessly
//         let low_offset = (offset & 0xFFFFFFFF) as u32;
//         let high_offset = ((offset >> 32) & 0xFFFFFFFF) as u32;
//         let mut overlapped = OVERLAPPED::default();
//         overlapped.Anonymous.Anonymous.Offset = low_offset;
//         overlapped.Anonymous.Anonymous.OffsetHigh = high_offset;
//         let mut bytes_read: u32 = 0;
//         unsafe {
//             let result = ReadFile(
//                 raw_handle,
//                 input.as_mut_ptr().cast(),
//                 (N * size_of::<T>()) as u32,
//                 &raw mut bytes_read,
//                 &raw mut overlapped,
//             );
//             // todo!();
//             if result == 0 {
//                 let err = GetLastError();
//                 let err = match err {
//                     ERROR_HANDLE_EOF => SqliteError::IoErrShortRead,
//                     _=> {
//                         SqliteError::IoError
//                     }
//                 };
//                 return Err(err);
//             }
//             Ok(())
//         }

//         // todo!()
//     }
// }
// #[cfg_attr(test, test)]
// pub fn sqlite3r_vfs_windows_test_vopen() {
//     let mut vfs = Sqlite3Vfs::open("test.db");
//     let mut input = [0u8; 100];
//     vfs.read_exact_const(&mut input, 0).unwrap();
//     println!("Sqlite3 vfs returned {input:?}");
// }
