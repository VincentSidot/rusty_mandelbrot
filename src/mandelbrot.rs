use std::sync::atomic::{AtomicU32, Ordering};

use crate::complex::Complex;
use dashmap::DashMap;
use rayon::prelude::*;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy)]
pub struct PixelColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

macro_rules! define_color {
    ($($name:ident => [$r:expr, $g:expr, $b:expr, $a:expr]);*$(;)?) => {
        $(
            define_color!(@inner $name => [$r, $g, $b, $a]);
        )*
    };
    (@inner $name:ident => [$r:expr, $g:expr, $b:expr, $a:expr]) => {
        #[allow(unused)]
        pub const $name: PixelColor = PixelColor::new($r, $g, $b, $a);
    };
}

impl PixelColor {
    const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    define_color!(
        BLACK => [0, 0, 0, 255];
        WHITE => [255, 255, 255, 255];
        RED => [255, 0, 0, 255];
        GREEN => [0, 255, 0, 255];
        BLUE => [0, 0, 255, 255];
        CYAN => [0, 255, 255, 255];
        MAGENTA => [255, 0, 255, 255];
        YELLOW => [255, 255, 0, 255];
    );

    fn gradient(n: u32, max_iter: u32, colors: &[Self]) -> Self {
        let p = n as f32 / max_iter as f32;
        let p = p.clamp(0.0, 0.9999);
        let n = colors.len() - 1;
        let idx = (p * n as f32).floor() as usize;
        let p = p * n as f32 - idx as f32;

        let c0 = colors[idx];
        let c1 = colors[idx + 1];

        let r = (c0.r as f32 + (c1.r as f32 - c0.r as f32) * p) as u8;
        let g = (c0.g as f32 + (c1.g as f32 - c0.g as f32) * p) as u8;
        let b = (c0.b as f32 + (c1.b as f32 - c0.b as f32) * p) as u8;
        let a = (c0.a as f32 + (c1.a as f32 - c0.a as f32) * p) as u8;

        Self::new(r, g, b, a)
    }

    fn compute_gradient_table(max_iter: u32, colors: &[Self]) -> Vec<Self> {
        (0..=max_iter)
            .map(|n| Self::gradient(n, max_iter, colors))
            .collect()
    }
}

pub struct MandelbrotUniverse {
    width: u32,
    height: u32,

    view: ViewPort,
    max_iter: u32,
    gradient_table: Vec<PixelColor>,

    apply: fn(Complex<f64>, u32) -> u32,

    data: Vec<PixelColor>,

    // Memoization
    memo_cache: DashMap<Complex<f64>, u32>,
    memo_history: std::sync::Mutex<VecDeque<Complex<f64>>>,
    memo_max_size: usize,
    memo_hits: AtomicU32,
    memo_misses: AtomicU32,

    // Adaptive resolution
    base_resolution: u32,
    adaptive_resolution: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ViewPort {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl ViewPort {
    pub fn zoom(&mut self, factor: f64, center_x: f64, center_y: f64) {
        let width = self.x_max - self.x_min;
        let height = self.y_max - self.y_min;

        let new_width = width / factor;
        let new_height = height / factor;

        self.x_min = center_x - new_width / 2.0;
        self.x_max = center_x + new_width / 2.0;
        self.y_min = center_y - new_height / 2.0;
        self.y_max = center_y + new_height / 2.0;
    }

    pub fn translate(&mut self, dx: f64, dy: f64) {
        self.x_min += dx;
        self.x_max += dx;
        self.y_min += dy;
        self.y_max += dy;
    }

    pub fn idx_to_complex(&self, x: u32, y: u32, width: u32, height: u32) -> Complex<f64> {
        let x = x as f64;
        let y = y as f64;

        let re = self.x_min + (self.x_max - self.x_min) * x / width as f64;
        let im = self.y_min + (self.y_max - self.y_min) * y / height as f64;

        Complex::new(re, im)
    }
}

impl std::default::Default for ViewPort {
    fn default() -> Self {
        Self {
            x_min: -2.0,
            x_max: 1.0,
            y_min: -1.5,
            y_max: 1.5,
        }
    }
}

impl MandelbrotUniverse {
    pub fn new(
        width: u32,
        height: u32,
        colors: &[PixelColor],
        max_iter: u32,
        function: fn(Complex<f64>, u32) -> u32,
    ) -> Self {
        log::debug!("Initializing MandelbrotUniverse with dimensions {}x{}", width, height);
        log::debug!("Max iterations: {}", max_iter);
        log::debug!("Color palette size: {}", colors.len());
        
        let start_time = std::time::Instant::now();
        let gradient_table = PixelColor::compute_gradient_table(max_iter, colors);
        let duration = start_time.elapsed();
        
        log::debug!("Computed gradient table with {} entries in {:?}", gradient_table.len(), duration);

        Self {
            width,
            height,

            gradient_table,
            apply: function,

            view: ViewPort::default(),
            max_iter,

            data: vec![PixelColor::BLACK; (width * height) as usize],

            // Memoization
            memo_cache: DashMap::new(),
            memo_history: std::sync::Mutex::new(VecDeque::new()),
            memo_max_size: 10000, // Default cache size
            memo_hits: AtomicU32::new(0),
            memo_misses: AtomicU32::new(0),

            // Adaptive resolution
            base_resolution: 1,
            adaptive_resolution: true,
        }
    }
    
    /// Get the viewport
    pub fn view(&self) -> ViewPort {
        self.view
    }
    
    /// Get a mutable reference to the data
    pub fn data_mut(&mut self) -> &mut [PixelColor] {
        &mut self.data
    }
    
    /// Get the maximum iterations
    pub fn max_iter(&self) -> u32 {
        self.max_iter
    }
    
    /// Get the width
    pub fn width(&self) -> u32 {
        self.width
    }
    
    /// Get the height
    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        log::debug!("Resizing universe from {}x{} to {}x{}", self.width, self.height, width, height);
        let start_time = std::time::Instant::now();
        
        self.width = width;
        self.height = height;
        self.data = vec![PixelColor::BLACK; (width * height) as usize];
        
        let duration = start_time.elapsed();
        log::debug!("Resize completed in {:?}, new data buffer size: {}", duration, self.data.len());
    }

    fn idx_to_complex(&self, x: u32, y: u32) -> Complex<f64> {
        self.view.idx_to_complex(x, y, self.width, self.height)
    }

    

    pub fn zoom(&mut self, factor: f64, center_x: u32, center_y: u32) {
        log::debug!("Zooming by factor {} at center ({}, {})", factor, center_x, center_y);
        let start_time = std::time::Instant::now();
        
        let center = self.idx_to_complex(center_x, center_y);
        self.view.zoom(factor, center.re, center.im);
        
        // For deep zooms, we might want to do progressive rendering
        let zoom_level = self.calculate_zoom_level();
        if zoom_level > 1000.0 && factor < 1.0 {
            // For very deep zooms when zooming in, we could implement progressive rendering
            // For now, we'll just compute normally but with adaptive resolution
            log::info!("Deep zoom detected ({}x), using adaptive resolution", zoom_level);
        }
        
        self.compute();
        
        let duration = start_time.elapsed();
        log::debug!("Zoom operation completed in {:?}", duration);
    }

    pub fn translate(&mut self, dx: f64, dy: f64) {
        log::debug!("Translating by ({}, {})", dx, dy);
        let start_time = std::time::Instant::now();
        
        let dx = dx * (self.view.x_max - self.view.x_min) / self.width as f64;
        let dy = dy * (self.view.y_max - self.view.y_min) / self.height as f64;
        self.view.translate(dx, dy);
        self.compute();
        
        let duration = start_time.elapsed();
        log::debug!("Translation completed in {:?}", duration);
    }

    fn compute_multi_thread(&mut self) {
        // Get the resolution for this computation
        let resolution = self.get_adaptive_resolution();
        log::info!("Computing with resolution: {}x", resolution);
        
        // If resolution > 1, we compute at lower resolution and upscale
        if resolution > 1 {
            self.compute_multi_thread_with_resolution(resolution);
        } else {
            // Standard full resolution computation
            self.compute_multi_thread_full_resolution();
        }
    }

    fn compute_multi_thread_full_resolution(&mut self) {
        // Create a new vector for the computed data
        let mut new_data = vec![PixelColor::BLACK; self.data.len()];
        
        // Use rayon's parallel iterator to compute all pixels in parallel
        let width = self.width;
        let height = self.height;
        let max_iter = self.max_iter;
        let gradient_table = &self.gradient_table;
        let viewport = self.view;
        let mandelbrot = self.apply;
        
        // Memoization
        let memo_cache = &self.memo_cache;
        let memo_history = &self.memo_history;
        let memo_max_size = self.memo_max_size;
        let memo_hits = &self.memo_hits;
        let memo_misses = &self.memo_misses;
        
        // Atomic counter for progress tracking
        let processed_pixels = AtomicU32::new(0);
        let total_pixels = self.data.len() as u32;
        
        // Parallel computation using rayon
        new_data.par_iter_mut().enumerate().for_each(|(idx, pixel)| {
            let x = idx as u32 % width;
            let y = idx as u32 / width;
            let c = viewport.idx_to_complex(x, y, width, height);
            
            // Try to get from cache first
            let n = if let Some(result) = memo_cache.get(&c) {
                memo_hits.fetch_add(1, Ordering::Relaxed);
                *result
            } else {
                // Cache miss - compute the result
                memo_misses.fetch_add(1, Ordering::Relaxed);
                let result = mandelbrot(c, max_iter);
                
                // Add to cache
                memo_cache.insert(c, result);
                
                // Track insertion order for LRU eviction
                let mut history = memo_history.lock().unwrap();
                history.push_back(c);
                
                // Evict oldest entries if cache is too large
                if memo_cache.len() > memo_max_size {
                    if let Some(oldest) = history.pop_front() {
                        memo_cache.remove(&oldest);
                    }
                }
                
                result
            };
            
            *pixel = if n == max_iter {
                PixelColor::BLACK
            } else {
                gradient_table[n as usize]
            };
            
            // Progress tracking
            let count = processed_pixels.fetch_add(1, Ordering::Relaxed);
            if count % (total_pixels / 100).max(1) == 0 {
                let progress = (count as f32 / total_pixels as f32) * 100.0;
                log::trace!("Progress: {:.1}%", progress);
            }
        });
        
        std::mem::swap(&mut self.data, &mut new_data);
    }

    fn compute_multi_thread_with_resolution(&mut self, resolution: u32) {
        // Create a lower resolution data buffer
        let low_width = (self.width + resolution - 1) / resolution;
        let low_height = (self.height + resolution - 1) / resolution;
        let low_data_size = (low_width * low_height) as usize;
        let mut low_data = vec![PixelColor::BLACK; low_data_size];
        
        // Compute at lower resolution
        let width = low_width;
        let _height = low_height;
        let max_iter = self.max_iter;
        let gradient_table = &self.gradient_table;
        let viewport = self.view;
        let mandelbrot = self.apply;
        
        // Memoization
        let memo_cache = &self.memo_cache;
        let memo_history = &self.memo_history;
        let memo_max_size = self.memo_max_size;
        let memo_hits = &self.memo_hits;
        let memo_misses = &self.memo_misses;
        
        // Atomic counter for progress tracking
        let processed_pixels = AtomicU32::new(0);
        let total_pixels = low_data_size as u32;
        
        // Parallel computation using rayon
        low_data.par_iter_mut().enumerate().for_each(|(idx, pixel)| {
            let x = idx as u32 % width;
            let y = idx as u32 / width;
            let c = viewport.idx_to_complex(x * resolution, y * resolution, self.width, self.height);
            
            // Try to get from cache first
            let n = if let Some(result) = memo_cache.get(&c) {
                memo_hits.fetch_add(1, Ordering::Relaxed);
                *result
            } else {
                // Cache miss - compute the result
                memo_misses.fetch_add(1, Ordering::Relaxed);
                let result = mandelbrot(c, max_iter);
                
                // Add to cache
                memo_cache.insert(c, result);
                
                // Track insertion order for LRU eviction
                let mut history = memo_history.lock().unwrap();
                history.push_back(c);
                
                // Evict oldest entries if cache is too large
                if memo_cache.len() > memo_max_size {
                    if let Some(oldest) = history.pop_front() {
                        memo_cache.remove(&oldest);
                    }
                }
                
                result
            };
            
            *pixel = if n == max_iter {
                PixelColor::BLACK
            } else {
                gradient_table[n as usize]
            };
            
            // Progress tracking
            let count = processed_pixels.fetch_add(1, Ordering::Relaxed);
            if count % (total_pixels / 100).max(1) == 0 {
                let progress = (count as f32 / total_pixels as f32) * 100.0;
                log::trace!("Progress: {:.1}%", progress);
            }
        });
        
        // Upscale to full resolution using nearest neighbor
        let mut new_data = vec![PixelColor::BLACK; (self.width * self.height) as usize];
        for y in 0..self.height {
            for x in 0..self.width {
                let low_x = x / resolution;
                let low_y = y / resolution;
                let low_idx = (low_y * low_width + low_x) as usize;
                let idx = (y * self.width + x) as usize;
                new_data[idx] = low_data[low_idx.min(low_data_size - 1)];
            }
        }
        
        std::mem::swap(&mut self.data, &mut new_data);
    }

    pub fn set_memo_max_size(&mut self, size: usize) {
        log::debug!("Setting memoization cache max size to {}", size);
        self.memo_max_size = size;
    }

    pub fn compute(&mut self) {
        log::info!("Starting Mandelbrot computation");
        let t1 = std::time::Instant::now();
        // Always use multi-threading with rayon, which automatically manages the thread pool
        self.compute_multi_thread();
        let t2 = std::time::Instant::now();
        log::info!("Compute time: {:?} with {} threads", t2 - t1, rayon::current_num_threads());
    }

    pub fn memo_stats(&self) -> (usize, u32, u32, f64) {
        let hits = self.memo_hits.load(Ordering::Relaxed);
        let misses = self.memo_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        (self.memo_cache.len(), hits, misses, hit_rate)
    }

    pub fn clear_memo_cache(&self) {
        log::debug!("Clearing memoization cache");
        let start_time = std::time::Instant::now();
        
        self.memo_cache.clear();
        let mut history = self.memo_history.lock().unwrap();
        history.clear();
        self.memo_hits.store(0, Ordering::Relaxed);
        self.memo_misses.store(0, Ordering::Relaxed);
        
        let duration = start_time.elapsed();
        log::debug!("Memoization cache cleared in {:?}", duration);
    }

    pub fn set_adaptive_resolution(&mut self, enabled: bool) {
        log::debug!("Setting adaptive resolution to {}", enabled);
        self.adaptive_resolution = enabled;
    }

    pub fn set_base_resolution(&mut self, resolution: u32) {
        log::debug!("Setting base resolution to {}", resolution);
        self.base_resolution = resolution.max(1);
    }

    /// Calculate the current zoom level based on viewport size
    fn calculate_zoom_level(&self) -> f64 {
        let initial_width = 3.0; // Initial viewport width (-2.0 to 1.0)
        let current_width = self.view.x_max - self.view.x_min;
        let zoom_level = initial_width / current_width;
        log::trace!("Calculated zoom level: {} (initial_width: {}, current_width: {})", zoom_level, initial_width, current_width);
        zoom_level
    }

    /// Determine the appropriate resolution based on zoom level
    fn get_adaptive_resolution(&self) -> u32 {
        if !self.adaptive_resolution {
            log::trace!("Adaptive resolution disabled, returning base resolution: {}", self.base_resolution);
            return self.base_resolution;
        }

        let zoom_level = self.calculate_zoom_level();
        log::trace!("Current zoom level: {}", zoom_level);
        
        // At higher zoom levels, we need higher resolution
        // This is a simple logarithmic scaling
        let resolution = if zoom_level < 10.0 {
            self.base_resolution
        } else if zoom_level < 100.0 {
            self.base_resolution * 2
        } else if zoom_level < 1000.0 {
            self.base_resolution * 4
        } else if zoom_level < 10000.0 {
            self.base_resolution * 8
        } else {
            self.base_resolution * 16
        };
        
        log::trace!("Calculated adaptive resolution: {}", resolution);
        resolution
    }

    pub fn render(&self, frame: &mut [u8]) {
        debug_assert!(self.data.len() * 4 <= frame.len());
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let color = self.data[i];
            pixel.copy_from_slice(&[color.r, color.g, color.b, color.a]);
        }
    }
}
