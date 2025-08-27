@echo off
setlocal

REM Build script for Mandelbrot project (Windows)

set PROFILE=debug
set TARGET=

REM Parse command line arguments
:parse
if "%1%"=="" goto build
if "%1%"=="-h" goto help
if "%1%"=="--help" goto help
if "%1%"=="-p" goto set_profile
if "%1%"=="--profile" goto set_profile
if "%1%"=="-t" goto set_target
if "%1%"=="--target" goto set_target
shift
goto parse

:set_profile
set PROFILE=%2%
shift
shift
goto parse

:set_target
set TARGET=--target %2%
shift
shift
goto parse

:help
echo Usage: build.bat [OPTIONS]
echo Build the Mandelbrot project
echo.
echo Options:
echo   -h, --help     Display this help message
echo   -p, --profile  Build profile (debug^|release) [default: debug]
echo   -t, --target   Build for specific target (e.g., x86_64-pc-windows-msvc)
echo.
echo Examples:
echo   build.bat                   # Build with debug profile
echo   build.bat -p release        # Build with release profile
echo   build.bat --profile release # Build with release profile
exit /b 0

:build
REM Validate profile
if not "%PROFILE%"=="debug" if not "%PROFILE%"=="release" (
    echo Error: Profile must be 'debug' or 'release'
    exit /b 1
)

REM Build the project
echo Building Mandelbrot project with profile: %PROFILE%
if "%TARGET%"=="" (
    echo Target: native
) else (
    echo Target: %TARGET%
)

if "%PROFILE%"=="release" (
    cargo build --release %TARGET%
    echo Build successful! Binary located at: target\release\mandelbrot.exe
) else (
    cargo build %TARGET%
    echo Build successful! Binary located at: target\debug\mandelbrot.exe
)