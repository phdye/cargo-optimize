@echo off
REM cargo-optimize setup script for Windows
REM This script helps set up cargo-optimize for a Rust project

setlocal enabledelayedexpansion

REM Check if cargo is installed
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Cargo is not installed. Please install Rust from https://rustup.rs/
    exit /b 1
)

REM Check if cargo-optimize is installed
where cargo-optimize >nul 2>nul
if %errorlevel% neq 0 (
    echo [INFO] cargo-optimize is not installed. Installing...
    cargo install cargo-optimize
    if %errorlevel% neq 0 (
        echo [ERROR] Failed to install cargo-optimize
        exit /b 1
    )
)

REM Parse arguments
set PROJECT_DIR=%~1
if "%PROJECT_DIR%"=="" set PROJECT_DIR=.

set OPTIMIZATION_LEVEL=%~2
if "%OPTIMIZATION_LEVEL%"=="" set OPTIMIZATION_LEVEL=balanced

REM Change to project directory
cd /d "%PROJECT_DIR%"
if %errorlevel% neq 0 (
    echo [ERROR] Failed to change to project directory: %PROJECT_DIR%
    exit /b 1
)

REM Check if this is a Rust project
if not exist "Cargo.toml" (
    echo [ERROR] No Cargo.toml found in %PROJECT_DIR%. Is this a Rust project?
    exit /b 1
)

echo [INFO] Setting up cargo-optimize for project in %PROJECT_DIR%

REM Run analysis first
echo [INFO] Analyzing project structure...
cargo optimize analyze --detailed
if %errorlevel% neq 0 (
    echo [WARN] Analysis failed, continuing anyway
)

REM Apply optimizations
echo [INFO] Applying optimizations (level: %OPTIMIZATION_LEVEL%)...
cargo optimize -O %OPTIMIZATION_LEVEL%
if %errorlevel% neq 0 (
    echo [ERROR] Failed to apply optimizations
    exit /b 1
)

REM Install recommended tools if they're missing
where sccache >nul 2>nul
if %errorlevel% neq 0 (
    echo [INFO] Installing sccache for build caching...
    cargo install sccache
    if %errorlevel% neq 0 (
        echo [WARN] Failed to install sccache
    )
)

REM Check for lld on Windows
where lld >nul 2>nul
if %errorlevel% neq 0 (
    echo [WARN] lld linker not found. For better performance, install lld:
    echo   Using Scoop: scoop install llvm
    echo   Or download from: https://releases.llvm.org/
)

REM Run a test build to verify everything works
echo [INFO] Running test build to verify optimizations...
cargo build
if %errorlevel% neq 0 (
    echo [ERROR] Test build failed. Run 'cargo optimize reset' to revert changes.
    exit /b 1
)

REM Show stats
echo [INFO] Showing cache statistics...
cargo optimize stats

REM Success!
echo.
echo [SUCCESS] cargo-optimize successfully configured!
echo.
echo Your builds should now be significantly faster.
echo.
echo Useful commands:
echo   cargo optimize benchmark  - Measure improvement
echo   cargo optimize stats      - Show cache statistics
echo   cargo optimize reset      - Revert all changes
echo.
echo Happy coding!

endlocal
