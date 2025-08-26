use std::sync::atomic::{AtomicU32, Ordering};

use crate::complex::Complex;
use rayon::prelude::*;

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

    // Mandelbrot universe
    view: ViewPort,
    max_iter: u32,
    gradient_table: Vec<PixelColor>,

    // Mandelbrot function
    apply: fn(Complex<f64>, u32) -> u32,

    // Mandelbrot data
    data: Vec<PixelColor>,
}

#[derive(Debug, Clone, Copy)]
pub struct ViewPort {
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
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
        // Compute gradient table

        let gradient_table = PixelColor::compute_gradient_table(max_iter, colors);

        Self {
            width,
            height,

            gradient_table,
            apply: function,

            view: ViewPort::default(),
            max_iter,

            data: vec![PixelColor::BLACK; (width * height) as usize],
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.data = vec![PixelColor::BLACK; (width * height) as usize];
    }

    fn idx_to_complex(&self, x: u32, y: u32) -> Complex<f64> {
        self.view.idx_to_complex(x, y, self.width, self.height)
    }

    

    pub fn zoom(&mut self, factor: f64, center_x: u32, center_y: u32) {
        let center = self.idx_to_complex(center_x, center_y);
        self.view.zoom(factor, center.re, center.im);
        self.compute();
    }

    pub fn translate(&mut self, dx: f64, dy: f64) {
        let dx = dx * (self.view.x_max - self.view.x_min) / self.width as f64;
        let dy = dy * (self.view.y_max - self.view.y_min) / self.height as f64;
        self.view.translate(dx, dy);
        self.compute();
    }

    fn compute_multi_thread(&mut self) {
        // Create a new vector for the computed data
        let mut new_data = vec![PixelColor::BLACK; self.data.len()];
        
        // Use rayon's parallel iterator to compute all pixels in parallel
        let width = self.width;
        let height = self.height;
        let max_iter = self.max_iter;
        let gradient_table = &self.gradient_table;
        let viewport = self.view;
        let mandelbrot = self.apply;
        
        // Atomic counter for progress tracking
        let processed_pixels = AtomicU32::new(0);
        let total_pixels = self.data.len() as u32;
        
        // Parallel computation using rayon
        new_data.par_iter_mut().enumerate().for_each(|(idx, pixel)| {
            let x = idx as u32 % width;
            let y = idx as u32 / width;
            let c = viewport.idx_to_complex(x, y, width, height);
            let n = mandelbrot(c, max_iter);
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

    pub fn compute(&mut self) {
        let t1 = std::time::Instant::now();
        // Always use multi-threading with rayon, which automatically manages the thread pool
        self.compute_multi_thread();
        let t2 = std::time::Instant::now();
        println!("Compute time: {:?} with {} threads", t2 - t1, rayon::current_num_threads());
    }

    pub fn render(&self, frame: &mut [u8]) {
        debug_assert!(self.data.len() * 4 <= frame.len());
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let color = self.data[i];
            pixel.copy_from_slice(&[color.r, color.g, color.b, color.a]);
        }
    }
}
