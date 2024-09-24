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

To build the project, use the following command:

```sh
cargo build
```

To run the project, use the following command:

```sh
cargo run
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

