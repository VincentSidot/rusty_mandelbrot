use pixels::{Error, Pixels, SurfaceTexture};

use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::mandelbrot::MandelbrotUniverse;

pub fn render(mut universe: MandelbrotUniverse, width: u32, height: u32) -> Result<(), Error> {
    log::info!("Starting render loop with window size {}x{}", width, height);
    let start_time = std::time::Instant::now();
    
    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(width as f64, height as f64);
        WindowBuilder::new()
            .with_title("Mandelbrot")
            .with_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(width, height, surface_texture)?
    };
    
    let init_duration = start_time.elapsed();
    log::info!("Render initialization completed in {:?}", init_duration);

    let mut last_mouse_pos = (0, 0);

    let mut is_left_mouse_button_pressed = false;

    event_loop.run(move |event, _, control_flow| {
        // Handle events
        match event {
            Event::RedrawRequested(_) => {
                universe.render(pixels.frame_mut());
                match pixels.render() {
                    Ok(_) => {}
                    Err(err) => {
                        log::error!("pixels.render: {}", err);
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                }
            }
            Event::MainEventsCleared => {
                // Remove last printed line
                // print!("\x1b[1A\x1b[2K");
                // println!("FPS: {}", 1.0 / elapsed.as_secs_f64());

                window.request_redraw();
            }
            Event::WindowEvent {
                window_id: _,
                event,
            } => match event {
                WindowEvent::Resized(new_size) => {
                    log::info!("Window resized to {}x{}", new_size.width, new_size.height);
                    let start_time = std::time::Instant::now();
                    
                    match pixels.resize_buffer(new_size.width, new_size.height) {
                        Ok(_) => {
                            match pixels.resize_surface(new_size.width, new_size.height) {
                                Ok(_) => {
                                    universe.resize(new_size.width, new_size.height);
                                    let duration = start_time.elapsed();
                                    log::info!("Resize operation completed in {:?}", duration);
                                }
                                Err(err) => {
                                    log::error!("pixels.resize_surface: {}", err);
                                    *control_flow = ControlFlow::Exit;
                                    return;
                                }
                            };
                        }
                        Err(err) => {
                            log::error!("pixels.resize_buffer: {}", err);
                            *control_flow = ControlFlow::Exit;
                            return;
                        }
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let (x, y) = (position.x, position.y);
                    log::trace!("Cursor moved to ({}, {})", x, y);

                    if is_left_mouse_button_pressed {
                        let (dx, dy) = (x - last_mouse_pos.0 as f64, y - last_mouse_pos.1 as f64);
                        log::debug!("Dragging with delta ({}, {})", dx, dy);
                        universe.translate(-dx, -dy);
                        window.request_redraw();
                    }

                    last_mouse_pos = (x as _, y as _);
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    match button {
                        winit::event::MouseButton::Left => {
                            let pressed = state == winit::event::ElementState::Pressed;
                            log::debug!("Left mouse button {}", if pressed { "pressed" } else { "released" });
                            is_left_mouse_button_pressed = pressed;
                        }
                        winit::event::MouseButton::Right => {
                            log::debug!("Right mouse button {:?}", state);
                            // Right mouse button handling could be added here if needed
                        }
                        _ => {
                            log::debug!("Other mouse button {:?} {:?}", button, state);
                        }
                    }
                },
                WindowEvent::MouseWheel { delta, .. } => {
                    let zoom = match delta {
                        winit::event::MouseScrollDelta::LineDelta(x, y) => (x, y),
                        winit::event::MouseScrollDelta::PixelDelta(pos) => (pos.x as _, pos.y as _),
                    }
                    .1 > 0.0;
                    
                    log::debug!("Mouse wheel event: zoom={}", zoom);

                    if zoom {
                        log::debug!("Zooming in at position ({}, {})", last_mouse_pos.0, last_mouse_pos.1);
                        universe.zoom(0.5, last_mouse_pos.0, last_mouse_pos.1);
                    } else {
                        log::debug!("Zooming out at position ({}, {})", last_mouse_pos.0, last_mouse_pos.1);
                        universe.zoom(1.5, last_mouse_pos.0, last_mouse_pos.1);
                    }
                    window.request_redraw();
                }
                WindowEvent::KeyboardInput {
                    device_id: _,
                    input,
                    is_synthetic: _,
                } => match input.virtual_keycode {
                    Some(VirtualKeyCode::Escape) => {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                    _ => {}
                },
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                _ => {}
            },
            _ => {}
        }
    });
}
