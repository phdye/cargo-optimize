//! Main optimizer implementation

use crate::{
    analyzer::ProjectAnalysis,
    cache::CacheConfig,
    config::{Config, OptimizationLevel},
    detector::Environment,
    error::{Error, Result},
    linker::LinkerConfig,
    profile::{ProfileManager, ProjectSize},
    utils::{self, print_info, print_success, print_warning},
};
use colored::Colorize;
use std::env;
use std::path::PathBuf;

/// Main optimizer struct
pub struct Optimizer {
    /// Project root directory
    project_root: PathBuf,
    /// Configuration
    config: Config,
    /// Environment information
    environment: Option<Environment>,
    /// Project analysis
    analysis: Option<ProjectAnalysis>,
}

impl Optimizer {
    /// Create a new optimizer with default configuration
    pub fn new(project_root: impl Into<PathBuf>) -> Result<Self> {
        let project_root = project_root.into();
        let config = Config::default();

        Ok(Self {
            project_root,
            config,
            environment: None,
            analysis: None,
        })
    }

    /// Create an optimizer with custom configuration
    pub fn with_config(project_root: impl Into<PathBuf>, config: Config) -> Result<Self> {
        let project_root = project_root.into();

        Ok(Self {
            project_root,
            config,
            environment: None,
            analysis: None,
        })
    }

    /// Run the optimization process
    pub fn optimize(&mut self) -> Result<()> {
        print_info("Starting cargo-optimize...");

        // Step 1: Detect environment
        if self.config.auto_detect_hardware {
            self.detect_environment()?;
        }

        // Step 2: Analyze project
        if self.config.analyze_project {
            self.analyze_project()?;
        }

        // Step 3: Apply optimizations
        self.apply_optimizations()?;

        // Step 4: Generate report
        self.generate_report();

        print_success("Optimization complete!");
        Ok(())
    }

    /// Detect environment and hardware
    fn detect_environment(&mut self) -> Result<()> {
        let spinner = utils::create_spinner("Detecting environment...");

        let env = Environment::detect()?;

        spinner.finish_with_message("Environment detected");

        // Print environment info if verbose
        if self.config.verbose {
            print_info(&format!(
                "CPU: {} cores ({} logical)",
                env.hardware.cpu_cores, env.hardware.logical_cpus
            ));
            print_info(&format!(
                "Memory: {}",
                utils::format_bytes(env.hardware.total_memory)
            ));
            print_info(&format!("OS: {:?}", env.hardware.os));
            print_info(&format!("Rust: {}", env.toolchain.rust_version));
        }

        // Adjust config based on environment
        if self.config.parallel_jobs.is_none() {
            self.config.parallel_jobs = Some(env.hardware.recommended_jobs());
        }

        // Adjust for CI environment
        if let Some(ci) = &env.ci_environment {
            print_info(&format!("CI environment detected: {:?}", ci));
            let ci_settings = ci.recommended_settings();
            self.config.parallel_jobs = Some(ci_settings.max_parallel_jobs);
        }

        self.environment = Some(env);
        Ok(())
    }

    /// Analyze the project structure
    fn analyze_project(&mut self) -> Result<()> {
        let spinner = utils::create_spinner("Analyzing project...");

        let analysis = ProjectAnalysis::analyze(&self.project_root)?;

        spinner.finish_with_message("Project analyzed");

        // Print analysis if verbose
        if self.config.verbose {
            print_info(&format!("Project: {}", analysis.metadata.name));
            print_info(&format!(
                "Lines of code: {}",
                analysis.code_stats.rust_lines
            ));
            print_info(&format!(
                "Dependencies: {} total ({} direct)",
                analysis.dependencies.total_dependencies, analysis.dependencies.direct_dependencies
            ));
            print_info(&format!(
                "Build complexity: {}/100",
                analysis.complexity.score
            ));
        }

        // Print recommendations
        if !analysis.recommendations.is_empty() {
            print_info("Recommendations:");
            for rec in &analysis.recommendations {
                print_info(&format!("  • {}", rec.description()));
            }
        }

        self.analysis = Some(analysis);
        Ok(())
    }

    /// Apply all optimizations
    fn apply_optimizations(&mut self) -> Result<()> {
        print_info("Applying optimizations...");

        let mut applied = Vec::new();
        let mut failed = Vec::new();

        // Apply based on optimization level
        match self.config.optimization_level {
            OptimizationLevel::Conservative => {
                self.apply_conservative_optimizations(&mut applied, &mut failed)?;
            }
            OptimizationLevel::Balanced => {
                self.apply_balanced_optimizations(&mut applied, &mut failed)?;
            }
            OptimizationLevel::Aggressive => {
                self.apply_aggressive_optimizations(&mut applied, &mut failed)?;
            }
            OptimizationLevel::Custom => {
                self.apply_custom_optimizations(&mut applied, &mut failed)?;
            }
        }

        // Print results
        for optimization in &applied {
            print_success(&format!("Applied: {}", optimization));
        }

        for (optimization, error) in &failed {
            print_warning(&format!("Failed to apply {}: {}", optimization, error));
        }

        // Set marker that optimizations are active
        env::set_var("CARGO_OPTIMIZE_ACTIVE", "1");

        Ok(())
    }

    /// Apply conservative optimizations
    fn apply_conservative_optimizations(
        &mut self,
        applied: &mut Vec<String>,
        failed: &mut Vec<(String, String)>,
    ) -> Result<()> {
        // 1. Configure linker (safe)
        if self.config.optimize_linker {
            match self.configure_linker() {
                Ok(linker) => applied.push(format!("Fast linker ({})", linker)),
                Err(e) => failed.push(("Fast linker".to_string(), e.to_string())),
            }
        }

        // 2. Enable caching (safe)
        if self.config.enable_cache {
            match self.configure_cache() {
                Ok(cache) => applied.push(format!("Build cache ({})", cache)),
                Err(e) => failed.push(("Build cache".to_string(), e.to_string())),
            }
        }

        // 3. Set parallel jobs (safe)
        if let Some(jobs) = self.config.parallel_jobs {
            self.set_parallel_jobs(jobs);
            applied.push(format!("Parallel jobs ({})", jobs));
        }

        Ok(())
    }

    /// Apply balanced optimizations
    fn apply_balanced_optimizations(
        &mut self,
        applied: &mut Vec<String>,
        failed: &mut Vec<(String, String)>,
    ) -> Result<()> {
        // Apply conservative first
        self.apply_conservative_optimizations(applied, failed)?;

        // 4. Configure build profiles
        match self.configure_profiles() {
            Ok(_) => applied.push("Optimized build profiles".to_string()),
            Err(e) => failed.push(("Build profiles".to_string(), e.to_string())),
        }

        // 5. Enable incremental compilation
        if self.config.incremental {
            env::set_var("CARGO_INCREMENTAL", "1");
            applied.push("Incremental compilation".to_string());
        }

        // 6. Split debuginfo
        if self.config.split_debuginfo {
            self.configure_split_debuginfo();
            applied.push("Split debuginfo".to_string());
        }

        Ok(())
    }

    /// Apply aggressive optimizations
    fn apply_aggressive_optimizations(
        &mut self,
        applied: &mut Vec<String>,
        failed: &mut Vec<(String, String)>,
    ) -> Result<()> {
        // Apply balanced first
        self.apply_balanced_optimizations(applied, failed)?;

        // 7. Native CPU optimizations
        if let Some(ref env) = self.environment {
            let target = env.hardware.cpu_target();
            env::set_var("CARGO_BUILD_TARGET_CPU", target);
            let mut rustflags = env::var("RUSTFLAGS").unwrap_or_default();
            rustflags.push_str(&format!(" -C target-cpu={}", target));
            env::set_var("RUSTFLAGS", rustflags);
            applied.push(format!("Native CPU optimization ({})", target));
        }

        // 8. Parallel frontend (nightly only)
        if utils::is_nightly() {
            env::set_var("CARGO_BUILD_JOBS", "default");
            env::set_var(
                "RUSTFLAGS",
                format!("{} -Z threads=0", env::var("RUSTFLAGS").unwrap_or_default()),
            );
            applied.push("Parallel frontend (nightly)".to_string());
        }

        // 9. Share generics
        let mut rustflags = env::var("RUSTFLAGS").unwrap_or_default();
        rustflags.push_str(" -Z share-generics=y");
        env::set_var("RUSTFLAGS", rustflags);
        applied.push("Share generics".to_string());

        Ok(())
    }

    /// Apply custom optimizations
    fn apply_custom_optimizations(
        &mut self,
        applied: &mut Vec<String>,
        failed: &mut Vec<(String, String)>,
    ) -> Result<()> {
        // Apply individual settings based on config

        if self.config.optimize_linker {
            match self.configure_linker() {
                Ok(linker) => applied.push(format!("Fast linker ({})", linker)),
                Err(e) => failed.push(("Fast linker".to_string(), e.to_string())),
            }
        }

        if self.config.enable_cache {
            match self.configure_cache() {
                Ok(cache) => applied.push(format!("Build cache ({})", cache)),
                Err(e) => failed.push(("Build cache".to_string(), e.to_string())),
            }
        }

        if let Some(jobs) = self.config.parallel_jobs {
            self.set_parallel_jobs(jobs);
            applied.push(format!("Parallel jobs ({})", jobs));
        }

        if self.config.incremental {
            env::set_var("CARGO_INCREMENTAL", "1");
            applied.push("Incremental compilation".to_string());
        }

        // Apply extra flags
        if !self.config.extra_cargo_flags.is_empty() {
            let flags = self.config.extra_cargo_flags.join(" ");
            env::set_var("CARGO_BUILD_FLAGS", &flags);
            applied.push(format!("Extra cargo flags: {}", flags));
        }

        if !self.config.extra_rustc_flags.is_empty() {
            let mut rustflags = env::var("RUSTFLAGS").unwrap_or_default();
            for flag in &self.config.extra_rustc_flags {
                rustflags.push(' ');
                rustflags.push_str(flag);
            }
            env::set_var("RUSTFLAGS", rustflags);
            applied.push("Extra rustc flags".to_string());
        }

        Ok(())
    }

    /// Configure the linker
    fn configure_linker(&self) -> Result<String> {
        if self.config.dry_run {
            return Ok("dry-run".to_string());
        }

        let config = LinkerConfig::auto_detect()?;
        config.apply()?;

        Ok(config.linker.executable())
    }

    /// Configure build caching
    fn configure_cache(&self) -> Result<String> {
        if self.config.dry_run {
            return Ok("dry-run".to_string());
        }

        let config = CacheConfig::auto_detect()?;

        // Try to install if needed
        config.install_if_needed().ok();

        config.apply()?;

        Ok(config
            .system
            .wrapper_command()
            .unwrap_or_else(|| "none".to_string()))
    }

    /// Configure build profiles
    fn configure_profiles(&self) -> Result<()> {
        if self.config.dry_run {
            return Ok(());
        }

        let cargo_toml = self.project_root.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Err(Error::invalid_project("Cargo.toml not found"));
        }

        // Backup Cargo.toml
        utils::backup_file(&cargo_toml)?;

        // Determine project size
        let project_size = if let Some(ref analysis) = self.analysis {
            ProjectSize::from_lines(analysis.code_stats.rust_lines)
        } else {
            ProjectSize::Medium
        };

        // Create profile manager
        let mut manager = ProfileManager::new();

        // Get recommended profiles
        let is_ci = utils::is_ci();
        let profiles = ProfileManager::recommend_profiles(project_size, is_ci);

        for profile in profiles.into_values() {
            manager.set_profile(profile);
        }

        // Apply to Cargo.toml
        manager.apply_to_cargo_toml(cargo_toml)?;

        Ok(())
    }

    /// Set parallel jobs
    fn set_parallel_jobs(&self, jobs: usize) {
        env::set_var("CARGO_BUILD_JOBS", jobs.to_string());

        // Also set for Make and other build systems
        env::set_var("MAKEFLAGS", format!("-j{}", jobs));
    }

    /// Configure split debuginfo
    fn configure_split_debuginfo(&self) {
        let mut rustflags = env::var("RUSTFLAGS").unwrap_or_default();

        if cfg!(target_os = "macos") {
            rustflags.push_str(" -C split-debuginfo=unpacked");
        } else if cfg!(target_os = "linux") {
            rustflags.push_str(" -C split-debuginfo=off");
        }

        env::set_var("RUSTFLAGS", rustflags);
    }

    /// Generate optimization report
    fn generate_report(&self) {
        println!("\n{}", "=".repeat(60));
        println!("{}", "OPTIMIZATION REPORT".bold());
        println!("{}", "=".repeat(60));

        // Environment
        if let Some(ref env) = self.environment {
            println!("\n{}:", "Environment".bold());
            println!(
                "  CPU: {} cores ({} logical)",
                env.hardware.cpu_cores, env.hardware.logical_cpus
            );
            println!(
                "  Memory: {}",
                utils::format_bytes(env.hardware.total_memory)
            );
            println!("  OS: {:?}", env.hardware.os);
            println!("  Rust: {}", env.toolchain.rust_version);
        }

        // Project
        if let Some(ref analysis) = self.analysis {
            println!("\n{}:", "Project".bold());
            println!("  Name: {}", analysis.metadata.name);
            println!("  Lines of code: {}", analysis.code_stats.rust_lines);
            println!(
                "  Dependencies: {}",
                analysis.dependencies.total_dependencies
            );
            println!("  Complexity: {}/100", analysis.complexity.score);

            if analysis.is_workspace() {
                println!("  Workspace members: {}", analysis.crate_count());
            }
        }

        // Configuration
        println!("\n{}:", "Configuration".bold());
        println!("  Optimization level: {:?}", self.config.optimization_level);
        println!(
            "  Parallel jobs: {}",
            self.config
                .parallel_jobs
                .map_or("auto".to_string(), |j| j.to_string())
        );

        // Tips
        println!("\n{}:", "Next Steps".bold());
        println!("  • Run 'cargo build' to see the improvements");
        println!("  • Use 'cargo clean' if you encounter any issues");
        println!("  • Set CARGO_OPTIMIZE_DISABLE=1 to temporarily disable");

        println!("\n{}", "=".repeat(60));
    }
}
