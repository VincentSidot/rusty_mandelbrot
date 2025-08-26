mod complex;
mod logger;
mod mandelbrot;
mod render;

use std::thread;

use complex::Complex;
use pixels::Error;
use rayon::ThreadPoolBuilder;

use mandelbrot::{MandelbrotUniverse, PixelColor};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const MAX_ITER: u32 = 25;

const COLORS: &[PixelColor] = &[PixelColor::WHITE, PixelColor::MAGENTA, PixelColor::BLACK];

fn mandelbrot(c: Complex<f64>, max_iter: u32) -> u32 {
    let mut z = Complex::new(0.0, 0.0);
    let mut n = 0;
    while z.norm() <= 4.0 && n < max_iter {
        z = z * z + c;
        n += 1;
    }
    n
}

fn mandelbrot_fast(c: Complex<f64>, max_iter: u32) -> u32 {
    // Center check
    if (c.re + 1.0).powi(2) + c.im.powi(2) < 0.0625 {
        return max_iter;
    }

    // Cardoid check
    let p = ((c.re - 0.25).powi(2) + c.im.powi(2)).sqrt();
    if c.re < (p - 2.0 * p.powi(2) + 0.25) {
        return max_iter;
    }

    // Compute
    let mut z = Complex::new(0.0, 0.0);
    let mut n = 0;
    while z.norm() <= 4.0 && n < max_iter {
        z = z * z + c;
        n += 1;
    }
    n
}

fn main() -> Result<(), Error> {
    logger::init(log::LevelFilter::Trace).expect("Failed to initialize logger");

    let threads = thread::available_parallelism()
        .map(|t| t.get())
        .unwrap_or(1);

    // Configure rayon thread pool
    ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .expect("Failed to configure rayon thread pool");

    let mut universe = MandelbrotUniverse::new(WIDTH, HEIGHT, COLORS, MAX_ITER, mandelbrot_fast);
    // Set cache size to 50000 entries
    universe.set_memo_max_size(50000);
    // Enable adaptive resolution
    universe.set_adaptive_resolution(true);
    
    universe.compute();

    // Display memoization statistics
    let (cache_size, hits, misses, hit_rate) = universe.memo_stats();
    println!(
        "Memoization stats: cache_size={}, hits={}, misses={}, hit_rate={:.2}%",
        cache_size, hits, misses, hit_rate
    );

    println!("Running on {} threads", threads);
    println!("Press ESC to exit");
    println!("");

    render::render(universe, WIDTH, HEIGHT)
}
