use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use mandelbrot::complex::Complex;
use mandelbrot::mandelbrot::{MandelbrotUniverse, PixelColor};

// Define a simple color for the benchmark
const COLORS: &[PixelColor] = &[PixelColor::WHITE, PixelColor::MAGENTA, PixelColor::BLACK];

// Simple mandelbrot function without optimizations
fn mandelbrot_simple(c: Complex<f64>, max_iter: u32) -> u32 {
    let mut z = Complex::new(0.0, 0.0);
    let mut n = 0;
    while z.norm() <= 4.0 && n < max_iter {
        z = z * z + c;
        n += 1;
    }
    n
}

// Benchmark function without memoization (simulating by clearing cache each time)
fn compute_without_memoization(width: u32, height: u32, iterations: u32) {
    let mut universe = MandelbrotUniverse::new(
        width,
        height,
        COLORS,
        iterations,
        mandelbrot_simple,
    );
    
    // Disable memoization by setting cache size to 0
    universe.set_memo_max_size(0);
    universe.compute();
}

// Benchmark function with memoization
fn compute_with_memoization(width: u32, height: u32, iterations: u32) {
    let mut universe = MandelbrotUniverse::new(
        width,
        height,
        COLORS,
        iterations,
        mandelbrot_simple,
    );
    
    // Use default cache size (10000 entries)
    universe.set_memo_max_size(10000);
    universe.compute();
}

fn memoization_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("memoization");
    
    // Test with different sizes
    let test_cases = vec![
        (100, 100, 25),   // Small test
        (400, 300, 50),   // Medium test
    ];
    
    for &(width, height, iterations) in &test_cases {
        let params = format!("{}x{}x{}", width, height, iterations);
        
        group.bench_with_input(
            BenchmarkId::new("without_memoization", &params),
            &(width, height, iterations),
            |b, &(w, h, i)| b.iter(|| {
                compute_without_memoization(black_box(w), black_box(h), black_box(i))
            }),
        );
        
        group.bench_with_input(
            BenchmarkId::new("with_memoization", &params),
            &(width, height, iterations),
            |b, &(w, h, i)| b.iter(|| {
                compute_with_memoization(black_box(w), black_box(h), black_box(i))
            }),
        );
    }
    
    group.finish();
}

criterion_group!(benches, memoization_benchmark);
criterion_main!(benches);