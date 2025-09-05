@echo off
REM Comprehensive test runner script for cargo-optimize (Windows)

setlocal enabledelayedexpansion

REM Default values
set CATEGORIES=all
set VERBOSE=
set FAIL_FAST=
set REPORT=text
set NO_VALIDATE=

REM Parse command line arguments
:parse_args
if "%~1"=="" goto run_tests

if "%~1"=="--categories" (
    set CATEGORIES=%~2
    shift
    shift
    goto parse_args
)

if "%~1"=="--verbose" (
    set VERBOSE=--verbose
    shift
    goto parse_args
)

if "%~1"=="-v" (
    set VERBOSE=--verbose
    shift
    goto parse_args
)

if "%~1"=="--fail-fast" (
    set FAIL_FAST=--fail-fast
    shift
    goto parse_args
)

if "%~1"=="--report" (
    set REPORT=%~2
    shift
    shift
    goto parse_args
)

if "%~1"=="--no-validate" (
    set NO_VALIDATE=--no-validate
    shift
    goto parse_args
)

if "%~1"=="--quick" (
    set CATEGORIES=unit
    set FAIL_FAST=--fail-fast
    echo Running quick tests (unit tests only)
    shift
    goto parse_args
)

if "%~1"=="--ci" (
    set FAIL_FAST=--fail-fast
    set REPORT=junit
    set VERBOSE=--verbose
    echo Running in CI mode
    shift
    goto parse_args
)

if "%~1"=="--help" goto show_help
if "%~1"=="-h" goto show_help

echo Unknown option: %~1
echo Use --help for usage information
exit /b 1

:show_help
echo cargo-optimize comprehensive test runner
echo.
echo Usage: %~nx0 [OPTIONS]
echo.
echo Options:
echo   --categories ^<LIST^>    Comma-separated list of test categories
echo   --verbose, -v          Enable verbose output
echo   --fail-fast            Stop on first test failure
echo   --report ^<FORMAT^>      Report format (text^|json^|junit^|html)
echo   --no-validate          Skip environment validation
echo   --quick                Run quick tests only (unit tests)
echo   --ci                   Run in CI mode
echo   --help, -h             Show this help message
echo.
echo Categories:
echo   unit          Unit tests
echo   integration   Integration tests
echo   property      Property-based tests
echo   fuzz          Fuzz tests
echo   performance   Performance benchmarks
echo   golden        Golden master tests
echo   stress        Stress tests
echo   boundary      Boundary value tests
echo   regression    Regression tests
echo   all           All tests (default)
exit /b 0

:run_tests
REM Build the test command
set CMD=cargo test --bin test_main

if not "%CATEGORIES%"=="all" (
    set CMD=!CMD! --categories %CATEGORIES%
)

if not "%VERBOSE%"=="" (
    set CMD=!CMD! %VERBOSE%
)

if not "%FAIL_FAST%"=="" (
    set CMD=!CMD! %FAIL_FAST%
)

if not "%REPORT%"=="text" (
    set CMD=!CMD! --report %REPORT%
)

if not "%NO_VALIDATE%"=="" (
    set CMD=!CMD! %NO_VALIDATE%
)

REM Print what we're going to run
echo Running comprehensive tests...
echo Command: !CMD!
echo.

REM Run the tests
!CMD!

REM Check the exit code
if %ERRORLEVEL% EQU 0 (
    echo.
    echo All tests passed!
    exit /b 0
) else (
    echo.
    echo Some tests failed
    exit /b 1
)
