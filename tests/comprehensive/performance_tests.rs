//! Performance tests and benchmarks for cargo-optimize

use cargo_optimize::{analyzer::*, detector::*, Config, OptimizationFeature, OptimizationLevel};

use criterion::black_box;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::TempDir;


// Benchmark configuration creation
#[test]
fn benchmark_config_creation() {
    let start = Instant::now();
    let iterations = 10_000;

    for _ in 0..iterations {
        let config = black_box(Config::new());
        black_box(config);
    }

    let duration = start.elapsed();
    let per_iteration = duration / iterations;

    println!("Config creation: {:?} per iteration", per_iteration);
    assert!(
        per_iteration < Duration::from_micros(100),
        "Config creation too slow: {:?}",
        per_iteration
    );
}

#[test]
fn benchmark_config_builder() {
    let start = Instant::now();
    let iterations = 1_000;

    for _ in 0..iterations {
        let mut config = Config::new();
        let result = config
            .set_optimization_level(OptimizationLevel::Aggressive)
            .set_parallel_jobs(8)
            .verbose()
            .dry_run();
        black_box(result);
    }

    let duration = start.elapsed();
    let per_iteration = duration / iterations;

    println!("Config builder chain: {:?} per iteration", per_iteration);
    assert!(
        per_iteration < Duration::from_millis(1),
        "Config builder too slow: {:?}",
        per_iteration
    );
}

#[test]
fn benchmark_hardware_detection() {
    let detector = SystemDetector::new();

    // Warm up
    let _ = detector.detect_all();

    let start = Instant::now();
    let iterations = 100;

    for _ in 0..iterations {
        let hardware = detector.detect_all();
        black_box(hardware);
    }

    let duration = start.elapsed();
    let per_iteration = duration / iterations;

    println!("Hardware detection: {:?} per iteration", per_iteration);
    // Hardware detection can be slower, but should be under 100ms
    assert!(
        per_iteration < Duration::from_millis(100),
        "Hardware detection too slow: {:?}",
        per_iteration
    );
}

#[test]
fn benchmark_cpu_detection() {
    let detector = SystemDetector::new();

    let start = Instant::now();
    let iterations = 1_000;

    for _ in 0..iterations {
        let cpu = detector.detect_cpu();
        black_box(cpu);
    }

    let duration = start.elapsed();
    let per_iteration = duration / iterations;

    println!("CPU detection: {:?} per iteration", per_iteration);
    assert!(
        per_iteration < Duration::from_millis(10),
        "CPU detection too slow: {:?}",
        per_iteration
    );
}

#[test]
fn benchmark_memory_detection() {
    let detector = SystemDetector::new();

    let start = Instant::now();
    let iterations = 1_000;

    for _ in 0..iterations {
        let memory = detector.detect_memory();
        black_box(memory);
    }

    let duration = start.elapsed();
    let per_iteration = duration / iterations;

    println!("Memory detection: {:?} per iteration", per_iteration);
    assert!(
        per_iteration < Duration::from_millis(10),
        "Memory detection too slow: {:?}",
        per_iteration
    );
}

#[test]
fn benchmark_project_analysis() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create a test project
    let cargo_toml = r#"
[package]
name = "bench-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
"#;
    fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();

    let src_dir = project_root.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    // Create multiple source files
    for i in 0..10 {
        let content = format!("// File {}\npub fn func_{}() {{}}\n", i, i);
        fs::write(src_dir.join(format!("mod{}.rs", i)), content).unwrap();
    }

    fs::write(src_dir.join("lib.rs"), "// Main lib file\n").unwrap();

    let start = Instant::now();
    let iterations = 10;

    for _ in 0..iterations {
        let analysis = ProjectAnalysis::analyze(project_root).unwrap();
        black_box(analysis);
    }

    let duration = start.elapsed();
    let per_iteration = duration / iterations;

    println!("Project analysis: {:?} per iteration", per_iteration);
    // Project analysis involves file I/O, so allow more time
    assert!(
        per_iteration < Duration::from_secs(1),
        "Project analysis too slow: {:?}",
        per_iteration
    );
}

#[test]
fn benchmark_code_stats_calculation() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create test files
    let src_dir = project_root.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    for i in 0..20 {
        let content = "fn main() {}\n".repeat(100);
        fs::write(src_dir.join(format!("file{}.rs", i)), content).unwrap();
    }

    let start = Instant::now();
    let iterations = 50;

    for _ in 0..iterations {
        let stats = CodeStats::calculate(project_root).unwrap();
        black_box(stats);
    }

    let duration = start.elapsed();
    let per_iteration = duration / iterations;

    println!("Code stats calculation: {:?} per iteration", per_iteration);
    assert!(
        per_iteration < Duration::from_millis(100),
        "Code stats calculation too slow: {:?}",
        per_iteration
    );
}

#[test]
fn benchmark_complexity_calculation() {
    // Create a test project to get real metadata
    let test_dir = TempDir::new().unwrap();
    let test_cargo_toml = r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(test_dir.path().join("Cargo.toml"), test_cargo_toml).unwrap();
    fs::create_dir_all(test_dir.path().join("src")).unwrap();
    fs::write(test_dir.path().join("src/lib.rs"), "").unwrap();
    let real_metadata = ProjectMetadata::load(test_dir.path()).unwrap();
    
    let metadata = ProjectMetadata {
        name: "test".to_string(),
        version: "0.1.0".to_string(),
        root_path: PathBuf::from("."),
        is_workspace: false,
        workspace_members: vec![],
        cargo_metadata: real_metadata.cargo_metadata,
    };

    let code_stats = CodeStats {
        total_lines: 10_000,
        rust_lines: 8_000,
        rust_files: 50,
        test_lines: 2_000,
        test_files: 10,
        bench_lines: 500,
        bench_files: 5,
        example_lines: 300,
        example_files: 3,
    };

    let dependencies = DependencyAnalysis {
        total_dependencies: 100,
        direct_dependencies: 20,
        transitive_dependencies: 80,
        proc_macro_count: 10,
        categories: Default::default(),
        heavy_dependencies: vec!["tokio".to_string()],
        has_heavy_dependencies: true,
        duplicates: vec![],
    };

    let start = Instant::now();
    let iterations = 10_000;

    for _ in 0..iterations {
        let complexity = BuildComplexity::calculate(&metadata, &code_stats, &dependencies);
        black_box(complexity);
    }

    let duration = start.elapsed();
    let per_iteration = duration / iterations;

    println!("Complexity calculation: {:?} per iteration", per_iteration);
    assert!(
        per_iteration < Duration::from_micros(100),
        "Complexity calculation too slow: {:?}",
        per_iteration
    );
}

#[test]
fn benchmark_config_serialization() {
    let mut config = Config::new();
    config
        .set_optimization_level(OptimizationLevel::Aggressive)
        .set_parallel_jobs(8)
        .verbose();

    for i in 0..20 {
        config.extra_cargo_flags.push(format!("--flag-{}", i));
        config
            .extra_rustc_flags
            .push(format!("-C opt-level={}", i % 4));
    }

    let start = Instant::now();
    let iterations = 1_000;

    for _ in 0..iterations {
        let serialized = toml::to_string(&config).unwrap();
        black_box(serialized);
    }

    let duration = start.elapsed();
    let per_iteration = duration / iterations;

    println!("Config serialization: {:?} per iteration", per_iteration);
    assert!(
        per_iteration < Duration::from_millis(1),
        "Config serialization too slow: {:?}",
        per_iteration
    );
}

#[test]
fn benchmark_config_deserialization() {
    let mut config = Config::new();
    config
        .set_optimization_level(OptimizationLevel::Aggressive)
        .set_parallel_jobs(8);

    let serialized = toml::to_string(&config).unwrap();

    let start = Instant::now();
    let iterations = 1_000;

    for _ in 0..iterations {
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        black_box(deserialized);
    }

    let duration = start.elapsed();
    let per_iteration = duration / iterations;

    println!("Config deserialization: {:?} per iteration", per_iteration);
    assert!(
        per_iteration < Duration::from_millis(1),
        "Config deserialization too slow: {:?}",
        per_iteration
    );
}

#[test]
fn benchmark_file_operations() {
    let temp_dir = TempDir::new().unwrap();

    let start = Instant::now();
    let iterations = 100;

    for i in 0..iterations {
        let config = Config::new();
        let path = temp_dir.path().join(format!("config_{}.toml", i));

        config.save(&path).unwrap();
        let loaded = Config::from_file(&path).unwrap();
        black_box(loaded);
    }

    let duration = start.elapsed();
    let per_iteration = duration / iterations;

    println!("File save/load: {:?} per iteration", per_iteration);
    assert!(
        per_iteration < Duration::from_millis(10),
        "File operations too slow: {:?}",
        per_iteration
    );
}

#[test]
fn benchmark_large_project_analysis() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create a large project structure
    let cargo_toml = r#"
[workspace]
members = ["crate1", "crate2", "crate3", "crate4", "crate5"]

[workspace.dependencies]
serde = "1.0"
tokio = "1.0"
"#;
    fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();

    // Create member crates with many files
    for crate_num in 1..=5 {
        let crate_dir = project_root.join(format!("crate{}", crate_num));
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let crate_toml = format!(
            r#"
[package]
name = "crate{}"
version = "0.1.0"
edition = "2021"

[dependencies]
serde.workspace = true
"#,
            crate_num
        );
        fs::write(crate_dir.join("Cargo.toml"), crate_toml).unwrap();

        // Create many source files
        for i in 0..20 {
            let content =
                format!("// Crate {} File {}\n", crate_num, i) + &"pub fn test() {}\n".repeat(100);
            fs::write(src_dir.join(format!("mod{}.rs", i)), content).unwrap();
        }

        fs::write(src_dir.join("lib.rs"), "// Lib file\n").unwrap();
    }

    let start = Instant::now();
    let analysis = ProjectAnalysis::analyze(project_root).unwrap();
    let duration = start.elapsed();

    println!("Large project analysis took: {:?}", duration);
    println!("  - {} lines of code", analysis.code_stats.rust_lines);
    println!("  - {} files", analysis.code_stats.rust_files);
    println!("  - {} crates", analysis.crate_count());

    assert!(
        duration < Duration::from_secs(5),
        "Large project analysis too slow: {:?}",
        duration
    );
}

#[test]
fn benchmark_optimization_features() {
    let features = vec![
        OptimizationFeature::FastLinker,
        OptimizationFeature::Incremental,
        OptimizationFeature::ParallelFrontend,
        OptimizationFeature::SplitDebuginfo,
        OptimizationFeature::Sccache,
        OptimizationFeature::NativeCpu,
        OptimizationFeature::ThinLto,
    ];

    let levels = vec![
        OptimizationLevel::Conservative,
        OptimizationLevel::Balanced,
        OptimizationLevel::Aggressive,
        OptimizationLevel::Custom,
    ];

    let start = Instant::now();
    let iterations = 10_000;

    for _ in 0..iterations {
        for level in &levels {
            for feature in &features {
                let enabled = level.should_enable(*feature);
                black_box(enabled);
            }
        }
    }

    let duration = start.elapsed();
    let per_check = duration / (iterations * levels.len() as u32 * features.len() as u32);

    println!("Feature check: {:?} per check", per_check);
    assert!(
        per_check < Duration::from_nanos(100),
        "Feature checking too slow: {:?}",
        per_check
    );
}

#[test]
fn benchmark_concurrent_config_access() {
    use std::sync::Arc;
    use std::thread;

    let config = Arc::new(Config::new());
    let start = Instant::now();
    let thread_count = 8;
    let iterations_per_thread = 1_000;

    let mut handles = Vec::new();

    for _ in 0..thread_count {
        let config_clone = Arc::clone(&config);
        let handle = thread::spawn(move || {
            for _ in 0..iterations_per_thread {
                // Read various fields
                let _ = config_clone.optimization_level;
                let _ = config_clone.parallel_jobs;
                let _ = config_clone.verbose;

                // Serialize
                let _ = toml::to_string(&*config_clone).unwrap();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    let total_operations = thread_count * iterations_per_thread;
    let per_operation = duration / total_operations;

    println!(
        "Concurrent config access: {:?} per operation",
        per_operation
    );
    assert!(
        per_operation < Duration::from_micros(100),
        "Concurrent access too slow: {:?}",
        per_operation
    );
}

#[test]
fn benchmark_memory_usage() {
    use std::mem;

    // Measure size of various structs
    println!("Memory usage:");
    println!("  Config: {} bytes", mem::size_of::<Config>());
    println!("  CodeStats: {} bytes", mem::size_of::<CodeStats>());
    println!(
        "  DependencyAnalysis: {} bytes",
        mem::size_of::<DependencyAnalysis>()
    );
    println!(
        "  BuildComplexity: {} bytes",
        mem::size_of::<BuildComplexity>()
    );
    println!(
        "  ProjectMetadata: {} bytes",
        mem::size_of::<ProjectMetadata>()
    );

    // Ensure reasonable memory usage
    assert!(mem::size_of::<Config>() < 1024, "Config struct too large");
    assert!(
        mem::size_of::<CodeStats>() < 256,
        "CodeStats struct too large"
    );
}

#[test]
fn benchmark_recommendation_generation() {
    let complexity = BuildComplexity {
        score: 70,
        is_large_project: true,
        is_complex: true,
        estimated_build_time: 200,
        test_ratio: 0.4,
        factors: vec![
            ComplexityFactor::LargeCodebase,
            ComplexityFactor::ManyDependencies,
        ],
    };

    let dependencies = DependencyAnalysis {
        total_dependencies: 150,
        direct_dependencies: 30,
        transitive_dependencies: 120,
        proc_macro_count: 15,
        categories: Default::default(),
        heavy_dependencies: vec!["tokio".to_string()],
        has_heavy_dependencies: true,
        duplicates: vec![],
    };

    let start = Instant::now();
    let iterations = 10_000;

    for _ in 0..iterations {
        let recommendations = ProjectAnalysis::generate_recommendations(&complexity, &dependencies);
        black_box(recommendations);
    }

    let duration = start.elapsed();
    let per_iteration = duration / iterations;

    println!(
        "Recommendation generation: {:?} per iteration",
        per_iteration
    );
    assert!(
        per_iteration < Duration::from_micros(100),
        "Recommendation generation too slow: {:?}",
        per_iteration
    );
}

// Stress test for sustained performance
#[test]
fn stress_test_sustained_performance() {
    let temp_dir = TempDir::new().unwrap();
    let mut timings = Vec::new();

    for i in 0..100 {
        let start = Instant::now();

        // Create and analyze a project
        let project_dir = temp_dir.path().join(format!("project_{}", i));
        fs::create_dir_all(&project_dir).unwrap();

        let cargo_toml = format!(
            r#"
[package]
name = "project_{}"
version = "0.1.0"
edition = "2021"
"#,
            i
        );
        fs::write(project_dir.join("Cargo.toml"), cargo_toml).unwrap();

        let src_dir = project_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("lib.rs"), "// Lib\n").unwrap();

        let _analysis = ProjectAnalysis::analyze(&project_dir);

        let duration = start.elapsed();
        timings.push(duration);
    }

    // Check for performance degradation
    let first_10_avg: Duration = timings.iter().take(10).sum::<Duration>() / 10;
    let last_10_avg: Duration = timings.iter().skip(90).sum::<Duration>() / 10;

    println!("First 10 average: {:?}", first_10_avg);
    println!("Last 10 average: {:?}", last_10_avg);

    // Performance should not degrade significantly
    assert!(
        last_10_avg < first_10_avg * 2,
        "Performance degraded: first {:?}, last {:?}",
        first_10_avg,
        last_10_avg
    );
}
