use std::os::raw::{c_int, c_void};

// Mirror SQLite's internal sqlite3_pcache_methods2 structural definition exactly
#[repr(C)]
pub struct SqlitePcacheMethods2 {
    pub i_version: c_int,
    pub p_arg: *mut c_void,
    pub x_init: Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
    pub x_shutdown: Option<unsafe extern "C" fn(*mut c_void)>,
    pub x_create: Option<unsafe extern "C" fn(c_int, c_int, c_int) -> *mut c_void>,
    pub x_cachesize: Option<unsafe extern "C" fn(*mut c_void, c_int) -> c_int>,
    pub x_pagecount: Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
    pub x_fetch: Option<unsafe extern "C" fn(*mut c_void, u32, c_int) -> *mut c_void>,
    pub x_unpin: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, c_int)>,
    pub x_rekey: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, u32, u32)>,
    pub x_truncate: Option<unsafe extern "C" fn(*mut c_void, u32)>,
    pub x_destroy: Option<unsafe extern "C" fn(*mut c_void)>,
}