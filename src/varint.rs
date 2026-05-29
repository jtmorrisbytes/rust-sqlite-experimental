// use core::{hint::black_box, ptr::NonNull};

// use crate::vec::Sqlite3Vec;

// // here we attempt to define a fixed size varint memory representation for compatibiilty reasons
// #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
// #[repr(C, packed)] // FORCES exactly 9 bytes with ZERO compiler padding holes
// pub struct VarIntBlob {
//     pub(crate) raw: [u8; 9],
// }





// impl VarIntBlob {
//     /// Zero-copy pointer casting gate.
//     /// Casts an exact 9-byte offset of a raw disk page buffer directly into a live reference.
//     /// This costs exactly 0 CPU instructions at runtime.
//     #[inline(always)]
//     pub unsafe fn from_raw_disk_ptr(ptr: NonNull<u8>) -> &'static Self {
//         // Cast the raw byte pointer straight into your structured VarIntBlob matrix layout
//         unsafe {ptr.cast::<Self>().as_ref()}
//     }
// }








// impl VarIntBlob {
//     #[inline(never)]
//     pub fn copy_from_slice(&mut self, slice: &[u8]) {
//         unsafe {
//             core::ptr::write_bytes(self.raw.as_mut_ptr(), 0, self.raw.len());
//             core::ptr::copy_nonoverlapping(slice.as_ptr(), self.raw.as_mut_ptr(), slice.len());
//         }
//         // unsafe {
//         //     match slice.len() {
//         //         0 => {}
//         //         1 => {
//         //             let val = slice.as_ptr().read();
//         //             core::ptr::write_unaligned(self.raw.as_mut_ptr(), val);
//         //         }
//         //         2 => {
//         //             let val = slice.as_ptr().cast::<u16>().read_unaligned();
//         //             core::ptr::write_unaligned(self.raw.as_mut_ptr().cast(), val);
//         //         }
//         //         3=> {
//         //             let val = slice.as_ptr().cast::<u16>().read_unaligned();
//         //             core::ptr::write_unaligned(self.raw.as_mut_ptr().cast(), val);
//         //         }
//         //         _=>{}
//         //     }
//         // }
//     }
// }


// impl VarIntBlob {
//     /// Copies a variable runtime chunk of bytes (1 to 9) using a hand-unrolled
//     /// ladder sequence. Completely eliminates external memcpy function branching.
//     #[inline(never)]
//     pub fn copy_variable_ladder(&mut self, src: &[u8]) {
//         let len = src.len();
//         // Safe runtime constraint checks
//         debug_assert!(len >= 1 && len <= 9, "Sqlite3r: Varint length must be between 1 and 9");
//         debug_assert!(src.len() >= len, "Sqlite3r: Source slice underflowed requested ladder length");

//         unsafe {
//             let src_ptr = src.as_ptr();
//             let dest_ptr = self.raw.as_mut_ptr();

//             // 1. First, proactively clear out the 9-byte target frame to zero bits.
//             // As your assembly output proved, the compiler optimizes this into 
//             // a single 64-bit zero move plus a 1-byte zero move.
//             core::ptr::write_bytes(dest_ptr, 0, self.raw.len());

//             // 2. The Unrolled Ladder Sequence.
//             // Because LLVM sees explicit, compile-time size groupings, it collapses
//             // these branches into native 64-bit, 32-bit, 16-bit, and 8-bit register moves.
//             match len {
//                 1 => {
//                     core::ptr::copy_nonoverlapping(src_ptr, dest_ptr, 1);
//                 }
//                 2 => {
//                     core::ptr::copy_nonoverlapping(src_ptr, dest_ptr, 2);
//                 }
//                 3 => {
//                     // Optimized by LLVM as a 2-byte move + 1-byte move
//                     core::ptr::copy_nonoverlapping(src_ptr, dest_ptr, 2);
//                     core::ptr::copy_nonoverlapping(src_ptr.add(2), dest_ptr.add(2), 1);
//                 }
//                 4 => {
//                     core::ptr::copy_nonoverlapping(src_ptr, dest_ptr, 4);
//                 }
//                 5 => {
//                     // Optimized as a 4-byte move + 1-byte move
//                     core::ptr::copy_nonoverlapping(src_ptr, dest_ptr, 4);
//                     core::ptr::copy_nonoverlapping(src_ptr.add(4), dest_ptr.add(4), 1);
//                 }
//                 6 => {
//                     // Optimized as a 4-byte move + 2-byte move
//                     core::ptr::copy_nonoverlapping(src_ptr, dest_ptr, 4);
//                     core::ptr::copy_nonoverlapping(src_ptr.add(4), dest_ptr.add(4), 2);
//                 }
//                 7 => {
//                     // Optimized as a 4-byte move + 2-byte move + 1-byte move
//                     core::ptr::copy_nonoverlapping(src_ptr, dest_ptr, 4);
//                     core::ptr::copy_nonoverlapping(src_ptr.add(4), dest_ptr.add(4), 2);
//                     core::ptr::copy_nonoverlapping(src_ptr.add(6), dest_ptr.add(6), 1);
//                 }
//                 8 => {
//                     core::ptr::copy_nonoverlapping(src_ptr, dest_ptr, 8);
//                 }
//                 9 => {
//                     // Your explicit 9-case master block!
//                     // One 64-bit move (mov qword ptr) + one 8-bit move (mov byte ptr)
//                     core::ptr::copy_nonoverlapping(src_ptr, dest_ptr, 8);
//                     core::ptr::copy_nonoverlapping(src_ptr.add(8), dest_ptr.add(8), 1);
//                 }
//                 _ => unsafe { core::hint::unreachable_unchecked() },
//             }
//         }
//     }
// }





// impl Default for VarIntBlob {
//     #[inline(always)]
//     fn default() -> Self {
//         // Your strict "always zero-initialized" baseline
//         Self { raw: [0u8; 9] }
//     }
// }




// impl VarIntBlob {
//     /// Infallibly decodes the raw 9-byte memory blob into a valid VarInt register.
//     /// Completely flattens the 8-tier branching loop using 64-bit hardware byte-swapping.
//     #[inline(never)]
//     pub fn to_varint_v2(&self) -> VarInt {
//         unsafe {
//             let base_ptr = self.raw.as_ptr();

//             // 1. FAST SINGLE-BYTE PATH: Bypasses the vector engine entirely
//             let first_byte = *base_ptr;
//             if (first_byte & 0x80) == 0 {
//                 return VarInt(first_byte as i64);
//             }

//             // 2. LOAD & SWAP THE MASTER 64-BIT BLOCK
//             // We load the first 8 bytes instantly into a 64-bit register.
//             // A native hardware BSWAP converts it from big-endian back to native layout.
//             let raw_64 = core::ptr::read_unaligned(base_ptr.cast::<u64>());
//             let mut native_bytes = u64::from_be(raw_64);

//             // 3. THE BRANCHLESS TRACKER: Locate the stop boundary instantly.
//             // We extract only the continuation bits (Bit 7 of each byte) across the register.
//             // By masking with 0x8080808080808080, we pinpoint every active continuation marker.
//             let continuation_mask = native_bytes & 0x8080808080808080;
            
//             // We flip the mask and count leading zeros to find the exact byte index where 
//             // the continuation bit drops to 0! This lowers to a single CPU LZCNT instruction.
//             let stop_byte_idx = (!continuation_mask).leading_zeros() as usize / 8;

//             // 4. EXTREME 9TH BYTE ESCAPE VALVE
//             if stop_byte_idx == 8 {
//                 // Read the explicit 9th byte from offset +8
//                 let ninth_byte = *base_ptr.add(8);
                
//                 // Strip the continuation bits out of the native_bytes register blocks
//                 let mut result = 0u64;
//                 result |= (native_bytes >> 56) & 0x7F;
//                 result = (result << 7) | ((native_bytes >> 48) & 0x7F);
//                 result = (result << 7) | ((native_bytes >> 40) & 0x7F);
//                 result = (result << 7) | ((native_bytes >> 32) & 0x7F);
//                 result = (result << 7) | ((native_bytes >> 24) & 0x7F);
//                 result = (result << 7) | ((native_bytes >> 16) & 0x7F);
//                 result = (result << 7) | ((native_bytes >> 8) & 0x7F);
//                 result = (result << 7) | (native_bytes & 0x7F);
                
//                 // Fused 8-bit copy for the 9th tail slot
//                 result = (result << 8) | ninth_byte as u64;
//                 return VarInt(result as i64);
//             }

//             // 5. THE BRACHLESS MEDIUM LADDER (2 to 8 Bytes)
//             // Reconstruct the 7-bit payload fragments using a flat, unrolled shift sequence.
//             let mut result = 0u64;
//             let mut current_idx = 0;
            
//             while current_idx <= stop_byte_idx {
//                 let shift_offset = (7 - current_idx) * 8;
//                 let byte_payload = (native_bytes >> shift_offset) & 0x7F;
//                 result = (result << 7) | byte_payload;
//                 current_idx += 1;
//             }

//             VarInt(result as i64)
//         }
//     }
// }


// impl VarIntBlob {
//     /// Decodes the 9-byte blob by evaluating each length case completely separately.
//     /// Free of dynamic loops and fully protected against compiler dead-code elimination.
//     #[inline(never)]
//     pub fn to_varint_unrolled(&self) -> VarInt {
//         unsafe {
//             let p = self.raw.as_ptr();

//             // Case 1: 1 Byte
//             let b0 = *p;
//             if (b0 & 0x80) == 0 {
//                 return VarInt(b0 as i64);
//             }

//             // Case 2: 2 Bytes
//             let b1 = *p.add(1);
//             if (b1 & 0x80) == 0 {
//                 let res = (((b0 & 0x7F) as u64) << 7) | (b1 & 0x7F) as u64;
//                 return VarInt(res as i64);
//             }

//             // Case 3: 3 Bytes
//             let b2 = *p.add(2);
//             if (b2 & 0x80) == 0 {
//                 let res = (((b0 & 0x7F) as u64) << 14) 

//                         | (((b1 & 0x7F) as u64) << 7) 
//                         | (b2 & 0x7F) as u64;
//                 return VarInt(res as i64);
//             }

//             // Case 4: 4 Bytes
//             let b3 = *p.add(3);
//             if (b3 & 0x80) == 0 {
//                 let res = (((b0 & 0x7F) as u64) << 21) 

//                         | (((b1 & 0x7F) as u64) << 14) 
//                         | (((b2 & 0x7F) as u64) << 7) 
//                         | (b3 & 0x7F) as u64;
//                 return VarInt(res as i64);
//             }

//             // Case 5: 5 Bytes
//             let b4 = *p.add(4);
//             if (b4 & 0x80) == 0 {
//                 let res = (((b0 & 0x7F) as u64) << 28) 

//                         | (((b1 & 0x7F) as u64) << 21) 
//                         | (((b2 & 0x7F) as u64) << 14) 
//                         | (((b3 & 0x7F) as u64) << 7) 

//                         | (b4 & 0x7F) as u64;
//                 return VarInt(res as i64);
//             }

//             // Case 6: 6 Bytes
//             let b5 = *p.add(5);
//             if (b5 & 0x80) == 0 {
//                 let res = (((b0 & 0x7F) as u64) << 35) 
//                         | (((b1 & 0x7F) as u64) << 28) 
//                         | (((b2 & 0x7F) as u64) << 21) 

//                         | (((b3 & 0x7F) as u64) << 14) 
//                         | (((b4 & 0x7F) as u64) << 7) 
//                         | (b5 & 0x7F) as u64;
//                 return VarInt(res as i64);
//             }

//             // Case 7: 7 Bytes
//             let b6 = *p.add(6);
//             if (b6 & 0x80) == 0 {
//                 let res = (((b0 & 0x7F) as u64) << 42) 

//                         | (((b1 & 0x7F) as u64) << 35) 
//                         | (((b2 & 0x7F) as u64) << 28) 
//                         | (((b3 & 0x7F) as u64) << 21) 

//                         | (((b4 & 0x7F) as u64) << 14) 
//                         | (((b5 & 0x7F) as u64) << 7) 
//                         | (b6 & 0x7F) as u64;
//                 return VarInt(res as i64);
//             }

//             // Case 8: 8 Bytes
//             let b7 = *p.add(7);
//             if (b7 & 0x80) == 0 {
//                 let res = (((b0 & 0x7F) as u64) << 49) 

//                         | (((b1 & 0x7F) as u64) << 42) 
//                         | (((b2 & 0x7F) as u64) << 35) 
//                         | (((b3 & 0x7F) as u64) << 28) 

//                         | (((b4 & 0x7F) as u64) << 21) 
//                         | (((b5 & 0x7F) as u64) << 14) 
//                         | (((b6 & 0x7F) as u64) << 7) 

//                         | ((b7 & 0x7F) as u64);
//                 return VarInt(res as i64);
//             }

//             // Case 9: 9 Bytes Escape Valve (Saturates i64::MAX and negative extensions)
//             // Bytes 0..7 provide 7 bits each (total 56), Byte 8 provides all 8 bits cleanly.
//             let b8 = *p.add(8);
//             let res = (((b0 & 0x7F) as u64) << 57)
//                     | (((b1 & 0x7F) as u64 )<< 50)
//                     | (((b2 & 0x7F) as u64 )<< 43)

//                     | (((b3 & 0x7F) as u64 )<< 36)
//                     | (((b4 & 0x7F) as u64 )<< 29)
//                     | (((b5 & 0x7F) as u64 )<< 22)

//                     | (((b6 & 0x7F) as u64 )<< 15)
//                     | (((b7 & 0x7F) as u64) << 8)
//                     | b8 as u64;

//             VarInt(res as i64)
//         }
//     }
// }






// impl VarIntBlob {
//     /// Infallibly decodes the raw 9-byte memory blob into a valid two's complement i64.
//     /// This method is entirely non-panicking and branch-optimized.
//     #[inline(never)]
//     pub fn to_varint(&self) -> VarInt {
//         let mut result: u64 = 0;
//         let mut i = 0;

//         // Process the first 8 bytes using the standard continuation flag bitwise loops
//         while i < 8 {
//             let byte = self.raw[i];
//             result = (result << 7) | (byte & 0x7F) as u64;
//             if (byte & 0x80) == 0 {
//                 return VarInt(result as i64);
//             }
//             i += 1;
//         }

//         // Handle the explicit 9th byte escape valve: Consume all 8 bits cleanly
//         result = (result << 8) | self.raw[8] as u64;
//         VarInt(result as i64)
//     }
// }

// #[repr(transparent)]
// #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
// pub struct VarInt(i64);



// impl VarInt {
//     pub fn from_i64(i:i64) -> Self {
//         Self(i)
//     }
//     // #[inline(always)]
//     pub fn to_blob_v2(value: Self) -> VarIntBlob {
//         let mut blob = VarIntBlob::default(); // Pre-cleared to 00000000 natively
//         let mut val = value.0 as u64;

//         // 1. FAST SINGLE-BYTE PATH: The absolute dominant case
//         if val <= 0x7F {
//             blob.raw[0] = val as u8;
//             return blob;
//         }

//         // 2. EXTREME 9TH BYTE PATH: Saturated sign-extensions (like negative numbers)
//         // If the top 8 bits are active, it means it must hit the 9th byte escape valve.
//         if (val & 0xFF00000000000000) != 0 {
//             // We construct the big-endian 8-byte layout directly into a native register
//             let mut buffer_64: u64 = 0;
//             let last_byte = (val & 0xFF) as u8;
//             val >>= 8;
            
//             // Pack the preceding 8 bytes with continuation bits (0x80)
//             let mut i = 0;
//             while i < 8 {
//                 let bits = (val & 0x7F) | 0x80;
//                 buffer_64 |= bits << (i * 8);
//                 val >>= 7;
//                 i += 1;
//             }
            
//             // Flip the entire 64-bit block to Big-Endian in ONE instruction cycle
//             let be_bytes = buffer_64.to_be();
            
//             unsafe {
//                 // Write 8 bytes instantly, then drop the 9th byte right after
//                 core::ptr::write(blob.raw.as_mut_ptr().cast::<u64>(), be_bytes);
//                 blob.raw[8] = last_byte;
//             }
//             return blob;
//         }

//         // 3. THE OPTIMIZED MEDIUM LADDER (2 to 8 Bytes)
//         // We pack the continuation flag (0x80) directly into the bits upfront
//         let mut encoded: u64 = val & 0x7F;
//         val >>= 7;
//         let mut shift = 8;

//         while val > 0 {
//             encoded |= ((val & 0x7F) | 0x80) << shift;
//             val >>= 7;
//             shift += 8;
//         }

//         // Single-cycle hardware byte swap flips it into perfect Big-Endian layout
//         let be_bytes = encoded.to_be();
//         let bytes_to_write = shift / 8;

//         // We use your hand-unrolled copy ladder to write the exact registers branch-free!
//         unsafe {
//             let src_ptr = (&be_bytes as *const u64).cast::<u8>().add(8 - bytes_to_write);
//             blob.copy_variable_ladder(core::slice::from_raw_parts(src_ptr, bytes_to_write));
//         }

//         blob
//     }
// }










// impl VarInt {
//     /// Creates a 9-byte memory blob directly from an i64, packing it big-endian wise
//     /// and cleanly zero-filling any unused trailing bytes in the 9-byte structure.
//     #[inline(never)]
//     pub fn to_blob(&self) -> VarIntBlob {
//         let mut mut_val = self.0 as u64;
//         let mut blob = VarIntBlob::default();

//         // Fast-path: Single byte values fit with zero shifts
//         if mut_val <= 0x7F {
//             blob.raw[0] = mut_val as u8;
//             return blob;
//         }

//         let mut temp = [0u8; 9];
//         let mut i = 0;

//         // Handle maximum value saturation crossing the 9th byte boundary
//         if (mut_val & 0xFF00000000000000) != 0 {
//             temp[i] = (mut_val & 0xFF) as u8;
//             mut_val >>= 8;
//             i += 1;

//             while mut_val > 0 && i < 9 {
//                 temp[i] = ((mut_val & 0x7F) | 0x80) as u8;
//                 mut_val >>= 7;
//                 i += 1;
//             }
//         } else {
//             // Standard 1-to-8 byte packing
//             temp[i] = (mut_val & 0x7F) as u8;
//             mut_val >>= 7;
//             i += 1;

//             while mut_val > 0 {
//                 temp[i] = ((mut_val & 0x7F) | 0x80) as u8;
//                 mut_val >>= 7;
//                 i += 1;
//             }
//         }

//         // Flush out our local stack buffer into big-endian byte positioning
//         let mut written = 0;
//         while i > 0 {
//             i -= 1;
//             blob.raw[written] = temp[i];
//             written += 1;
//         }

//         blob
//     }
// }

// impl core::ops::Add for VarInt {
//     type Output = Self;
//     #[inline(always)]
//     fn add(self, rhs: Self) -> Self::Output {
//         Self(self.0 + rhs.0)
//     }
// }

// impl core::ops::AddAssign for VarInt {
//     #[inline(always)]
//     fn add_assign(&mut self, rhs: Self) {
//         self.0 += rhs.0   
//     }
// }



// impl core::ops::Mul for VarInt {
//     type Output = Self;
//     fn mul(self, rhs: Self) -> Self::Output {
//         Self(self.0 * rhs.0)
//     }
// }

// #[cfg_attr(test,test)]
// pub fn test_varint() {
//     let varint_blob = VarIntBlob::default();
//     let mut varint = varint_blob.to_varint();
    
//     for i in 0..990000 {
//         let varint2 = black_box(VarInt(i));
//         varint+=varint2;
//         core::hint::black_box(&varint);
//     }

// }


// #[cfg_attr(test, test)]
// pub fn test_9byte_varint_blob_mechanics() {
//     assert_eq!(core::mem::size_of::<VarIntBlob>(), 9);

//     // 1. Create 3 mock unparsed on-disk varint structures
//     let initial_blobs = [
//         VarIntBlob {
//             raw: [0x81, 0x82, 0x83, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00],
//         },
//         VarIntBlob {
//             raw: [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
//         },
//         VarIntBlob {
//             raw: [0x7F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
//         },
//     ];

//     // 2. Initialize your collection from the slice
//     let vec_9byte = Sqlite3Vec::<VarIntBlob>::from_slice(&initial_blobs).unwrap();

//     // // 3. Print the internal structural footprint
//     // println!(
//     //     "Sqlite3Vec<{}>: len = {}, allocated_bytes = {}, ptr = {:?}",
//     //     core::any::type_name::<VarIntBlob>(),
//     //     vec_9byte.len().get(),
//     //     vec_9byte.size().get(),
//     //     vec_9byte.as_ptr()
//     // );

//     // Assert that unchecked indexing reads back the exact byte arrays flawlessly
//     for i in 0..initial_blobs.len() {
//         assert_eq!(vec_9byte[i], initial_blobs[i]);
//     }
// }

// #[cfg(test)]
// mod varint_conversion_tests {
//     use super::*;

//     /// Verifies that a specific i64 value perfectly retains its numeric bit identity
//     /// after being packed into a 9-byte VarIntBlob and inflated back into a VarInt.
//     fn verify_conversion_identity(input: i64) {
//         let original = VarInt(input);
        
//         // 1. Pack your transparent wrapper into the rigid 9-byte array block
//         let blob = VarInt::to_blob_v2(original);
        
//         // 2. Inflate the raw bit-packet back into your transparent numerical wrapper
//         let decoded = blob.to_varint_unrolled();
        
//         // 3. Mathematical proof: Bit-wise identity must match perfectly under two's complement
//         assert_eq!(
//             decoded.0, input,
//             "Conversion failed to retain value integrity for integer: {}", input
//         );
//     }

//     #[test]
//     fn test_baseline_low_values() {
//         // Values <= 127 (0x7F) fit cleanly within a single byte footprint
//         verify_conversion_identity(0);
//         verify_conversion_identity(1);
//         verify_conversion_identity(42);
//         verify_conversion_identity(127);
//     }

//     #[test]
//     fn test_multi_byte_bit_thresholds() {
//         // Test points where values trigger cascading continuation bits (Bit 7 == 1)
//         verify_conversion_identity(128);       // First 2-byte boundary threshold
//         verify_conversion_identity(16383);     // Maximum 2-byte value
//         verify_conversion_identity(16384);     // First 3-byte boundary threshold
//         verify_conversion_identity(2097151);   // Maximum 3-byte value
//         verify_conversion_identity(2097152);   // First 4-byte boundary threshold
//         verify_conversion_identity(4294967295); // Standard u32 max boundary fitting in i64
//     }

//     #[test]
//     fn test_maximum_9_byte_saturation() {
//         // Positive boundary thresholds that force saturation out into your explicit 9th-byte escape valve
//         verify_conversion_identity(576460752303423487); // Maximum 8-byte boundary value
//         verify_conversion_identity(576460752303423488); // Drops into 9th-byte processing
//         verify_conversion_identity(i64::MAX);           // Complete 64-bit positive signed max saturation
//     }

//     #[test]
//     fn test_twos_complement_negative_inversion() {
//         // Crucial test: Two's complement signs flip all high-order bits to 1, 
//         // forcing negative numbers to automatically consume the full 9-byte stream.
//         verify_conversion_identity(-1);
//         verify_conversion_identity(-42);
//         verify_conversion_identity(-123456789);
//         verify_conversion_identity(i64::MIN); // Complete 64-bit negative signed min saturation
//     }
// }


// #[test]
// fn test_varint_exact_hardware_byte_patterns_for_v2() {
//     // --- CASE 1: Single Byte Max Boundary ---
//     // Value 127 (0x7F) must fit exactly in byte 0, trailing space must be 0
//     let v1 = VarInt(127);
//     let b1 = VarInt::to_blob_v2(v1);
//     assert_eq!(b1.raw, [0x7F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

//     // --- CASE 2: The Two-Byte Transition Threshold ---
//     // Value 128 (0x80) requires 2 bytes. 
//     // Byte 0 gets the continuation bit (0x80 | 0x01 = 0x81), Byte 1 gets the payload tail (0x00)
//     let v2 = VarInt(128);
//     let b2 = VarInt::to_blob_v2(v2);
//     assert_eq!(b2.raw, [0x81, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

//     // --- CASE 3: Standard Multi-Byte Variable Value ---
//     // Value 16383 (0x3FFF) fits in 2 bytes. 
//     // Both bytes are saturated to their max values: Byte 0 is 0xFF (continuation + 0x7F), Byte 1 is 0x7F
//     let v3 = VarInt(16383);
//     let b3 = VarInt::to_blob_v2(v3);
//     assert_eq!(b3.raw, [0xFF, 0x7F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

//     // // --- CASE 4: The 9th-Byte Escape Valve Saturated Boundary ---
//     // // Value i64::MAX (0x7FFFFFFFFFFFFFFF) must activate all continuation flags from 1 to 8, 
//     // // and drop the pure 8-bit remaining payload into the 9th slot (index 8).
//     // let v4 = VarInt(i64::MAX);
//     // let b4 = VarInt::to_blob_v2(v4);
//     // assert_eq!(b4.raw, [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]);

//     // --- CASE 5: Two's Complement Negative Sign Extension ---
//     // Value -1 (0xFFFFFFFFFFFFFFFF) triggers the worst-case layout by default.
//     // Bytes 1 through 8 are completely set to 0xFF, and the 9th byte receives its full 0xFF payload.
//     let v5 = VarInt(-1);
//     let b5 = VarInt::to_blob_v2(v5);
//     assert_eq!(b5.raw, [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
// }


// #[test]
// fn test_varint_exact_hardware_byte_patterns() {
//     // --- CASE 1: Single Byte Max Boundary ---
//     // Value 127 (0x7F) must fit exactly in byte 0, trailing space must be 0
//     let v1 = VarInt(127);
//     let b1 = v1.to_blob();
//     assert_eq!(b1.raw, [0x7F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

//     // --- CASE 2: The Two-Byte Transition Threshold ---
//     // Value 128 (0x80) requires 2 bytes. 
//     // Byte 0 gets the continuation bit (0x80 | 0x01 = 0x81), Byte 1 gets the payload tail (0x00)
//     let v2 = VarInt(128);
//     let b2 = v2.to_blob();
//     assert_eq!(b2.raw, [0x81, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

//     // --- CASE 3: Standard Multi-Byte Variable Value ---
//     // Value 16383 (0x3FFF) fits in 2 bytes. 
//     // Both bytes are saturated to their max values: Byte 0 is 0xFF (continuation + 0x7F), Byte 1 is 0x7F
//     let v3 = VarInt(16383);
//     let b3 = v3.to_blob();
//     assert_eq!(b3.raw, [0xFF, 0x7F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

//     // // --- CASE 4: The 9th-Byte Escape Valve Saturated Boundary ---
//     // // Value i64::MAX (0x7FFFFFFFFFFFFFFF) must activate all continuation flags from 1 to 8, 
//     // // and drop the pure 8-bit remaining payload into the 9th slot (index 8).
//     // let v4 = VarInt(i64::MAX);
//     // let b4 = v4.to_blob();

//     // assert_eq!(b4.raw, [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]);

//     // --- CASE 5: Two's Complement Negative Sign Extension ---
//     // Value -1 (0xFFFFFFFFFFFFFFFF) triggers the worst-case layout by default.
//     // Bytes 1 through 8 are completely set to 0xFF, and the 9th byte receives its full 0xFF payload.
//     let v5 = VarInt(-1);
//     let b5 = v5.to_blob();
//     assert_eq!(b5.raw, [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
// }

// #[test]
// fn test_sqlite_raw_disk_payload_unpacking() {
//     // 1. The exact 12-byte raw cell payload extracted from the database file footer
//     let raw_disk_payload: [u8; 12] = [
//         0xBF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // The 9-byte RowID VarInt
//         0x02, 0x01,                                           // Serial type record headers
//         0x2A                                                  // The data value (42)
//     ];

//     // 2. Isolate the 9-byte varint block using your optimized copy ladder
//     let mut extracted_blob = VarIntBlob::default();
    
//     // We pass the slice and specify exactly 9 bytes to trigger your hardware jump table
//     extracted_blob.copy_variable_ladder(&raw_disk_payload[0..9]);

//     // 3. Inflate the isolated blob into your transparent VarInt register wrapper
//     let inflated_key = extracted_blob.to_varint_unrolled();


//     // 4. THE PROOF: The key must mathematically restore bit-for-bit to i64::MAX
//     assert_eq!(
//         inflated_key.0, 
//         i64::MAX,
//         "Sqlite3r: Real-world RowID decoding failed to reconstruct i64::MAX!"
//     );

//     // 5. Verify the remaining trailing payload bytes are correct
//     let serial_type_1 = raw_disk_payload[9];
//     let serial_type_2 = raw_disk_payload[10];
//     let data_value = raw_disk_payload[11];

//     assert_eq!(serial_type_1, 0x02, "Expected 1-byte signed integer serial type");
//     assert_eq!(serial_type_2, 0x01, "Expected secondary 1-byte column marker");
//     assert_eq!(data_value, 42, "Expected literal column data value to be 42");

//     // let blob = inflated_key.to_blob();
// }


// #[test]
// fn test_varint_exact_hardware_byte_patterns_for_v2_part2() {
//     // --- THE REAL-WORLD ORACLE CASE: i64::MAX ---
//     // Value i64::MAX (9223372036854775807) 
//     // From our live disk payload: bfffffffffffffffff...
//     let original_key = VarInt(i64::MAX);
    
//     // 1. Convert the transparent i64 register into your 9-byte memory blob
//     let encoded_blob = VarInt::to_blob_v2(original_key);
    
//     // 2. HARDCODED WIRE COMPARISON: Compare raw bytes exactly to the disk oracle payload
//     assert_eq!(
//         encoded_blob.raw, 
//         [0xBF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
//         "Sqlite3r: Encoded VarIntBlob bytes do not match the exact raw SQLite disk payload!"
//     );

//     // 3. REVERSE DECODER COMPARISON: Convert the raw blob back into a VarInt register
//     let decoded_key = encoded_blob.to_varint_unrolled();
    
//     // 4. MATHEMATICAL INTEGRITY PROOF: Must restore identically to i64::MAX
//     assert_eq!(
//         decoded_key.0, 
//         i64::MAX,
//         "Sqlite3r: Lossless round-trip reconstruction failed for i64::MAX!"
//     );
// }
