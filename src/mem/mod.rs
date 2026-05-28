// welcome to my world where we take a 50 year old database engine and make
//it STUPID FAST
#[cfg(target_arch="x86_64")]
pub mod x86_64;
pub use x86_64::prelude::*;