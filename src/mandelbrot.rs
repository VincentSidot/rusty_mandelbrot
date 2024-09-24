use std::thread;

use crate::complex::Complex;

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
    threads: usize,

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
        threads: usize,
        colors: &[PixelColor],
        max_iter: u32,
        function: fn(Complex<f64>, u32) -> u32,
    ) -> Self {
        // Compute gradient table

        let gradient_table = PixelColor::compute_gradient_table(max_iter, colors);
        let threads = threads.max(1); // At least 1 thread

        Self {
            width,
            height,
            threads,

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
        self.compute();
    }

    fn rev_convert_idx(&self, idx: usize) -> (u32, u32) {
        let x = idx as u32 % self.width;
        let y = idx as u32 / self.width;

        (x, y)
    }

    fn idx_to_complex(&self, x: u32, y: u32) -> Complex<f64> {
        self.view.idx_to_complex(x, y, self.width, self.height)
    }

    fn compute_single_thread(&mut self) {
        for idx in 0..self.data.len() {
            let (x, y) = self.rev_convert_idx(idx);
            let c = self.idx_to_complex(x, y);
            let n = (self.apply)(c, self.max_iter);

            let color = if n == self.max_iter {
                PixelColor::BLACK
            } else {
                self.gradient_table[n as usize]
                // let brightness = 255 - n * 255 / self.max_iter;
                // PixelColor::new(brightness as u8, brightness as u8, brightness as u8, 255)
            };

            self.data[idx] = color;
        }
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
        let concurrent_threads = self.threads;
        let pixels_per_thread = self.data.len() / concurrent_threads;

        let mut new_data = vec![PixelColor::BLACK; self.data.len()];

        let mut pixels = {
            let mut rep = Vec::new();
            let mut pixels = new_data.as_mut_slice();
            for _ in 0..concurrent_threads {
                let (start, end) = pixels.split_at_mut(pixels_per_thread as usize);
                rep.push(start);
                pixels = end;
            }
            rep
        };

        let width = self.width;
        let height = self.height;
        let max_iter = self.max_iter;

        let gradient_table = self.gradient_table.clone();
        let viewport = self.view.clone();

        // Create a scope for the threads to run in
        thread::scope(|s| {
            for (i, cells) in pixels.iter_mut().enumerate() {
                let base_index = i * pixels_per_thread as usize;
                let gradient_table = gradient_table.clone();
                let viewport = viewport.clone();

                let mandelbrot = self.apply;

                s.spawn(move || {
                    for (i, pixel) in cells.iter_mut().enumerate() {
                        let (x, y) = (
                            (base_index + i) as u32 % width,
                            (base_index + i) as u32 / width,
                        );
                        let c = viewport.idx_to_complex(x, y, width, height);
                        let n = mandelbrot(c, max_iter);
                        *pixel = gradient_table[n as usize];
                    }
                });
            }
        });

        // let max_iter = new_data.iter().max().unwrap();
        // let gradient = PixelColor::compute_gradient_table(
        //     *max_iter,
        //     vec![PixelColor::BLUE, PixelColor::YELLOW],
        // );
        // self.data = new_data.iter().map(|&n| gradient[n as usize]).collect();
        std::mem::swap(&mut self.data, &mut new_data);
    }

    pub fn compute(&mut self) {
        let t1 = std::time::Instant::now();
        if self.threads == 1 {
            self.compute_single_thread();
        } else {
            self.compute_multi_thread();
        }
        let t2 = std::time::Instant::now();
        println!("Compute time: {:?}", t2 - t1);
    }

    pub fn render(&self, frame: &mut [u8]) {
        debug_assert!(self.data.len() * 4 <= frame.len());
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let color = self.data[i];
            pixel.copy_from_slice(&[color.r, color.g, color.b, color.a]);
        }
    }
}
