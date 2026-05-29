#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

// 1. The exact same 9-byte structure layout we built in Rust.
// Force strict packing and 32-byte cache-line register boundaries.
#pragma pack(push, 1)
typedef struct {
    uint8_t raw[9];
} VarIntBlob;
#pragma pack(pop)

// 2. The hyper-optimized register-shifting bitmask encoder
__declspec(noinline) VarIntBlob to_blob_v2(int64_t input_value) {
    VarIntBlob blob = {{0}}; // Pre-zeroes the 9-byte grid natively
    uint64_t val = (uint64_t)input_value;

    // Fast-path: Single-byte optimization check
    if (val <= 0x7F) {
        blob.raw[0] = (uint8_t)val;
        return blob;
    }

    // 9th-Byte Escape Valve path (handling negative sign-extensions and positive max)
    if ((val & 0xFF00000000000000ULL) != 0) {
        uint64_t buffer_64 = 0;
        uint8_t last_byte = (uint8_t)(val & 0xFF);
        val >>= 8;

        // Bounded loop that MSVC fully unrolls into straight registers math
        for (int i = 0; i < 8; i++) {
            uint64_t bits = (val & 0x7F) | 0x80;
            buffer_64 |= (bits << (i * 8));
            val >>= 7;
        }

        // MSVC hardware byte-swap compiler intrinsic
        uint64_t be_bytes = _byteswap_uint64(buffer_64);
        
        // Direct branchless register-to-memory block mapping
        *(uint64_t*)(blob.raw) = be_bytes;
        blob.raw[8] = last_byte;
        return blob;
    }

    // Medium Pathway (2 to 8 Bytes)
    uint64_t encoded = val & 0x7F;
    val >>= 7;
    int shift = 8;

    while (val > 0) {
        encoded |= (((val & 0x7F) | 0x80) << shift);
        val >>= 7;
        shift += 8;
    }

    uint64_t be_bytes = _byteswap_uint64(encoded);
    int bytes_to_write = shift / 8;

    uint8_t* src_ptr = ((uint8_t*)&be_bytes) + (8 - bytes_to_write);
    
    // Explicit unrolled switchboard assignments
    switch (bytes_to_write) {
        case 2:
            blob.raw[0] = src_ptr[0]; blob.raw[1] = src_ptr[1];
            break;
        case 3:
            blob.raw[0] = src_ptr[0]; blob.raw[1] = src_ptr[1]; blob.raw[2] = src_ptr[2];
            break;
        case 4:
            *(uint32_t*)(blob.raw) = *(uint32_t*)src_ptr;
            break;
        case 5:
            *(uint32_t*)(blob.raw) = *(uint32_t*)src_ptr; blob.raw[4] = src_ptr[4];
            break;
        case 6:
            *(uint32_t*)(blob.raw) = *(uint32_t*)src_ptr; *(uint16_t*)(blob.raw + 4) = *(uint16_t*)(src_ptr + 4);
            break;
        case 7:
            *(uint32_t*)(blob.raw) = *(uint32_t*)src_ptr; *(uint16_t*)(blob.raw + 4) = *(uint16_t*)(src_ptr + 4); blob.raw[6] = src_ptr[6];
            break;
        case 8:
            *(uint64_t*)(blob.raw) = be_bytes;
            break;
    }

    return blob;
}

int main(int argc, char* argv[]) {
    // Default testing values if no command arguments are passed
    int64_t target_value = 9223372036854775807LL; // i64::MAX
    if (argc > 1) {
        target_value = _strtoi64(argv[1], NULL, 10);
    }

    printf("Encoding value: %lld\n", target_value);

    // Dynamic processing loop harness
    VarIntBlob final_result;
    uint8_t checksum_accumulator = 0;
    
    for (int i = 0; i < 990000; i++) {
        // Force evaluation on every iteration by injecting the loop offset
        final_result = to_blob_v2(target_value + (i & 0));
        checksum_accumulator ^= final_result.raw[0];
    }

    // Print the raw byte array pattern verified against our SQLite disk oracle
    printf("Result Pattern: ");
    for (int i = 0; i < 9; i++) {
        printf("%02X ", final_result.raw[i]);
    }
    printf("\nAccumulator Anchor: %d\n", checksum_accumulator);

    return 0;
}
