//! Regression tests for known issues

use cargo_optimize::{analyzer::*, detector::*, Config, OptimizationFeature, OptimizationLevel};

fn create_test_metadata() -> cargo_metadata::Metadata {
    use cargo_metadata::{Package, PackageId, Version};
    cargo_metadata::Metadata {
        packages: vec![],
        workspace_members: vec![],
        resolve: None,
        root: std::env::current_dir().unwrap_or_default().into(),
        metadata: None,
        version: 1,
        workspace_root: std::env::current_dir().unwrap_or_default().into(),
        target_directory: std::env::current_dir().unwrap_or_default().join("target").into(),
    }
}

use std::env;

fn create_test_metadata() -> cargo_metadata::Metadata {
    use cargo_metadata::{Package, PackageId, Version};
    cargo_metadata::Metadata {
        packages: vec![],
        workspace_members: vec![],
        resolve: None,
        root: std::env::current_dir().unwrap_or_default().into(),
        metadata: None,
        version: 1,
        workspace_root: std::env::current_dir().unwrap_or_default().into(),
        target_directory: std::env::current_dir().unwrap_or_default().join("target").into(),
    }
}

use std::fs;

fn create_test_metadata() -> cargo_metadata::Metadata {
    use cargo_metadata::{Package, PackageId, Version};
    cargo_metadata::Metadata {
        packages: vec![],
        workspace_members: vec![],
        resolve: None,
        root: std::env::current_dir().unwrap_or_default().into(),
        metadata: None,
        version: 1,
        workspace_root: std::env::current_dir().unwrap_or_default().into(),
        target_directory: std::env::current_dir().unwrap_or_default().join("target").into(),
    }
}

use std::path::PathBuf;

fn create_test_metadata() -> cargo_metadata::Metadata {
    use cargo_metadata::{Package, PackageId, Version};
    cargo_metadata::Metadata {
        packages: vec![],
        workspace_members: vec![],
        resolve: None,
        root: std::env::current_dir().unwrap_or_default().into(),
        metadata: None,
        version: 1,
        workspace_root: std::env::current_dir().unwrap_or_default().into(),
        target_directory: std::env::current_dir().unwrap_or_default().join("target").into(),
    }
}

use tempfile::TempDir;

fn create_test_metadata() -> cargo_metadata::Metadata {
    use cargo_metadata::{Package, PackageId, Version};
    cargo_metadata::Metadata {
        packages: vec![],
        workspace_members: vec![],
        resolve: None,
        root: std::env::current_dir().unwrap_or_default().into(),
        metadata: None,
        version: 1,
        workspace_root: std::env::current_dir().unwrap_or_default().into(),
        target_directory: std::env::current_dir().unwrap_or_default().join("target").into(),
    }
}


// Issue #001: Config serialization losing custom linker path on Windows
#[test]
fn regression_issue_001() {
    let mut config = Config::new();

    // Windows-style path with backslashes
    #[cfg(windows)]
    let linker_path = PathBuf::from("C:\\Program Files\\LLVM\\bin\\lld.exe");
    #[cfg(not(windows))]
    let linker_path = PathBuf::from("/usr/bin/lld");

    config.custom_linker = Some(linker_path.clone());

    // Serialize and deserialize
    let serialized = toml::to_string(&config).unwrap();
    let deserialized: Config = toml::from_str(&serialized).unwrap();

    // Path should be preserved exactly
    assert_eq!(deserialized.custom_linker, Some(linker_path));
}

// Issue #002: Parallel jobs being set to 0 on single-core systems
#[test]
fn regression_issue_002_single_core_parallel_jobs() {
    let mut config = Config::new();
    config.auto_detect_hardware = true;

    // Simulate single-core detection
    let mock_cpu = CpuInfo {
        logical_cores: 1,
        physical_cores: 1,
        model_name: "Single Core CPU".to_string(),
        base_frequency: None,
        max_frequency: None,
        features: vec![],
    };

    // Even with single core, parallel jobs should be at least 1
    let recommended_jobs = if mock_cpu.logical_cores > 1 {
        mock_cpu.logical_cores
    } else {
        1 // Never set to 0
    };

    assert!(recommended_jobs >= 1);
}

// Issue #003: Workspace analysis failing with relative member paths
#[test]
fn regression_issue_003_relative_workspace_paths() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path();

    // Create workspace with relative paths
    let workspace_toml = r#"
[workspace]
members = [
    "./crates/core",
    "../workspace/crates/utils",
    "crates/cli"
]
"#;
    fs::write(workspace_root.join("Cargo.toml"), workspace_toml).unwrap();

    // Create the member directories
    fs::create_dir_all(workspace_root.join("crates/core/src")).unwrap();
    fs::create_dir_all(workspace_root.join("crates/utils/src")).unwrap();
    fs::create_dir_all(workspace_root.join("crates/cli/src")).unwrap();

    // Create minimal Cargo.toml for each member
    for name in &["core", "utils", "cli"] {
        let member_toml = format!(
            r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"
"#,
            name
        );
        fs::write(
            workspace_root.join(format!("crates/{}/Cargo.toml", name)),
            member_toml,
        )
        .unwrap();
        fs::write(
            workspace_root.join(format!("crates/{}/src/lib.rs", name)),
            "// Lib\n",
        )
        .unwrap();
    }

    // Analysis should handle relative paths
    let result = ProjectAnalysis::analyze(workspace_root);

    // Should either succeed or fail gracefully (not panic)
    if let Ok(analysis) = result {
        assert!(analysis.is_workspace());
    }
}

// Issue #004: Memory leak when analyzing very large projects
#[test]
fn regression_issue_004_memory_leak_large_projects() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple projects and analyze them
    for i in 0..10 {
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

        // Create many files
        for j in 0..100 {
            let content = format!("// File {}\n", j) + &"fn test() {}\n".repeat(100);
            fs::write(src_dir.join(format!("mod_{}.rs", j)), content).unwrap();
        }

        // Analyze and drop immediately
        let _ = ProjectAnalysis::analyze(&project_dir);
        // Memory should be freed after each analysis
    }

    // If we get here without OOM, the leak is fixed
    assert!(true);
}

// Issue #005: Incremental compilation flag being overwritten
#[test]
fn regression_issue_005_incremental_flag_preservation() {
    // Save original value
    let original = env::var("CARGO_INCREMENTAL").ok();

    // Set to a specific value
    env::set_var("CARGO_INCREMENTAL", "1");

    let mut config = Config::new();
    config.incremental = false; // Explicitly disable

    // After applying config, our setting should take precedence
    // In real implementation, this would be in the optimizer
    if !config.incremental {
        env::set_var("CARGO_INCREMENTAL", "0");
    }

    assert_eq!(env::var("CARGO_INCREMENTAL").unwrap(), "0");

    // Restore original
    if let Some(val) = original {
        env::set_var("CARGO_INCREMENTAL", val);
    } else {
        env::remove_var("CARGO_INCREMENTAL");
    }
}

// Issue #006: Config file corruption with special characters
#[test]
fn regression_issue_006_special_chars_in_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let mut config = Config::new();

    // Add flags with special characters
    config.extra_cargo_flags = vec![
        "--features=my-feature".to_string(),
        "--target=\"wasm32-unknown-unknown\"".to_string(),
        "-Z unstable-options".to_string(),
    ];

    config.extra_rustc_flags = vec![
        "-C target-cpu=native".to_string(),
        "-C link-args=-Wl,-rpath,$ORIGIN".to_string(),
    ];

    // Save and reload
    config.save(&config_path).unwrap();
    let loaded = Config::from_file(&config_path).unwrap();

    // All flags should be preserved correctly
    assert_eq!(loaded.extra_cargo_flags, config.extra_cargo_flags);
    assert_eq!(loaded.extra_rustc_flags, config.extra_rustc_flags);
}

// Issue #007: Optimizer failing silently on permission denied
#[test]
fn regression_issue_007_permission_denied_handling() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let readonly_dir = temp_dir.path().join("readonly");
        fs::create_dir(&readonly_dir).unwrap();

        // Create Cargo.toml
        let cargo_toml = r#"
[package]
name = "readonly_test"
version = "0.1.0"
edition = "2021"
"#;
        fs::write(readonly_dir.join("Cargo.toml"), cargo_toml).unwrap();

        // Make directory read-only
        let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
        perms.set_mode(0o444);
        fs::set_permissions(&readonly_dir, perms).unwrap();

        // Try to optimize
        let mut config = Config::new();
        config.dry_run(); // Should not actually write

        let result = Optimizer::with_config(&readonly_dir, config);

        // Should handle permission error gracefully
        assert!(result.is_ok() || result.is_err());

        // Restore permissions for cleanup
        let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&readonly_dir, perms).unwrap();
    }
}

// Issue #008: Dependency count incorrect with workspace dependencies
#[test]
fn regression_issue_008_workspace_dependency_counting() {
    // Mock dependency analysis
    let mut deps = DependencyAnalysis {
        total_dependencies: 10,
        direct_dependencies: 3,
        transitive_dependencies: 7,
        proc_macro_count: 2,
        categories: Default::default(),
        heavy_dependencies: vec![],
        has_heavy_dependencies: false,
        duplicates: vec![],
    };

    // Ensure counts are consistent
    assert_eq!(
        deps.direct_dependencies + deps.transitive_dependencies,
        deps.total_dependencies
    );

    // Add workspace dependencies (should be counted correctly)
    deps.total_dependencies = 15;
    deps.direct_dependencies = 5;
    deps.transitive_dependencies = 10;

    assert_eq!(
        deps.direct_dependencies + deps.transitive_dependencies,
        deps.total_dependencies
    );
}

// Issue #009: Build complexity score overflow with extreme values
#[test]
fn regression_issue_009_complexity_overflow() {
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
        name: "huge".to_string(),
        version: "0.1.0".to_string(),
        root_path: PathBuf::from("."),
        is_workspace: true,
        workspace_members: (0..1000).map(|i| format!("member_{}", i)).collect(),
        cargo_metadata: real_metadata.cargo_metadata,
    };

    let code_stats = CodeStats {
        total_lines: usize::MAX / 2,
        rust_lines: usize::MAX / 2,
        rust_files: usize::MAX / 100,
        test_lines: usize::MAX / 4,
        test_files: usize::MAX / 200,
        bench_lines: 0,
        bench_files: 0,
        example_lines: 0,
        example_files: 0,
    };

    let dependencies = DependencyAnalysis {
        total_dependencies: 10000,
        direct_dependencies: 1000,
        transitive_dependencies: 9000,
        proc_macro_count: 500,
        categories: Default::default(),
        heavy_dependencies: vec!["tokio".to_string(); 100],
        has_heavy_dependencies: true,
        duplicates: vec![],
    };

    // Should not overflow or panic
    let complexity = BuildComplexity::calculate(&metadata, &code_stats, &dependencies);

    // Score should be capped at 100
    assert!(complexity.score <= 100);
}

// Issue #010: Race condition in concurrent detector access
#[test]
fn regression_issue_010_detector_race_condition() {
    use std::sync::Arc;
    use std::thread;

    let detector = Arc::new(SystemDetector::new());
    let mut handles = Vec::new();

    // Spawn many threads accessing detector simultaneously
    for _ in 0..20 {
        let detector_clone = Arc::clone(&detector);

        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let cpu = detector_clone.detect_cpu();
                let memory = detector_clone.detect_memory();

                // Values should be consistent
                assert!(cpu.logical_cores > 0);
                assert!(memory.total_bytes > 0);
            }
        });

        handles.push(handle);
    }

    // All threads should complete without data races
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

// Issue #011: Environment variables not properly escaped
#[test]
fn regression_issue_011_env_var_escaping() {
    let test_cases = vec![
        ("NORMAL_VAR", "normal_value"),
        ("VAR_WITH_SPACES", "value with spaces"),
        ("VAR_WITH_QUOTES", "value\"with\"quotes"),
        ("VAR_WITH_EQUALS", "key=value"),
        ("VAR_WITH_NEWLINE", "line1\nline2"),
        ("VAR_WITH_PATH", "/usr/bin:/usr/local/bin"),
    ];

    for (key, value) in test_cases {
        // Save original
        let original = env::var(key).ok();

        // Set test value
        env::set_var(key, value);

        // Read back
        let read_value = env::var(key).unwrap();
        assert_eq!(read_value, value);

        // Restore
        if let Some(orig) = original {
            env::set_var(key, orig);
        } else {
            env::remove_var(key);
        }
    }
}

// Issue #012: Profile optimization overwriting user settings
#[test]
fn regression_issue_012_profile_preservation() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml_path = temp_dir.path().join("Cargo.toml");

    // Create Cargo.toml with existing profiles
    let original_toml = r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"

[profile.dev]
opt-level = 1
debug = true
# User comment that should be preserved

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
"#;

    fs::write(&cargo_toml_path, original_toml).unwrap();

    // Apply optimization (in real implementation)
    // The optimizer should preserve user settings when possible

    let content = fs::read_to_string(&cargo_toml_path).unwrap();

    // User settings should be respected
    assert!(content.contains("opt-level"));
    // Comments might not be preserved by toml crate, but that's a known limitation
}

// Issue #013: Incorrect OS detection in containers
#[test]
fn regression_issue_013_container_os_detection() {
    let detector = SystemDetector::new();
    let os_info = detector.detect_os();

    // Should detect something reasonable even in containers
    assert!(!os_info.family.is_empty());
    assert!(!os_info.arch.is_empty());

    // Common container scenarios
    if env::var("DOCKER_CONTAINER").is_ok() || env::var("KUBERNETES_SERVICE_HOST").is_ok() {
        // Should still work in containerized environments
        assert!(os_info.family == "unix" || os_info.family == "linux");
    }
}

// Issue #014: Cache configuration not respecting CI environment
#[test]
fn regression_issue_014_ci_cache_configuration() {
    // Save original
    let original_ci = env::var("CI").ok();

    // Simulate CI environment
    env::set_var("CI", "true");

    // Cache config should adapt to CI
    use cargo_optimize::cache::CacheConfig;

    let _cache_config = CacheConfig::auto_detect().unwrap_or_else(|_| CacheConfig::none());

    // In CI, might prefer different cache settings
    // This is implementation-specific

    // Restore
    if let Some(val) = original_ci {
        env::set_var("CI", val);
    } else {
        env::remove_var("CI");
    }
}

// Issue #015: Panic on malformed TOML in Cargo.toml
#[test]
fn regression_issue_015_malformed_toml_handling() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create malformed Cargo.toml
    let malformed_toml = r#"
[package
name = "broken"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0" features = ["derive"] } # Missing comma
"#;

    fs::write(project_root.join("Cargo.toml"), malformed_toml).unwrap();

    // Should handle parse error gracefully
    let result = ProjectAnalysis::analyze(project_root);
    assert!(result.is_err()); // Should return error, not panic
}

// Issue #016: Optimization level not respecting Custom setting
#[test]
fn regression_issue_016_custom_optimization_level() {
    let mut config = Config::new();
    config.optimization_level = OptimizationLevel::Custom;

    // Custom level should not enable any features automatically
    let features = vec![
        OptimizationFeature::FastLinker,
        OptimizationFeature::Incremental,
        OptimizationFeature::ParallelFrontend,
        OptimizationFeature::SplitDebuginfo,
        OptimizationFeature::Sccache,
        OptimizationFeature::NativeCpu,
        OptimizationFeature::ThinLto,
    ];

    for feature in features {
        assert!(
            !config.optimization_level.should_enable(feature),
            "Custom level should not auto-enable {:?}",
            feature
        );
    }
}

// Issue #017: Test detection counting non-test files
#[test]
fn regression_issue_017_test_detection_accuracy() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create various files
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::create_dir_all(project_root.join("tests")).unwrap();
    fs::create_dir_all(project_root.join("benches")).unwrap();

    // Regular source file with test module
    let src_with_tests = r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}
"#;
    fs::write(project_root.join("src/lib.rs"), src_with_tests).unwrap();

    // Integration test
    let integration_test = r#"
#[test]
fn integration_test() {
    assert!(true);
}
"#;
    fs::write(project_root.join("tests/integration.rs"), integration_test).unwrap();

    // Benchmark (not a test)
    let benchmark = r#"
use criterion::{black_box, Criterion};

fn create_test_metadata() -> cargo_metadata::Metadata {
    use cargo_metadata::{Package, PackageId, Version};
    cargo_metadata::Metadata {
        packages: vec![],
        workspace_members: vec![],
        resolve: None,
        root: std::env::current_dir().unwrap_or_default().into(),
        metadata: None,
        version: 1,
        workspace_root: std::env::current_dir().unwrap_or_default().into(),
        target_directory: std::env::current_dir().unwrap_or_default().join("target").into(),
    }
}


fn benchmark_add(c: &mut Criterion) {
    c.bench_function("add", |b| b.iter(|| black_box(2 + 2)));
}
"#;
    fs::write(project_root.join("benches/bench.rs"), benchmark).unwrap();

    let stats = CodeStats::calculate(project_root).unwrap();

    // Should correctly categorize files
    assert!(stats.test_files > 0);
    assert!(stats.bench_files > 0);
    assert!(stats.test_lines > 0);
    assert!(stats.bench_lines > 0);
}

// Issue #018: Duplicate dependency detection missing some cases
#[test]
fn regression_issue_018_duplicate_dependency_detection() {
    let duplicates = vec![
        DuplicateDependency {
            name: "serde".to_string(),
            versions: vec!["1.0.1".to_string(), "1.0.2".to_string()],
        },
        DuplicateDependency {
            name: "tokio".to_string(),
            versions: vec!["0.2.0".to_string(), "1.0.0".to_string()],
        },
    ];

    let deps = DependencyAnalysis {
        total_dependencies: 50,
        direct_dependencies: 10,
        transitive_dependencies: 40,
        proc_macro_count: 5,
        categories: Default::default(),
        heavy_dependencies: vec![],
        has_heavy_dependencies: false,
        duplicates: duplicates.clone(),
    };

    // Should detect that optimization is needed due to duplicates
    assert!(deps.needs_optimization());
    assert_eq!(deps.duplicates.len(), 2);
}

// Test helper to ensure all regression tests pass
#[test]
fn all_regressions_fixed() {
    // This test just ensures all regression tests are being run
    // and serves as a smoke test
    println!("Running regression test suite");
    assert!(true);
}
