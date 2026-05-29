

use super::Sqlite3Vec;


// impl<T> Sqlite3Vec<T> {
//     #[inline(always)]
//     ///  performs a raw asm vector eq bytewise, ignoring T's partial eq.
//     ///  use when the compiler is stubborn and refuses to optimize the hot loop.
//     /// please note that this does a 'shallow' eq on ptr itself and if you have a complex 
//     /// type it will only eq bytes on the outer container
//     pub fn raw_asm_eq(&self, other: &Self) -> bool {
//         if self.len != other.len {
//             return false;
//         }

//         // We'll let inline asm drive the loop flags directly
//         let mut is_equal: u8;
//         // let _self = unsafe { self.view_as_simd_slice(0) };
//         // let _other = unsafe { other.view_as_simd_slice(0) };
//         // the compiler was being very stubborn here and gept trying to or and shuffle
//         unsafe {
//             core::arch::asm!(
//                 "2:",
//                 // 1. Load 32-bytes from LHS into ymm0
//                 "vmovdqa ymm0, ymmword ptr [{lhs}]",
//                 // 2. XOR with RHS (identical bits become 0)
//                 "vpxor ymm0, ymm0, ymmword ptr [{rhs}]",
//                 // 3. True hardware vptest. Sets ZF flag if ymm0 is entirely 0
//                 "vptest ymm0, ymm0",
//                 // 4. If ZF is 0 (meaning a bit mismatched), jump straight to failure
//                 "jnz 3f",

//                 // 5. Advance pointers by 32 bytes
//                 "add {lhs}, 32",
//                 "add {rhs}, 32",

//                 // 6. Decrement loop counter and repeat if not zero
//                 "dec {count}",
//                 "jnz 2b",

//                 // Success path: set return register to 1
//                 "mov {res}, 1",
//                 "jmp 4f",

//                 // Failure path label
//                 "3:",
//                 "xor {res}, {res}",

//                 // Exit label
//                 "4:",

//                 // Inputs and Outputs
//                 lhs = in(reg) self.ptr.as_ptr(),
//                 rhs = in(reg) other.ptr.as_ptr(),
//                 count = in(reg) self.size.get(),
//                 res = out(reg_byte) is_equal,
//                 // Tell the compiler we are blowing away ymm0 and updating flags
//                 out("ymm0") _,
//                 options(nostack, preserves_flags)
//             );
//         }
//         is_equal != 0
//     }
// }
