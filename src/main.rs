mod complex;
mod logger;
mod mandelbrot;
mod render;
#[cfg(feature = "gpu")]
mod gpu;

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

#[cfg(feature = "gpu")]
fn benchmark_gpu() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Starting GPU benchmark");
    let start_time = Instant::now();
    
    let mut universe = MandelbrotUniverse::new(WIDTH, HEIGHT, COLORS, MAX_ITER, mandelbrot_fast);
    universe.set_memo_max_size(50000);
    universe.set_adaptive_resolution(true);
    
    match gpu::compute_mandelbrot_gpu(WIDTH, HEIGHT, MAX_ITER, universe.view()) {
        Ok(iterations) => {
            // Convert iteration counts to colors
            let data = universe.data_mut();
            for (i, &iter_count) in iterations.iter().enumerate() {
                if i < data.len() {
                    let color_idx = iter_count.min(MAX_ITER);
                    data[i] = COLORS[color_idx as usize % COLORS.len()];
                }
            }
            
            let duration = start_time.elapsed();
            log::info!("GPU benchmark completed in {:?}", duration);
            Ok(())
        }
        Err(e) => {
            log::error!("GPU benchmark failed: {}", e);
            Err(e)
        }
    }
}

fn benchmark_cpu() {
    log::info!("Starting CPU benchmark");
    let start_time = Instant::now();
    
    let mut universe = MandelbrotUniverse::new(WIDTH, HEIGHT, COLORS, MAX_ITER, mandelbrot_fast);
    universe.set_memo_max_size(50000);
    universe.set_adaptive_resolution(true);
    universe.compute();
    
    let duration = start_time.elapsed();
    log::info!("CPU benchmark completed in {:?}", duration);
}

fn print_usage() {
    println!("Usage: mandelbrot [OPTIONS]");
    println!("");
    println!("Options:");
    println!("  --benchmark-cpu    Run CPU benchmark");
    #[cfg(feature = "gpu")]
    println!("  --benchmark-gpu    Run GPU benchmark (requires --features gpu)");
    println!("  --help             Display this help message");
    println!("");
    println!("If no options are provided, the application will run in interactive mode.");
}

fn main() -> Result<(), Error> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "--benchmark-cpu" => {
                // Initialize logger for benchmark
                let config = LoggerConfig {
                    level: log::LevelFilter::Info,
                    log_to_file: Some("benchmark.log".to_string()),
                    log_to_console: true,
                };
                
                init(config).expect("Failed to initialize logger");
                log::info!("Starting CPU benchmark");
                
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
                
                benchmark_cpu();
                return Ok(());
            }
            #[cfg(feature = "gpu")]
            "--benchmark-gpu" => {
                // Initialize logger for benchmark
                let config = LoggerConfig {
                    level: log::LevelFilter::Info,
                    log_to_file: Some("benchmark.log".to_string()),
                    log_to_console: true,
                };
                
                init(config).expect("Failed to initialize logger");
                log::info!("Starting GPU benchmark");
                
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
                
                // Run GPU benchmark
                match benchmark_gpu() {
                    Ok(()) => log::info!("GPU benchmark completed successfully"),
                    Err(e) => log::error!("GPU benchmark failed: {}", e),
                }
                return Ok(());
            }
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
            _ => {
                println!("Unknown option: {}", args[1]);
                print_usage();
                return Ok(());
            }
        }
    }
    
    // Interactive mode (default behavior)
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
    
    // Use GPU computation if the feature is enabled
    #[cfg(feature = "gpu")]
    {
        log::info!("Using GPU computation");
        let start_time = Instant::now();
        let max_iter = universe.max_iter();
        match gpu::compute_mandelbrot_gpu(universe.width(), universe.height(), max_iter, universe.view()) {
            Ok(iterations) => {
                // Convert iteration counts to colors using the gradient table
                let data = universe.data_mut();
                for (i, &iter_count) in iterations.iter().enumerate() {
                    if i < data.len() {
                        let color_idx = iter_count.min(max_iter);
                        data[i] = COLORS[color_idx as usize % COLORS.len()];
                    }
                }
                
                let duration = start_time.elapsed();
                log::info!("GPU computation completed in {:?}", duration);
            }
            Err(e) => {
                log::error!("GPU computation failed: {}, falling back to CPU", e);
                universe.compute();
            }
        }
    }
    
    #[cfg(not(feature = "gpu"))]
    {
        log::info!("Using CPU computation");
        universe.compute();
    }

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
