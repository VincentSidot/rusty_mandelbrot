# TODO List

## 1. Performance Enhancements

### 1.1. Implement Proper Multi-threading
- [x] Replace current thread implementation with a thread pool
- [x] Implement work-stealing or work-sharing mechanism for better load balancing
- [x] Add progress tracking for long computations
- [x] Consider using `rayon` crate for easier parallelization

### 1.2. Optimize Memoization
- [ ] Replace HashMap with a thread-safe alternative (e.g., `dashmap`)
- [ ] Implement size-limited cache with LRU eviction policy
- [ ] Add cache statistics tracking (hit rate, etc.)
- [ ] Benchmark performance impact of memoization

### 1.3. Implement Adaptive Resolution
- [ ] Add logic to detect zoom level
- [ ] Adjust computation resolution based on zoom
- [ ] Implement progressive rendering for deep zooms

### 1.4. GPU Acceleration
- [ ] Research WebGPU integration with Rust
- [ ] Implement Mandelbrot computation on GPU
- [ ] Set up data transfer between CPU and GPU
- [ ] Benchmark performance gains

## 2. User Interface Improvements

### 2.1. Add UI Controls
- [ ] Implement UI framework (egui, iced, or similar)
- [ ] Add iteration count slider
- [ ] Add color scheme selector
- [ ] Add reset view button
- [ ] Add zoom level indicator

### 2.2. Coordinate Display
- [ ] Add coordinate display to window
- [ ] Show current mouse position in complex coordinates
- [ ] Display current viewport boundaries

### 2.3. Preset System
- [ ] Design preset data structure
- [ ] Implement save/load functionality
- [ ] Add default interesting locations
- [ ] Create UI for managing presets

### 2.4. Export Functionality
- [ ] Implement image export (PNG, JPEG)
- [ ] Add resolution scaling options
- [ ] Implement export progress indicator
- [ ] Add file dialog for save location

## 3. Mathematical Extensions

### 3.1. Julia Sets
- [ ] Implement Julia set computation
- [ ] Add parameter selection for Julia constant
- [ ] Create UI for switching between Mandelbrot and Julia
- [ ] Implement interactive Julia constant selection

### 3.2. Alternative Fractals
- [ ] Implement Burning Ship fractal
- [ ] Add Newton fractals
- [ ] Create generic fractal interface
- [ ] Add fractal type selector

### 3.3. Parameterized Mandelbrot
- [ ] Generalize exponent parameter
- [ ] Add UI control for exponent
- [ ] Implement other mathematical generalizations

### 3.4. 3D Visualization
- [ ] Research 3D Mandelbrot variants (Mandelbulb)
- [ ] Implement 3D rendering engine
- [ ] Add 3D navigation controls
- [ ] Optimize for performance

## 4. Code Quality Improvements

### 4.1. Dependency Updates
- [ ] Update `pixels` to latest version
- [ ] Update `winit` to latest version
- [ ] Update `env_logger` to latest version
- [ ] Remove unused `rand` dependency
- [ ] Test compatibility after updates

### 4.2. Configuration System
- [ ] Define configuration structure
- [ ] Add file-based configuration (TOML/YAML)
- [ ] Add command-line argument parsing
- [ ] Implement configuration validation

### 4.3. Error Handling
- [ ] Replace panics with proper error handling
- [ ] Add custom error types
- [ ] Implement error propagation
- [ ] Add user-friendly error messages

### 4.4. Documentation
- [ ] Add module-level documentation
- [ ] Add function-level documentation
- [ ] Create user guide
- [ ] Add examples for usage

### 4.5. Testing
- [ ] Add unit tests for complex number operations
- [ ] Add tests for Mandelbrot computations
- [ ] Add rendering tests
- [ ] Set up continuous integration

## 5. Advanced Features

### 5.1. Animation System
- [ ] Design animation data structure
- [ ] Implement keyframe system
- [ ] Add animation playback controls
- [ ] Create UI for animation creation

### 5.2. Multi-touch Support
- [ ] Research multi-touch APIs
- [ ] Implement pinch-to-zoom gesture
- [ ] Add two-finger pan gesture
- [ ] Test on touch-enabled devices

### 5.3. Network Rendering
- [ ] Design distributed computation protocol
- [ ] Implement client-server architecture
- [ ] Add work distribution logic
- [ ] Handle network failures gracefully

### 5.4. Scripting Interface
- [ ] Choose scripting language (Lua, Rhai, etc.)
- [ ] Implement scripting engine
- [ ] Add API for custom coloring algorithms
- [ ] Create documentation for scripting

## 6. Immediate Tasks

### 6.1. Refactor Current Code
- [ ] Fix the `mandelbrot` function that uses `z.pow(42)` (likely a bug)
- [ ] Clean up commented-out code in `mandelbrot.rs`
- [ ] Fix the loop condition in `mandelbrot` function (should use `z.norm()` not `z.re + z.im`)
- [ ] Review and fix any other mathematical inconsistencies

### 6.2. Improve Build Process
- [ ] Add build profiles (debug, release)
- [ ] Optimize compilation flags
- [ ] Add build scripts for different platforms
- [ ] Document build process

### 6.3. Add Logging
- [ ] Add more detailed logging throughout the application
- [ ] Implement different log levels for different use cases
- [ ] Add performance timing logs
- [ ] Implement log file output option