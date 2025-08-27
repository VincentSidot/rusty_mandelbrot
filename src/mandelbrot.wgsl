// mandelbrot.wgsl
// Compute shader for Mandelbrot set computation

// Uniform buffer containing Mandelbrot parameters
struct Uniforms {
    width: u32,
    height: u32,
    max_iter: u32,
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
};

// Storage buffer for the output (iteration counts)
struct Output {
    data: array<u32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read_write> output: Output;

// Convert pixel coordinates to complex number
fn pixel_to_complex(x: u32, y: u32) -> vec2<f32> {
    let fx = f32(x) / f32(uniforms.width);
    let fy = f32(y) / f32(uniforms.height);
    
    let re = uniforms.x_min + fx * (uniforms.x_max - uniforms.x_min);
    let im = uniforms.y_min + fy * (uniforms.y_max - uniforms.y_min);
    
    return vec2<f32>(re, im);
}

// Compute Mandelbrot iteration count for a complex number
fn mandelbrot(c: vec2<f32>) -> u32 {
    var z = vec2<f32>(0.0, 0.0);
    var n: u32 = 0u;
    
    loop {
        // Check if we've exceeded max iterations or escaped
        if (n >= uniforms.max_iter) {
            break;
        }
        
        // Check if we've escaped (|z| > 2)
        if (z.x * z.x + z.y * z.y > 4.0) {
            break;
        }
        
        // z = z^2 + c
        let temp = z.x * z.x - z.y * z.y + c.x;
        z.y = 2.0 * z.x * z.y + c.y;
        z.x = temp;
        
        n = n + 1u;
    }
    
    return n;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    // Check bounds
    if (x >= uniforms.width || y >= uniforms.height) {
        return;
    }
    
    // Convert pixel to complex coordinates
    let c = pixel_to_complex(x, y);
    
    // Compute Mandelbrot iteration count
    let iterations = mandelbrot(c);
    
    // Store result
    let index = y * uniforms.width + x;
    output.data[index] = iterations;
}