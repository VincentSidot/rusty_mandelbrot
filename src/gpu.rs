//! GPU-accelerated Mandelbrot computation using wgpu

use std::borrow::Cow;
use wgpu::{Adapter, Device, Instance, Queue};
use wgpu::{BufferDescriptor, BufferUsages, CommandEncoderDescriptor, ComputePassDescriptor};
use wgpu::{ComputePipeline, BindGroupLayout};
use wgpu::{util::DeviceExt, PowerPreference};

use crate::mandelbrot::ViewPort;

/// GPU context for Mandelbrot computation
pub struct GpuContext {
    _instance: Instance,
    _adapter: Adapter,
    device: Device,
    queue: Queue,
    compute_pipeline: ComputePipeline,
    bind_group_layout: BindGroupLayout,
}

/// Uniforms for the Mandelbrot compute shader
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    width: u32,
    height: u32,
    max_iter: u32,
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
    _padding: [u32; 1], // Padding to make the struct size a multiple of 16 bytes
}

impl GpuContext {
    /// Create a new GPU context
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create instance
        let instance = Instance::new(wgpu::InstanceDescriptor::default());
        
        // Get adapter
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            }
        ).await.ok_or("No suitable GPU adapter found")?;
        
        // Get device and queue
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ).await?;
        
        // Load shader
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Mandelbrot Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("mandelbrot.wgsl"))),
        });
        
        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Mandelbrot Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Mandelbrot Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Create compute pipeline
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Mandelbrot Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "main",
        });
        
        Ok(Self {
            _instance: instance,
            _adapter: adapter,
            device,
            queue,
            compute_pipeline,
            bind_group_layout,
        })
    }
    
    /// Compute Mandelbrot set using GPU
    pub async fn compute_mandelbrot(
        &self,
        width: u32,
        height: u32,
        max_iter: u32,
        viewport: ViewPort,
    ) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
        // Create uniforms
        let uniforms = Uniforms {
            width,
            height,
            max_iter,
            x_min: viewport.x_min as f32,
            x_max: viewport.x_max as f32,
            y_min: viewport.y_min as f32,
            y_max: viewport.y_max as f32,
            _padding: [0; 1],
        };
        
        // Create uniform buffer
        let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        
        // Create output buffer
        let output_size = (width * height * std::mem::size_of::<u32>() as u32) as wgpu::BufferAddress;
        let output_buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some("Output Buffer"),
            size: output_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        // Create staging buffer for reading results
        let staging_buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some("Staging Buffer"),
            size: output_size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Mandelbrot Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &output_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });
        
        // Create command encoder
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Mandelbrot Command Encoder"),
        });
        
        // Create compute pass
        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Mandelbrot Compute Pass"),
            });
            
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            
            // Dispatch compute work
            let work_groups_x = (width + 15) / 16;  // Round up to nearest multiple of 16
            let work_groups_y = (height + 15) / 16; // Round up to nearest multiple of 16
            compute_pass.dispatch_workgroups(work_groups_x, work_groups_y, 1);
        }
        
        // Copy output buffer to staging buffer
        encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, output_size);
        
        // Submit commands
        self.queue.submit(Some(encoder.finish()));
        
        // Read results
        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = futures::channel::oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        
        // Poll until mapping is complete
        self.device.poll(wgpu::Maintain::Wait);
        let result = receiver.await?;
        result?;
        
        // Extract data
        let data = buffer_slice.get_mapped_range();
        let result: Vec<u32> = bytemuck::cast_slice(&data).to_vec();
        drop(data); // Release mapping
        staging_buffer.unmap();
        
        Ok(result)
    }
}

impl ViewPort {
    /// Compute Mandelbrot set using GPU acceleration
    pub async fn compute_mandelbrot_gpu(
        width: u32,
        height: u32,
        max_iter: u32,
        viewport: ViewPort,
    ) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
        let gpu_context = GpuContext::new().await?;
        gpu_context.compute_mandelbrot(width, height, max_iter, viewport).await
    }
}