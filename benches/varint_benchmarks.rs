use criterion::{black_box, criterion_group, criterion_main, Criterion};
// Assuming your crate is named `sqlite`
use sqlite::varint::{VarInt, VarIntBlob}; 

fn bench_varint_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sqlite3r VarInt Serialization");

    // ---- BENCHMARK 1: THE SINGLE-BYTE FAST PATH ----
    group.bench_function("to_blob_v2 (Single-Byte: 42)", |b| {
        let input = VarInt::from_i64(42);
        b.iter(|| {
            // black_box forces the compiler to treat the value as a runtime variable,
            // preventing LLVM from completely deleting our code during optimization passes!
            VarInt::to_blob_v2(black_box(input))
        });
    });

    // ---- BENCHMARK 2: THE 9TH-BYTE HARDWARE BSWAP PATH ----
    group.bench_function("to_blob_v2 (9th-Byte Saturated: i64::MAX)", |b| {
        let input = VarInt::from_i64(i64::MAX);
        b.iter(|| {
            VarInt::to_blob_v2(black_box(input))
        });
    });

    group.bench_function("to_blob_v2 (Two's Complement: -1)", |b| {
        let input = VarInt::from_i64(-1);
        b.iter(|| {
            VarInt::to_blob_v2(black_box(input))
        });
    });

    // ---- BENCHMARK 3: THE DECODER INFLATION CORE ----
    // Pack a pre-baked blob using the oracle layout [191, 255...]
    let saturated_blob = VarInt::to_blob_v2(VarInt::from_i64(i64::MAX));

    group.bench_function("to_varint (Inflation Loop)", |b| {
        b.iter(|| {
            black_box(&saturated_blob).to_varint()
        });
    });

    group.finish();
}

// A realistic production mix: 80% single-byte, 15% 2-byte, 5% 3-byte
fn bench_dynamic_production_mix(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sqlite3r Real-World Workload");
    
    // Pre-bake an array of mixed, realistic database values
    let mut mixed_values = [VarInt::from_i64(0); 100];
    for i in 0..100 {
        mixed_values[i] = match i % 20 {
            0..=15 => VarInt::from_i64(i as i64),         // 80% Single-byte data lengths
            16..=18 => VarInt::from_i64(250 + i as i64),  // 15% 2-byte record counts
            _ => VarInt::from_i64(20000 + i as i64),      // 5% 3-byte page offsets
        };
    }

    group.bench_function("to_blob_v2 (Dynamic Production Mix)", |b| {
        let mut index = 0;
        b.iter(|| {
            // Cycle through the mixed array to prevent the CPU from cheating via static optimization
            let val = mixed_values[index % 100];
            index += 1;
            VarInt::to_blob_v2(black_box(val))
        });
    });
    group.finish();
}




fn bench_varint_serialization2(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sqlite3r VarInt Engine");

    // ---- ENCODER PATHWAYS ----
    group.bench_function("to_blob_v2 (Single-Byte: 42)", |b| {
        let input = VarInt::from_i64(42);
        b.iter(|| {
            VarInt::to_blob_v2(black_box(input))
        });
    });

    group.bench_function("to_blob_v2 (9th-Byte Saturated: i64::MAX)", |b| {
        let input = VarInt::from_i64(i64::MAX);
        b.iter(|| {
            VarInt::to_blob_v2(black_box(input))
        });
    });

    group.bench_function("to_blob_v2 (Two's Complement: -1)", |b| {
        let input = VarInt::from_i64(-1);
        b.iter(|| {
            VarInt::to_blob_v2(black_box(input))
        });
    });


    // ---- DECODER COMPARISON GRID ----
    // Pack a pre-baked real-world oracle blob matching your extracted file footprint [0xBF, 0xFF...]
    let saturated_blob = VarInt::to_blob_v2(VarInt::from_i64(i64::MAX));

    // 1. The Old Fallback Loop Track
    group.bench_function("to_varint_v2 (Loop-Unrolled Fallback Core)", |b| {
        b.iter(|| {
            black_box(&saturated_blob).to_varint_v2()
        });
    });

    // 2. THE CASCADING SIMD MONSTER (Your New Masterpiece)
    group.bench_function("to_varint_unrolled (SIMD Switchboard Cascade)", |b| {
        b.iter(|| {
            black_box(&saturated_blob).to_varint_unrolled()
        });
    });

    group.finish();
}






criterion_group!(benches, bench_varint_serialization,bench_dynamic_production_mix,bench_varint_serialization,bench_varint_serialization2);
criterion_main!(benches);
