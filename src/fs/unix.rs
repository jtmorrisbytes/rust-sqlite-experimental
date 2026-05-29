use std::os::unix::fs::FileExt;
use std::num::{NonZeroU32, NonZeroUsize};
impl super::Sqlite3Vfs {
    pub fn open<P: AsRef<std::path::Path>>(p: P, config: &()) -> Self {
        let path = p.as_ref();
        let _path = path.canonicalize().unwrap();
        let mut options = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .to_owned();
        use fs4::FileExt;
        let file = options.open(_path).unwrap();
        file.try_lock().unwrap();
        
        let mut buffer = [0u8; 2];
        file.read_exact_at(&mut buffer, 16).unwrap();
        let page_size = {
            let n = u16::from_be_bytes(buffer) as u32;
            if n == 0 {
                super::TODO_TEMPORARY_DEFAULT_SECTOR_SIZE_CHANGEME_SOON as u32
            }
            else {
                n as u32
            }
        };
        let page_size = unsafe {NonZeroU32::new_unchecked(page_size)};
        

        Self { db_file: file, page_size }
    }
}
