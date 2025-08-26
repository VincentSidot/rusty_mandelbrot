mod complex;
mod logger;
mod mandelbrot;
mod render;

use std::thread;

use complex::Complex;
use pixels::Error;
use rayon::ThreadPoolBuilder;

use logger::{LoggerConfig, init};
use mandelbrot::{MandelbrotUniverse, PixelColor};

const WIDTH: u32 = 800;
use std::time::Instant;

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
    // Initialize logger with file output
    let config = LoggerConfig {
        level: log::LevelFilter::Trace,
        log_to_file: Some("mandelbrot.log".to_string()),
        log_to_console: true,
    };
    
    init(config).expect("Failed to initialize logger");
    log::info!("Application started");

    let threads = thread::available_parallelism()
        .map(|t| t.get())
        .unwrap_or(1);

    log::info!("Detected {} available threads", threads);

    // Configure rayon thread pool
    ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .expect("Failed to configure rayon thread pool");
    log::info!("Rayon thread pool configured with {} threads", threads);

    let start_time = Instant::now();
    log::info!("Creating Mandelbrot universe with dimensions {}x{}", WIDTH, HEIGHT);
    
    let mut universe = MandelbrotUniverse::new(WIDTH, HEIGHT, COLORS, MAX_ITER, mandelbrot_fast);
    // Set cache size to 50000 entries
    universe.set_memo_max_size(50000);
    log::debug!("Memoization cache size set to 50000 entries");
    
    // Enable adaptive resolution
    universe.set_adaptive_resolution(true);
    log::debug!("Adaptive resolution enabled");
    
    log::info!("Starting computation...");
    universe.compute();

    let duration = start_time.elapsed();
    log::info!("Computation completed in {:?}", duration);

    // Display memoization statistics
    let (cache_size, hits, misses, hit_rate) = universe.memo_stats();
    log::info!(
        "Memoization stats: cache_size={}, hits={}, misses={}, hit_rate={:.2}%",
        cache_size, hits, misses, hit_rate
    );

    log::info!("Running on {} threads", threads);
    println!("Press ESC to exit");
    println!("");

    render::render(universe, WIDTH, HEIGHT)
}
