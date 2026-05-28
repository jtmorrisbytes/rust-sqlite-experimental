use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sqlite::mem::Sqlite3Vec; // Assumes your crate exposes Sqlite3Vec publically
use std::num::NonZeroUsize;

fn bench_partial_eq(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sqlite3Vec vs Standard Vec (PartialEq)");

    // Test across a variety of dataset byte profiles
    for size_in_bytes in [32, 256, 1024, 8192].iter() {
        let count = *size_in_bytes;

        // --- Setup Data Payloads ---
        // Identical sets
        let src_a = vec![0u8; count];
        let src_b = vec![0u8; count];

        // Mismatched sets (stray bit at the very end of the array to force a full loop run)
        let mut src_mismatch = vec![0u8; count];
        if count > 0 {
            src_mismatch[count - 1] = 1;
        }

        // --- Build Your AVX2 Vectors ---
        let sq_vec_a = Sqlite3Vec::from_slice(&src_a);
        let sq_vec_b = Sqlite3Vec::from_slice(&src_b);
        let sq_vec_mismatch = Sqlite3Vec::from_slice(&src_mismatch);

        // =========================================================================
        // CASE 1: MATCHING IDENTICAL BUFFERS
        // =========================================================================
        group.bench_with_input(
            BenchmarkId::new("Sqlite3Vec (AVX2 Loop) - Match", count),
            &count,

            |b, _| {
                b.iter(|| black_box(&sq_vec_a) == black_box(&sq_vec_b))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Standard Vec (Scalar/Auto-SIMD) - Match", count),
            &count,

            |b, _| {
                b.iter(|| black_box(&src_a) == black_box(&src_b))
            },
        );

        // =========================================================================
        // CASE 2: MISMATCHED BUFFERS (Worst Case: Mismatch on Last Element)
        // =========================================================================
        group.bench_with_input(
            BenchmarkId::new("Sqlite3Vec (AVX2 Loop) - Mismatch End", count),
            &count,

            |b, _| {
                b.iter(|| black_box(&sq_vec_a) == black_box(&sq_vec_mismatch))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Standard Vec (Scalar/Auto-SIMD) - Mismatch End", count),
            &count,

            |b, _| {
                b.iter(|| black_box(&src_a) == black_box(&src_mismatch))
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_partial_eq);
criterion_main!(benches);
