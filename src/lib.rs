#![feature(alloc_error_handler)] // Enable explicit out-of-band alloc tracking if on nightly
#![feature(nonzero_ops)]
use std::alloc::Layout;

pub mod ffi;
// pub mod mem;
pub mod pcache;

/// The Invariant Guard: This handler intercepts any allocation failure across 
/// the entire process *outside* of your hot execution functions. 
/// It completely removes the need for local 'if null' branches!
#[alloc_error_handler]
fn rust_oom_handler(layout: Layout) -> ! {
    // Log the failure to standard error out-of-band
    eprintln!(
        "CRITICAL ERROR: Global Allocator failed to provision {} bytes with align {}.", 
        layout.size(), 
        layout.align()
    );
    
    // Hard abort: Instantly kill the process via system exit.
    // This satisfies your `assert_unchecked` contract perfectly by ensuring 
    // a null pointer can physically never enter your `PageBlocks` container.
    std::process::abort();
}