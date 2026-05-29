// #[cfg(target_os = "windows")]
// pub mod windows;
// #[cfg(unix)]
// pub mod unix;
// use std::num::{NonZeroU32, NonZeroUsize};
// pub mod sqlite_format_v3;
// pub struct Sqlite3Vfs {
//     db_file: std::fs::File,
//     page_size: NonZeroU32
// }


// I have decided for this modern port of sqlite3 that the entire idea of letting another process
// tocuch this data is inherently risky and doesnt map to &references.
// making file access exclusive means I can apply optimizations only really avaliable at the laguage
// level by treating any accesses to the file as an '&' reference instad of a pointer,
// wich allows the borrow checker to do its job and (hopefully)
// make invalid access impossible though the offical apis


// pub(crate) const TODO_TEMPORARY_DEFAULT_SECTOR_SIZE_CHANGEME_SOON: usize = 4096;
