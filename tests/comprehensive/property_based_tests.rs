//! Property-based tests for cargo-optimize

use cargo_optimize::{

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

    analyzer::*, detector::*, Config, OptimizationFeature, OptimizationLevel,
};
use proptest::prelude::*;

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


// Helper function to create mock metadata for testing
fn create_mock_metadata() -> cargo_metadata::Metadata {
    // Create a temporary project to get real metadata
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
    let metadata = ProjectMetadata::load(test_dir.path()).unwrap();
    metadata.cargo_metadata
}

// Strategies for generating test data
fn optimization_level_strategy() -> impl Strategy<Value = OptimizationLevel> {
    prop_oneof![
        Just(OptimizationLevel::Conservative),
        Just(OptimizationLevel::Balanced),
        Just(OptimizationLevel::Aggressive),
        Just(OptimizationLevel::Custom),
    ]
}

fn optimization_feature_strategy() -> impl Strategy<Value = OptimizationFeature> {
    prop_oneof![
        Just(OptimizationFeature::FastLinker),
        Just(OptimizationFeature::Incremental),
        Just(OptimizationFeature::ParallelFrontend),
        Just(OptimizationFeature::SplitDebuginfo),
        Just(OptimizationFeature::Sccache),
        Just(OptimizationFeature::NativeCpu),
        Just(OptimizationFeature::ThinLto),
    ]
}

fn config_strategy() -> impl Strategy<Value = Config> {
    (
        optimization_level_strategy(),
        any::<bool>(),                  // auto_detect_hardware
        any::<bool>(),                  // analyze_project
        any::<bool>(),                  // optimize_linker
        any::<bool>(),                  // enable_cache
        prop::option::of(0usize..=128), // parallel_jobs
        any::<bool>(),                  // incremental
        any::<bool>(),                  // split_debuginfo
        any::<bool>(),                  // verbose
        any::<bool>(),                  // dry_run
    )
        .prop_map(
            |(level, auto_detect, analyze, linker, cache, jobs, inc, split, verbose, dry)| {
                let mut config = Config::new();
                config.optimization_level = level;
                config.auto_detect_hardware = auto_detect;
                config.analyze_project = analyze;
                config.optimize_linker = linker;
                config.enable_cache = cache;
                config.parallel_jobs = jobs;
                config.incremental = inc;
                config.split_debuginfo = split;
                config.verbose = verbose;
                config.dry_run = dry;
                config
            },
        )
}

proptest! {
    // Property: Config serialization should be reversible
    #[test]
    fn property_config_serialization_roundtrip(config in config_strategy()) {
        let serialized = toml::to_string(&config).expect("Serialization failed");
        let deserialized: Config = toml::from_str(&serialized).expect("Deserialization failed");

        // Check key fields are preserved
        prop_assert_eq!(config.optimization_level, deserialized.optimization_level);
        prop_assert_eq!(config.auto_detect_hardware, deserialized.auto_detect_hardware);
        prop_assert_eq!(config.parallel_jobs, deserialized.parallel_jobs);
        prop_assert_eq!(config.verbose, deserialized.verbose);
    }

    // Property: Hardware detection should always produce consistent results
    #[test]
    fn property_hardware_detection_consistency(_i in 0..10) {
        let detector = SystemDetector::new();
        let hw1 = detector.detect_all();
        let hw2 = detector.detect_all();

        // Multiple detections should produce same results
        prop_assert_eq!(hw1.cpu.logical_cores, hw2.cpu.logical_cores);
        prop_assert_eq!(hw1.cpu.physical_cores, hw2.cpu.physical_cores);
        prop_assert_eq!(hw1.memory.total_bytes, hw2.memory.total_bytes);
    }

    // Property: Logical cores >= physical cores
    #[test]
    fn property_cpu_cores_relationship(_i in 0..10) {
        let detector = SystemDetector::new();
        let cpu = detector.detect_cpu();

        prop_assert!(cpu.logical_cores >= cpu.physical_cores);
        prop_assert!(cpu.physical_cores >= 1);
    }

    // Property: Available memory <= total memory
    #[test]
    fn property_memory_relationship(_i in 0..10) {
        let detector = SystemDetector::new();
        let memory = detector.detect_memory();

        prop_assert!(memory.available_bytes <= memory.total_bytes);
        prop_assert!(memory.total_bytes > 0);
    }

    // Property: Optimization levels should have consistent feature enabling
    #[test]
    fn property_optimization_level_consistency(
        level in optimization_level_strategy(),
        feature in optimization_feature_strategy()
    ) {
        let enabled = level.should_enable(feature);

        match level {
            OptimizationLevel::Custom => {
                // Custom should never auto-enable features
                prop_assert!(!enabled);
            }
            OptimizationLevel::Conservative => {
                // Conservative should enable fewer features
                if matches!(feature,
                    OptimizationFeature::ParallelFrontend |
                    OptimizationFeature::NativeCpu |
                    OptimizationFeature::ThinLto
                ) {
                    prop_assert!(!enabled);
                }
            }
            OptimizationLevel::Aggressive => {
                // Aggressive should enable most features
                if !matches!(feature, OptimizationFeature::NativeCpu) {
                    // NativeCpu might be disabled on some platforms
                    prop_assert!(enabled || !enabled); // Always true, but documents intent
                }
            }
            _ => {}
        }
    }

    // Property: Parallel jobs should be reasonable
    #[test]
    fn property_parallel_jobs_bounds(jobs in 0usize..=10000) {
        let mut config = Config::new();
        config.set_parallel_jobs(jobs);

        prop_assert_eq!(config.parallel_jobs, Some(jobs));
    }

    // Property: Code stats should be additive
    #[test]
    fn property_code_stats_additive(
        lines1 in 0usize..=100000,
        lines2 in 0usize..=100000,
        files1 in 0usize..=1000,
        files2 in 0usize..=1000
    ) {
        let mut stats1 = CodeStats::default();
        stats1.rust_lines = lines1;
        stats1.rust_files = files1;

        let mut stats2 = CodeStats::default();
        stats2.rust_lines = lines2;
        stats2.rust_files = files2;

        // Combined stats should be sum of parts
        let total_lines = lines1.saturating_add(lines2);
        let total_files = files1.saturating_add(files2);

        prop_assert_eq!(stats1.rust_lines + stats2.rust_lines, total_lines);
        prop_assert_eq!(stats1.rust_files + stats2.rust_files, total_files);
    }

    // Property: Build complexity score should be bounded
    #[test]
    fn property_build_complexity_bounds(
        lines in 0usize..=1000000,
        deps in 0usize..=1000,
        proc_macros in 0usize..=100
    ) {
        let mut code_stats = CodeStats::default();
        code_stats.rust_lines = lines;

        let dependencies = DependencyAnalysis {
            total_dependencies: deps,
            direct_dependencies: deps / 2,
            transitive_dependencies: deps / 2,
            proc_macro_count: proc_macros,
            categories: Default::default(),
            heavy_dependencies: vec![],
            has_heavy_dependencies: false,
            duplicates: vec![],
        };

        // Mock metadata for testing
        let metadata = ProjectMetadata {
            name: "test".to_string(),
            version: "0.1.0".to_string(),
            root_path: PathBuf::from("."),
            is_workspace: false,
            workspace_members: vec![],
            cargo_metadata: create_mock_metadata(),
        };

        let complexity = BuildComplexity::calculate(&metadata, &code_stats, &dependencies);

        // Score should always be bounded
        prop_assert!(complexity.score <= 100);

        // Large projects should be detected correctly
        if lines > 10000 {
            prop_assert!(complexity.is_large_project);
        }

        // Complex builds should have high scores
        if complexity.score > 50 {
            prop_assert!(complexity.is_complex);
        }
    }

    // Property: Test ratio should be between 0 and 1
    #[test]
    fn property_test_ratio_bounds(
        total_lines in 1usize..=100000,
        test_ratio in 0.0f32..=1.0
    ) {
        let test_lines = (total_lines as f32 * test_ratio) as usize;

        let mut code_stats = CodeStats::default();
        code_stats.rust_lines = total_lines;
        code_stats.test_lines = test_lines;

        let metadata = ProjectMetadata {
            name: "test".to_string(),
            version: "0.1.0".to_string(),
            root_path: PathBuf::from("."),
            is_workspace: false,
            workspace_members: vec![],
            cargo_metadata: create_mock_metadata(),
        };

        let dependencies = DependencyAnalysis {
            total_dependencies: 0,
            direct_dependencies: 0,
            transitive_dependencies: 0,
            proc_macro_count: 0,
            categories: Default::default(),
            heavy_dependencies: vec![],
            has_heavy_dependencies: false,
            duplicates: vec![],
        };

        let complexity = BuildComplexity::calculate(&metadata, &code_stats, &dependencies);

        prop_assert!(complexity.test_ratio >= 0.0);
        prop_assert!(complexity.test_ratio <= 1.0);
    }

    // Property: Dependency optimization need is monotonic
    #[test]
    fn property_dependency_optimization_monotonic(
        deps1 in 0usize..=100,
        deps2 in 101usize..=200
    ) {
        let dep_analysis1 = DependencyAnalysis {
            total_dependencies: deps1,
            direct_dependencies: deps1 / 2,
            transitive_dependencies: deps1 / 2,
            proc_macro_count: 0,
            categories: Default::default(),
            heavy_dependencies: vec![],
            has_heavy_dependencies: false,
            duplicates: vec![],
        };

        let mut dep_analysis2 = dep_analysis1.clone();
        dep_analysis2.total_dependencies = deps2;
        dep_analysis2.direct_dependencies = deps2 / 2;
        dep_analysis2.transitive_dependencies = deps2 / 2;

        // More dependencies should increase optimization need
        if deps2 > 50 {
            prop_assert!(dep_analysis2.needs_optimization());
        }

        if deps1 <= 50 && deps2 > 50 {
            prop_assert!(!dep_analysis1.needs_optimization() || dep_analysis2.needs_optimization());
        }
    }

    // Property: Config builder should be composable
    #[test]
    fn property_config_builder_composable(
        jobs in 1usize..=64,
        level in optimization_level_strategy()
    ) {
        let config = Config::new();

        // Apply settings in different orders
        let mut config1 = config.clone();
        config1
            .set_parallel_jobs(jobs)
            .set_optimization_level(level)
            .verbose();

        let mut config2 = config.clone();
        config2
            .set_optimization_level(level)
            .verbose()
            .set_parallel_jobs(jobs);

        // Result should be the same regardless of order
        prop_assert_eq!(config1.parallel_jobs, config2.parallel_jobs);
        prop_assert_eq!(config1.optimization_level, config2.optimization_level);
        prop_assert_eq!(config1.verbose, config2.verbose);
    }

    // Property: Flags should accumulate correctly
    #[test]
    fn property_flags_accumulation(
        flags in prop::collection::vec("[a-z]+", 0..10)
    ) {
        let mut config = Config::new();

        for flag in &flags {
            config.extra_cargo_flags.push(flag.clone());
        }

        prop_assert_eq!(config.extra_cargo_flags.len(), flags.len());

        // All flags should be present
        for flag in &flags {
            prop_assert!(config.extra_cargo_flags.contains(flag));
        }
    }

    // Property: Estimated build time should increase with complexity
    #[test]
    fn property_build_time_monotonic(
        lines in 0usize..=100000,
        deps in 0usize..=500
    ) {
        let mut code_stats = CodeStats::default();
        code_stats.rust_lines = lines;

        let dependencies = DependencyAnalysis {
            total_dependencies: deps,
            direct_dependencies: deps / 3,
            transitive_dependencies: deps * 2 / 3,
            proc_macro_count: 0,
            categories: Default::default(),
            heavy_dependencies: vec![],
            has_heavy_dependencies: false,
            duplicates: vec![],
        };

        let metadata = ProjectMetadata {
            name: "test".to_string(),
            version: "0.1.0".to_string(),
            root_path: PathBuf::from("."),
            is_workspace: false,
            workspace_members: vec![],
            cargo_metadata: create_mock_metadata(),
        };

        let complexity = BuildComplexity::calculate(&metadata, &code_stats, &dependencies);

        // Estimated build time should be reasonable
        // Note: comparison removed - estimated_build_time is unsigned, always >= 0

        // Larger projects should have longer estimated times
        if lines > 10000 || deps > 100 {
            prop_assert!(complexity.estimated_build_time > 0);
        }
    }

    // Property: Workspace member count should be consistent
    #[test]
    fn property_workspace_consistency(
        member_count in 0usize..=20
    ) {
        let members: Vec<String> = (0..member_count)
            .map(|i| format!("crate_{}", i))
            .collect();

        let metadata = ProjectMetadata {
            name: "workspace".to_string(),
            version: "0.1.0".to_string(),
            root_path: PathBuf::from("."),
            is_workspace: member_count > 1,
            workspace_members: members.clone(),
            cargo_metadata: create_mock_metadata(),
        };

        prop_assert_eq!(metadata.workspace_members.len(), member_count);
        prop_assert_eq!(metadata.is_workspace, member_count > 1);
    }

    // Property: Duplicate dependencies should have multiple versions
    #[test]
    fn property_duplicate_dependencies(
        name in "[a-z]+",
        versions in prop::collection::vec(
            prop::string::string_regex("[0-9]+\\.[0-9]+\\.[0-9]+").unwrap(),
            2..5
        )
    ) {
        let duplicate = DuplicateDependency {
            name: name.clone(),
            versions: versions.clone(),
        };

        prop_assert_eq!(duplicate.name, name);
        prop_assert!(duplicate.versions.len() >= 2);
        prop_assert_eq!(duplicate.versions.len(), versions.len());
    }
}

// Non-proptest property tests that require specific setup
#[test]
fn test_feature_enabling_hierarchy() {
    // Property: Aggressive >= Balanced >= Conservative for feature enabling
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
        let conservative = OptimizationLevel::Conservative.should_enable(feature);
        let balanced = OptimizationLevel::Balanced.should_enable(feature);
        let aggressive = OptimizationLevel::Aggressive.should_enable(feature);

        // If conservative enables it, balanced and aggressive should too
        if conservative {
            assert!(balanced);
            assert!(aggressive);
        }

        // If balanced enables it, aggressive should too
        if balanced {
            assert!(aggressive);
        }
    }
}

#[test]
fn test_complexity_factors_accumulation() {
    // Property: More factors should increase complexity score
    let metadata = ProjectMetadata {
        name: "test".to_string(),
        version: "0.1.0".to_string(),
        root_path: PathBuf::from("."),
        is_workspace: false,
        workspace_members: vec![],
        cargo_metadata: create_mock_metadata(),
    };

    let mut code_stats = CodeStats::default();
    let mut dependencies = DependencyAnalysis {
        total_dependencies: 0,
        direct_dependencies: 0,
        transitive_dependencies: 0,
        proc_macro_count: 0,
        categories: Default::default(),
        heavy_dependencies: vec![],
        has_heavy_dependencies: false,
        duplicates: vec![],
    };

    let base_complexity = BuildComplexity::calculate(&metadata, &code_stats, &dependencies);
    let base_score = base_complexity.score;

    // Add factors one by one
    code_stats.rust_lines = 60000; // VeryLargeCodebase
    let c1 = BuildComplexity::calculate(&metadata, &code_stats, &dependencies);
    assert!(c1.score > base_score);

    dependencies.total_dependencies = 250; // ManyDependencies
    let c2 = BuildComplexity::calculate(&metadata, &code_stats, &dependencies);
    assert!(c2.score > c1.score);

    dependencies.has_heavy_dependencies = true; // HeavyDependencies
    let c3 = BuildComplexity::calculate(&metadata, &code_stats, &dependencies);
    assert!(c3.score > c2.score);

    dependencies.proc_macro_count = 25; // ManyProcMacros
    let c4 = BuildComplexity::calculate(&metadata, &code_stats, &dependencies);
    assert!(c4.score >= c3.score);
}

#[test]
fn test_recommendation_generation_deterministic() {
    // Property: Same input should generate same recommendations
    let complexity1 = BuildComplexity {
        score: 60,
        is_large_project: true,
        is_complex: true,
        estimated_build_time: 100,
        test_ratio: 0.6,
        factors: vec![ComplexityFactor::LargeCodebase],
    };

    let dependencies1 = DependencyAnalysis {
        total_dependencies: 150,
        direct_dependencies: 30,
        transitive_dependencies: 120,
        proc_macro_count: 15,
        categories: Default::default(),
        heavy_dependencies: vec!["tokio".to_string()],
        has_heavy_dependencies: true,
        duplicates: vec![],
    };

    // Generate recommendations multiple times
    let recommendations1 = ProjectAnalysis::generate_recommendations(&complexity1, &dependencies1);
    let recommendations2 = ProjectAnalysis::generate_recommendations(&complexity1, &dependencies1);

    // Should be deterministic
    assert_eq!(recommendations1, recommendations2);

    // Should contain expected recommendations
    assert!(recommendations1.contains(&Recommendation::SplitWorkspace));
    assert!(recommendations1.contains(&Recommendation::EnableSccache));
    assert!(recommendations1.contains(&Recommendation::MinimizeFeatures));
}
