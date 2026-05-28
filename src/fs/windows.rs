use std::{cell::UnsafeCell, num::NonZeroUsize};

pub mod prelude {}

thread_local! {
    static CURRENT_DIR_OS_PATH_BUFFER: UnsafeCell<crate::vec::Sqlite3Vec<u16>> = UnsafeCell::new(crate::vec::Sqlite3Vec::with_initial_size(unsafe {NonZeroUsize::new_unchecked(1024)},None));
}
