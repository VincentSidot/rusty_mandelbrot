# rusty_mandelbrot

- [rusty\_mandelbrot](#rusty_mandelbrot)
  - [Description](#description)
  - [Project Structure](#project-structure)
    - [Files](#files)
  - [Dependencies](#dependencies)
  - [Building and Running](#building-and-running)
  - [Mandelbrot Implementations](#mandelbrot-implementations)
    - [Basic Mandelbrot](#basic-mandelbrot)
    - [Optimized Mandelbrot](#optimized-mandelbrot)
    - [Cosine Mandelbrot](#cosine-mandelbrot)
  - [License](#license)


## Description

`rusty_mandelbrot` is a Rust project that generates Mandelbrot fractals. This project includes various implementations of the Mandelbrot set calculation,
including optimizations and different mathematical approaches.

## Project Structure

### Files

- **src/complex.rs**: Contains the implementation of complex number operations.
- **src/logger.rs**: Handles logging functionality.
- **src/main.rs**: The main entry point of the application.
- **src/mandelbrot.rs**: Contains different implementations of the Mandelbrot set calculation.
- **src/render.rs**: Handles rendering of the Mandelbrot set.

## Dependencies

The project uses the following dependencies:

- `env_logger = "0.10"`
- `log = "0.4.22"`
- `pixels = "0.13.0"`
- `rand = "0.8.5"`
- `winit = "0.28"`

Some are out of date, I need to try updating and check for compatibility.

## Building and Running

### Using Cargo Directly

To build the project in debug mode, use the following command:

```sh
cargo build
```

To build the project in release mode (optimized), use the following command:

```sh
cargo build --release
```

To run the project in debug mode, use the following command:

```sh
cargo run
```

To run the project in release mode, use the following command:

```sh
cargo run --release
```

### Using the Build Scripts

The project includes platform-specific build scripts that simplify building with different profiles:

#### Linux/macOS

```sh
# Build with debug profile (default)
./build.sh

# Build with release profile
./build.sh -p release

# Display help
./build.sh -h
```

#### Windows

```cmd
# Build with debug profile (default)
build.bat

# Build with release profile
build.bat -p release

# Display help
build.bat -h
```

#### macOS (Alternative)

```sh
# Build with debug profile (default)
./build.mac.sh

# Build with release profile
./build.mac.sh -p release

# Display help
./build.mac.sh -h
```

### Build Profiles

The project defines two build profiles:

1. **Debug Profile**: Default settings for development with debug symbols and no optimizations
2. **Release Profile**: Optimized settings for production with maximum optimizations and stripped symbols

### Cross-Platform Building

To build for different target platforms, use the `--target` option with the build scripts:

```sh
# Build for Windows on Linux/macOS
./build.sh --target x86_64-pc-windows-gnu

# Build for macOS on Linux
./build.sh --target x86_64-apple-darwin

# Build for Linux on macOS
./build.mac.sh --target x86_64-unknown-linux-gnu
```

Note: Cross-compilation requires installing the appropriate target toolchains. For example:
```sh
rustup target add x86_64-pc-windows-gnu
```

## Mandelbrot Implementations

### Basic Mandelbrot
The basic Mandelbrot set calculation is implemented in the mandelbrot function in src/mandelbrot.rs.

### Optimized Mandelbrot
The optimized Mandelbrot set calculation is implemented in the mandelbrot_fast function in src/mandelbrot.rs. This includes optimizations like the center and cardioid checks.

### Cosine Mandelbrot
The cosine Mandelbrot set calculation is implemented in the mandelbrot_cos function in src/mandelbrot.rs.

## License
This project is licensed under the MIT License.

