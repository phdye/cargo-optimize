//! cargo-optimize CLI tool

use cargo_optimize::{
    config::{Config, OptimizationLevel},
    utils::{print_error, print_info, print_success},
    Optimizer,
};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(
    name = "cargo-optimize",
    version,
    about = "Automatically optimize Rust build times",
    long_about = "cargo-optimize automatically detects your hardware, analyzes your project, \
                  and applies optimal build settings to dramatically reduce compilation times."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to the project (defaults to current directory)
    #[arg(short, long, value_name = "PATH")]
    path: Option<PathBuf>,

    /// Optimization level
    #[arg(short = 'O', long, value_enum, default_value = "balanced")]
    optimization: OptLevel,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Dry run (show what would be done without making changes)
    #[arg(long)]
    dry_run: bool,

    /// Number of parallel jobs
    #[arg(short, long, value_name = "N")]
    jobs: Option<usize>,

    /// Disable specific optimizations
    #[arg(long, value_name = "OPT")]
    disable: Vec<String>,

    /// Configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Apply optimizations to the project
    Optimize {
        /// Optimization level
        #[arg(short = 'O', long, value_enum, default_value = "balanced")]
        level: OptLevel,
    },

    /// Set up cargo-optimize for a Rust project (replaces setup.sh/setup.bat)
    Setup {
        /// Project directory to set up (defaults to current directory)
        #[arg(short, long, value_name = "PATH")]
        path: Option<PathBuf>,

        /// Optimization level to apply
        #[arg(short = 'O', long, value_enum, default_value = "balanced")]
        level: OptLevel,

        /// Skip installing recommended tools
        #[arg(long)]
        no_tools: bool,

        /// Skip running test build
        #[arg(long)]
        no_verify: bool,
    },

    /// Initialize a new project with cargo-optimize configuration
    Init {
        /// Project directory (defaults to current directory)
        #[arg(short, long, value_name = "PATH")]
        path: Option<PathBuf>,

        /// Create example configuration file
        #[arg(long)]
        example_config: bool,
    },

    /// Analyze project without applying optimizations
    Analyze {
        /// Show detailed analysis
        #[arg(long)]
        detailed: bool,
    },

    /// Show current configuration
    Config {
        /// Output format
        #[arg(long, value_enum, default_value = "toml")]
        format: OutputFormat,
    },

    /// Install recommended tools (sccache, mold, etc.)
    Install {
        /// Tool to install
        #[arg(value_enum)]
        tool: Option<Tool>,
    },

    /// Benchmark the project with and without optimizations
    Benchmark {
        /// Number of iterations
        #[arg(short, long, default_value = "3")]
        iterations: u32,
    },

    /// Reset all optimizations
    Reset {
        /// Also clean the target directory
        #[arg(long)]
        clean: bool,
    },

    /// Show cache statistics
    Stats,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OptLevel {
    /// Conservative optimizations (safest)
    Conservative,
    /// Balanced optimizations (recommended)
    Balanced,
    /// Aggressive optimizations (fastest)
    Aggressive,
    /// Custom settings from config file
    Custom,
}

impl From<OptLevel> for OptimizationLevel {
    fn from(level: OptLevel) -> Self {
        match level {
            OptLevel::Conservative => OptimizationLevel::Conservative,
            OptLevel::Balanced => OptimizationLevel::Balanced,
            OptLevel::Aggressive => OptimizationLevel::Aggressive,
            OptLevel::Custom => OptimizationLevel::Custom,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    /// TOML format
    Toml,
    /// JSON format
    Json,
    /// Human-readable format
    Text,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Tool {
    /// sccache for build caching
    Sccache,
    /// mold linker (Linux only)
    Mold,
    /// lld linker
    Lld,
    /// All recommended tools
    All,
}

fn main() {
    // Parse arguments
    let cli = Cli::parse();

    // Set up environment
    if cli.verbose {
        std::env::set_var("CARGO_OPTIMIZE_VERBOSE", "1");
        std::env::set_var("RUST_LOG", "cargo_optimize=debug");
    }

    if cli.dry_run {
        std::env::set_var("CARGO_OPTIMIZE_DRY_RUN", "1");
    }

    // Handle cargo subcommand invocation
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "optimize" {
        // Called as 'cargo optimize'
        let cli = Cli::parse_from(args.iter().skip(1));
        run(cli);
    } else {
        // Called directly as 'cargo-optimize'
        run(cli);
    }
}

fn run(cli: Cli) {
    let result = match cli.command {
        Some(Commands::Optimize { level }) => optimize(cli.path.clone(), level, &cli),
        Some(Commands::Setup {
            path,
            level,
            no_tools,
            no_verify,
        }) => setup_project(path, level, no_tools, no_verify),
        Some(Commands::Init {
            path,
            example_config,
        }) => init_project(path, example_config),
        Some(Commands::Analyze { detailed }) => analyze(cli.path.clone(), detailed),
        Some(Commands::Config { format }) => show_config(format),
        Some(Commands::Install { tool }) => install_tools(tool),
        Some(Commands::Benchmark { iterations }) => benchmark(cli.path.clone(), iterations),
        Some(Commands::Reset { clean }) => reset(cli.path.clone(), clean),
        Some(Commands::Stats) => show_stats(),
        None => {
            // Default action: optimize with provided settings
            optimize(cli.path.clone(), cli.optimization, &cli)
        }
    };

    if let Err(e) = result {
        print_error(&format!("Error: {}", e));
        process::exit(1);
    }
}

fn setup_project(
    path: Option<PathBuf>,
    level: OptLevel,
    no_tools: bool,
    no_verify: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    let project_root = path.unwrap_or_else(|| PathBuf::from("."));

    print_info("🚀 Setting up cargo-optimize for your Rust project...");

    // Check if this is a Rust project
    let cargo_toml = project_root.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(format!(
            "No Cargo.toml found in {}. Is this a Rust project?",
            project_root.display()
        )
        .into());
    }

    // Change to project directory
    std::env::set_current_dir(&project_root)?;

    // Run analysis first
    print_info("📊 Analyzing project structure...");
    if let Err(e) = analyze(Some(project_root.clone()), true) {
        print_info(&format!("⚠️  Analysis failed: {}, continuing anyway", e));
    }

    // Apply optimizations
    print_info(&format!(
        "⚡ Applying optimizations (level: {:?})...",
        level
    ));
    let mut config = Config::new();
    config.set_optimization_level(level.into());

    let mut optimizer = Optimizer::with_config(&project_root, config)?;
    optimizer.optimize()?;

    // Install recommended tools
    if !no_tools {
        print_info("🛠️  Installing recommended tools...");
        if let Err(e) = install_tools(Some(Tool::All)) {
            print_info(&format!("⚠️  Some tools failed to install: {}", e));
        }
    }

    // Run test build to verify everything works
    if !no_verify {
        print_info("🔨 Running test build to verify optimizations...");
        let status = Command::new("cargo").arg("build").status()?;

        if !status.success() {
            return Err("Test build failed. Run 'cargo optimize reset' to revert changes.".into());
        }
    }

    // Show cache statistics
    print_info("📈 Cache statistics:");
    if let Err(e) = show_stats() {
        print_info(&format!("⚠️  Could not show stats: {}", e));
    }

    // Success message
    print_success("✅ cargo-optimize successfully configured!");
    println!();
    println!("🚀 Your builds should now be significantly faster!");
    println!();
    println!("📚 Useful commands:");
    println!("  cargo optimize benchmark  # Measure improvement");
    println!("  cargo optimize stats      # Show cache statistics");
    println!("  cargo optimize reset      # Revert all changes");
    println!("  cargo optimize analyze    # Re-analyze project");
    println!();
    println!("Happy coding! 🎉");

    Ok(())
}

fn init_project(
    path: Option<PathBuf>,
    example_config: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let project_root = path.unwrap_or_else(|| PathBuf::from("."));

    print_info("🏗️  Initializing cargo-optimize configuration...");

    // Check if this is a Rust project
    let cargo_toml = project_root.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(format!(
            "No Cargo.toml found in {}. Is this a Rust project?",
            project_root.display()
        )
        .into());
    }

    // Create .cargo-optimize.toml if it doesn't exist
    let config_file = project_root.join(".cargo-optimize.toml");
    if !config_file.exists() {
        let default_config = if example_config {
            include_str!("../../.cargo-optimize.toml")
        } else {
            r#"# cargo-optimize configuration
# This file was generated by 'cargo optimize init'

optimization_level = "balanced"
auto_detect_hardware = true
analyze_project = true
optimize_linker = true
enable_cache = true

# Uncomment to set specific values:
# parallel_jobs = 8
# incremental = true
# split_debuginfo = true

# Profile overrides (uncomment to customize):
# [profile_overrides.dev]
# opt_level = 0
# incremental = true
# codegen_units = 256

# [profile_overrides.release] 
# opt_level = 3
# lto = "thin"
# codegen_units = 1
"#
        };

        std::fs::write(&config_file, default_config)?;
        print_success(&format!("Created {}", config_file.display()));
    } else {
        print_info(&format!("{} already exists", config_file.display()));
    }

    // Create .gitignore entries if needed
    let gitignore = project_root.join(".gitignore");
    if gitignore.exists() {
        let contents = std::fs::read_to_string(&gitignore)?;
        if !contents.contains("# cargo-optimize cache") {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&gitignore)?;

            use std::io::Write;
            writeln!(file, "")?;
            writeln!(file, "# cargo-optimize cache")?;
            writeln!(file, ".cargo-optimize-cache/")?;

            print_success("Added cache entries to .gitignore");
        }
    }

    print_success("✅ Project initialized with cargo-optimize!");
    println!();
    println!("🎯 Next steps:");
    println!("  cargo optimize setup      # Apply optimizations");
    println!("  cargo optimize analyze    # Analyze your project");
    println!("  cargo optimize benchmark  # Measure improvements");

    Ok(())
}

fn optimize(
    path: Option<PathBuf>,
    level: OptLevel,
    cli: &Cli,
) -> Result<(), Box<dyn std::error::Error>> {
    let project_root = path.unwrap_or_else(|| PathBuf::from("."));

    // Load or create configuration
    let mut config = if let Some(config_path) = &cli.config {
        Config::from_file(config_path)?
    } else {
        Config::new()
    };

    // Apply CLI overrides
    config.set_optimization_level(level.into());
    config.verbose = cli.verbose;
    config.dry_run = cli.dry_run;

    if let Some(jobs) = cli.jobs {
        config.set_parallel_jobs(jobs);
    }

    // Handle disabled optimizations
    for opt in &cli.disable {
        match opt.as_str() {
            "linker" => config.optimize_linker = false,
            "cache" => config.enable_cache = false,
            "incremental" => config.incremental = false,
            "split-debuginfo" => config.split_debuginfo = false,
            _ => print_info(&format!("Unknown optimization: {}", opt)),
        }
    }

    // Run optimizer
    let mut optimizer = Optimizer::with_config(project_root, config)?;
    optimizer.optimize()?;

    Ok(())
}

fn analyze(path: Option<PathBuf>, detailed: bool) -> Result<(), Box<dyn std::error::Error>> {
    use cargo_optimize::analyzer::ProjectAnalysis;

    let project_root = path.unwrap_or_else(|| PathBuf::from("."));

    print_info("Analyzing project...");
    let analysis = ProjectAnalysis::analyze(project_root)?;

    println!("\n{}", "=".repeat(60));
    println!("PROJECT ANALYSIS");
    println!("{}", "=".repeat(60));

    // Basic info
    println!("\nProject: {}", analysis.metadata.name);
    println!("Version: {}", analysis.metadata.version);
    println!(
        "Type: {}",
        if analysis.is_workspace() {
            "Workspace"
        } else {
            "Single crate"
        }
    );

    if analysis.is_workspace() {
        println!("Workspace members: {}", analysis.crate_count());
    }

    // Code statistics
    println!("\nCode Statistics:");
    println!("  Total lines: {}", analysis.code_stats.total_lines);
    println!("  Rust code: {} lines", analysis.code_stats.rust_lines);
    println!("  Test code: {} lines", analysis.code_stats.test_lines);
    println!(
        "  Files: {} total, {} tests",
        analysis.code_stats.rust_files, analysis.code_stats.test_files
    );

    // Dependencies
    println!("\nDependencies:");
    println!("  Total: {}", analysis.dependencies.total_dependencies);
    println!("  Direct: {}", analysis.dependencies.direct_dependencies);
    println!(
        "  Transitive: {}",
        analysis.dependencies.transitive_dependencies
    );
    println!("  Proc macros: {}", analysis.dependencies.proc_macro_count);

    if !analysis.dependencies.heavy_dependencies.is_empty() {
        println!("  Heavy dependencies detected:");
        for dep in &analysis.dependencies.heavy_dependencies {
            println!("    - {}", dep);
        }
    }

    if !analysis.dependencies.duplicates.is_empty() {
        println!("  Duplicate dependencies:");
        for dup in &analysis.dependencies.duplicates {
            println!("    - {}: {:?}", dup.name, dup.versions);
        }
    }

    // Build complexity
    println!("\nBuild Complexity:");
    println!("  Score: {}/100", analysis.complexity.score);
    println!(
        "  Estimated build time: {}s",
        analysis.complexity.estimated_build_time
    );

    if detailed {
        println!("  Factors:");
        for factor in &analysis.complexity.factors {
            println!("    - {:?}", factor);
        }
    }

    // Recommendations
    if !analysis.recommendations.is_empty() {
        println!("\nRecommendations:");
        for rec in &analysis.recommendations {
            println!("  • {}", rec.description());
        }
    }

    println!("\n{}", "=".repeat(60));

    Ok(())
}

fn show_config(format: OutputFormat) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::default();

    match format {
        OutputFormat::Toml => {
            let toml = toml::to_string_pretty(&config)?;
            println!("{}", toml);
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&config)?;
            println!("{}", json);
        }
        OutputFormat::Text => {
            println!("Current Configuration:");
            println!("  Optimization level: {:?}", config.optimization_level);
            println!("  Auto-detect hardware: {}", config.auto_detect_hardware);
            println!("  Analyze project: {}", config.analyze_project);
            println!("  Optimize linker: {}", config.optimize_linker);
            println!("  Enable cache: {}", config.enable_cache);
            println!("  Incremental compilation: {}", config.incremental);
            println!("  Split debuginfo: {}", config.split_debuginfo);

            if let Some(jobs) = config.parallel_jobs {
                println!("  Parallel jobs: {}", jobs);
            } else {
                println!("  Parallel jobs: auto");
            }
        }
    }

    Ok(())
}

fn install_tools(tool: Option<Tool>) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    let tools = match tool {
        Some(Tool::Sccache) => vec![Tool::Sccache],
        Some(Tool::Mold) => vec![Tool::Mold],
        Some(Tool::Lld) => vec![Tool::Lld],
        Some(Tool::All) | None => vec![Tool::Sccache, Tool::Lld, Tool::Mold],
    };

    for tool in tools {
        match tool {
            Tool::Sccache => {
                print_info("Installing sccache...");
                let status = Command::new("cargo")
                    .args(&["install", "sccache", "--locked"])
                    .status()?;

                if status.success() {
                    print_success("sccache installed successfully");
                } else {
                    print_error("Failed to install sccache");
                }
            }
            Tool::Mold => {
                if cfg!(target_os = "linux") {
                    print_info("To install mold on Linux:");
                    println!("  Ubuntu/Debian: sudo apt-get install mold");
                    println!("  Fedora: sudo dnf install mold");
                    println!("  From source: https://github.com/rui314/mold");
                } else {
                    print_info("mold is only available on Linux");
                }
            }
            Tool::Lld => {
                print_info("To install lld:");
                if cfg!(target_os = "linux") {
                    println!("  Ubuntu/Debian: sudo apt-get install lld");
                    println!("  Fedora: sudo dnf install lld");
                } else if cfg!(target_os = "macos") {
                    println!("  macOS: brew install llvm");
                } else if cfg!(target_os = "windows") {
                    println!("  Windows: scoop install llvm");
                }
            }
            Tool::All => {} // Handled by iteration
        }
    }

    Ok(())
}

fn benchmark(path: Option<PathBuf>, iterations: u32) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;
    use std::time::Instant;

    let project_root = path.unwrap_or_else(|| PathBuf::from("."));

    print_info(&format!("Benchmarking with {} iterations...", iterations));

    // Clean before benchmarking
    Command::new("cargo")
        .current_dir(&project_root)
        .arg("clean")
        .status()?;

    // Benchmark without optimizations
    std::env::set_var("CARGO_OPTIMIZE_DISABLE", "1");
    print_info("Building without optimizations...");

    let mut baseline_times = Vec::new();
    for i in 1..=iterations {
        print_info(&format!("  Iteration {}/{}", i, iterations));
        Command::new("cargo")
            .current_dir(&project_root)
            .arg("clean")
            .status()?;

        let start = Instant::now();
        let status = Command::new("cargo")
            .current_dir(&project_root)
            .arg("build")
            .status()?;

        if !status.success() {
            return Err("Build failed".into());
        }

        baseline_times.push(start.elapsed());
    }

    // Benchmark with optimizations
    std::env::remove_var("CARGO_OPTIMIZE_DISABLE");

    // Apply optimizations
    let config = Config::new();
    let mut optimizer = Optimizer::with_config(&project_root, config)?;
    optimizer.optimize()?;

    print_info("Building with optimizations...");

    let mut optimized_times = Vec::new();
    for i in 1..=iterations {
        print_info(&format!("  Iteration {}/{}", i, iterations));
        Command::new("cargo")
            .current_dir(&project_root)
            .arg("clean")
            .status()?;

        let start = Instant::now();
        let status = Command::new("cargo")
            .current_dir(&project_root)
            .arg("build")
            .status()?;

        if !status.success() {
            return Err("Build failed".into());
        }

        optimized_times.push(start.elapsed());
    }

    // Calculate results
    let baseline_avg = baseline_times.iter().sum::<std::time::Duration>() / iterations;
    let optimized_avg = optimized_times.iter().sum::<std::time::Duration>() / iterations;
    let improvement = (baseline_avg.as_secs_f64() - optimized_avg.as_secs_f64())
        / baseline_avg.as_secs_f64()
        * 100.0;

    // Print results
    println!("\n{}", "=".repeat(60));
    println!("BENCHMARK RESULTS");
    println!("{}", "=".repeat(60));
    println!("\nBaseline (without optimizations):");
    println!("  Average: {:.2}s", baseline_avg.as_secs_f64());
    for (i, time) in baseline_times.iter().enumerate() {
        println!("  Run {}: {:.2}s", i + 1, time.as_secs_f64());
    }

    println!("\nOptimized:");
    println!("  Average: {:.2}s", optimized_avg.as_secs_f64());
    for (i, time) in optimized_times.iter().enumerate() {
        println!("  Run {}: {:.2}s", i + 1, time.as_secs_f64());
    }

    println!("\nImprovement: {:.1}% faster", improvement);
    println!("{}", "=".repeat(60));

    if improvement > 0.0 {
        print_success(&format!(
            "Optimizations improved build time by {:.1}%!",
            improvement
        ));
    } else {
        print_info(
            "No significant improvement detected. Your project may already be well-optimized.",
        );
    }

    Ok(())
}

fn reset(path: Option<PathBuf>, clean: bool) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    let project_root = path.unwrap_or_else(|| PathBuf::from("."));

    print_info("Resetting optimizations...");

    // Remove environment variables
    std::env::remove_var("CARGO_OPTIMIZE_ACTIVE");
    std::env::remove_var("RUSTC_WRAPPER");
    std::env::remove_var("CARGO_BUILD_JOBS");
    std::env::remove_var("CARGO_INCREMENTAL");
    std::env::remove_var("RUSTFLAGS");

    // Restore Cargo.toml from backup if it exists
    let cargo_toml = project_root.join("Cargo.toml");
    let backup = cargo_toml.with_extension("toml.backup");
    if backup.exists() {
        std::fs::copy(&backup, &cargo_toml)?;
        std::fs::remove_file(&backup)?;
        print_success("Restored Cargo.toml from backup");
    }

    // Remove .cargo/config.toml if we created it
    let cargo_config = project_root.join(".cargo").join("config.toml");
    if cargo_config.exists() {
        // Check if it contains our marker
        if let Ok(contents) = std::fs::read_to_string(&cargo_config) {
            if contents.contains("# Added by cargo-optimize") {
                std::fs::remove_file(&cargo_config)?;
                print_success("Removed .cargo/config.toml");
            }
        }
    }

    // Clean target directory if requested
    if clean {
        print_info("Cleaning target directory...");
        Command::new("cargo")
            .current_dir(&project_root)
            .arg("clean")
            .status()?;
        print_success("Target directory cleaned");
    }

    print_success("All optimizations have been reset");

    Ok(())
}

fn show_stats() -> Result<(), Box<dyn std::error::Error>> {
    use cargo_optimize::cache::{CacheConfig, CacheSystem};

    print_info("Cache Statistics:");

    let config = CacheConfig::new();

    if config.system == CacheSystem::None {
        print_info("No cache system is configured");
        return Ok(());
    }

    match config.get_stats() {
        Ok(stats) => {
            println!("  Cache hits: {}", stats.hits);
            println!("  Cache misses: {}", stats.misses);
            println!("  Hit rate: {:.1}%", stats.hit_rate());

            if stats.size_bytes > 0 {
                println!(
                    "  Cache size: {}",
                    cargo_optimize::utils::format_bytes(stats.size_bytes)
                );
            }
            if stats.file_count > 0 {
                println!("  Cached files: {}", stats.file_count);
            }
        }
        Err(e) => {
            print_error(&format!("Failed to get cache statistics: {}", e));
        }
    }

    Ok(())
}
