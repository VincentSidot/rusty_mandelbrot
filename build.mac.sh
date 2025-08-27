#!/bin/bash

# Build script for Mandelbrot project (macOS)

set -e  # Exit on any error

# Default values
PROFILE="debug"
TARGET=""

# Function to display usage
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Build the Mandelbrot project"
    echo ""
    echo "Options:"
    echo "  -h, --help     Display this help message"
    echo "  -p, --profile  Build profile (debug|release) [default: debug]"
    echo "  -t, --target   Build for specific target (e.g., x86_64-apple-darwin)"
    echo ""
    echo "Examples:"
    echo "  $0                   # Build with debug profile"
    echo "  $0 -p release        # Build with release profile"
    echo "  $0 --profile release # Build with release profile"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        -p|--profile)
            PROFILE="$2"
            shift 2
            ;;
        -t|--target)
            TARGET="--target $2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Validate profile
if [[ "$PROFILE" != "debug" && "$PROFILE" != "release" ]]; then
    echo "Error: Profile must be 'debug' or 'release'"
    exit 1
fi

# Build the project
echo "Building Mandelbrot project with profile: $PROFILE"
echo "Target: ${TARGET:-native}"

if [[ "$PROFILE" == "release" ]]; then
    cargo build --release $TARGET
    echo "Build successful! Binary located at: target/release/mandelbrot"
else
    cargo build $TARGET
    echo "Build successful! Binary located at: target/debug/mandelbrot"
fi