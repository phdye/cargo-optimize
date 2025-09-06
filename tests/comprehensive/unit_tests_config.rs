//! Comprehensive unit tests for the config module

use cargo_optimize::config::{
    Config, LtoConfig, OptimizationFeature, OptimizationLevel, PanicStrategy,
    StripConfig,
};
use std::path::PathBuf;
use tempfile::TempDir;


#[test]
fn test_config_default_values() {
    let config = Config::default();

    pretty_assertions::assert_eq!(config.optimization_level, OptimizationLevel::Balanced);
    assert!(config.auto_detect_hardware);
    assert!(config.analyze_project);
    assert!(config.optimize_linker);
    assert!(config.enable_cache);
    pretty_assertions::assert_eq!(config.parallel_jobs, None);
    pretty_assertions::assert_eq!(config.custom_linker, None);
    assert!(config.incremental);
    assert!(config.split_debuginfo);
    pretty_assertions::assert_eq!(config.target_cpu, None);
    assert!(config.extra_cargo_flags.is_empty());
    assert!(config.extra_rustc_flags.is_empty());
    assert!(!config.verbose);
    assert!(!config.dry_run);
}

#[test]
fn test_config_builder_pattern() {
    let mut config = Config::new();
    let result = config
        .set_optimization_level(OptimizationLevel::Aggressive)
        .set_auto_detect(false)
        .set_parallel_jobs(8)
        .verbose()
        .dry_run();

    pretty_assertions::assert_eq!(result.optimization_level, OptimizationLevel::Aggressive);
    assert!(!result.auto_detect_hardware);
    pretty_assertions::assert_eq!(result.parallel_jobs, Some(8));
    assert!(result.verbose);
    assert!(result.dry_run);
}

#[test]
fn test_optimization_level_descriptions() {
    assert!(!OptimizationLevel::Conservative.description().is_empty());
    assert!(!OptimizationLevel::Balanced.description().is_empty());
    assert!(!OptimizationLevel::Aggressive.description().is_empty());
    assert!(!OptimizationLevel::Custom.description().is_empty());
}

#[test]
fn test_optimization_level_features_conservative() {
    let level = OptimizationLevel::Conservative;

    // Should enable safe features
    assert!(level.should_enable(OptimizationFeature::FastLinker));
    assert!(level.should_enable(OptimizationFeature::Incremental));
    assert!(level.should_enable(OptimizationFeature::Sccache));

    // Should not enable risky features
    assert!(!level.should_enable(OptimizationFeature::ParallelFrontend));
    assert!(!level.should_enable(OptimizationFeature::NativeCpu));
    assert!(!level.should_enable(OptimizationFeature::ThinLto));
}

#[test]
fn test_optimization_level_features_balanced() {
    let level = OptimizationLevel::Balanced;

    // Should enable most features
    assert!(level.should_enable(OptimizationFeature::FastLinker));
    assert!(level.should_enable(OptimizationFeature::Incremental));
    assert!(level.should_enable(OptimizationFeature::ParallelFrontend));
    assert!(level.should_enable(OptimizationFeature::SplitDebuginfo));
    assert!(level.should_enable(OptimizationFeature::Sccache));
    assert!(level.should_enable(OptimizationFeature::ThinLto));

    // Still avoid some risky features
    assert!(!level.should_enable(OptimizationFeature::NativeCpu));
}

#[test]
fn test_optimization_level_features_aggressive() {
    let level = OptimizationLevel::Aggressive;

    // Should enable all features
    assert!(level.should_enable(OptimizationFeature::FastLinker));
    assert!(level.should_enable(OptimizationFeature::Incremental));
    assert!(level.should_enable(OptimizationFeature::ParallelFrontend));
    assert!(level.should_enable(OptimizationFeature::SplitDebuginfo));
    assert!(level.should_enable(OptimizationFeature::Sccache));
    assert!(level.should_enable(OptimizationFeature::NativeCpu));
    assert!(level.should_enable(OptimizationFeature::ThinLto));
}

#[test]
fn test_optimization_level_features_custom() {
    let level = OptimizationLevel::Custom;

    // Custom should not make automatic decisions
    assert!(!level.should_enable(OptimizationFeature::FastLinker));
    assert!(!level.should_enable(OptimizationFeature::Incremental));
    assert!(!level.should_enable(OptimizationFeature::NativeCpu));
}

#[test]
fn test_config_serialization_roundtrip() {
    let mut original = Config::default();
    original
        .set_optimization_level(OptimizationLevel::Aggressive)
        .set_parallel_jobs(16)
        .verbose();

    // Serialize to TOML
    let toml_str = toml::to_string(&original).expect("Serialization failed");

    // Deserialize back
    let deserialized: Config = toml::from_str(&toml_str).expect("Deserialization failed");

    pretty_assertions::assert_eq!(original.optimization_level, deserialized.optimization_level);
    pretty_assertions::assert_eq!(original.parallel_jobs, deserialized.parallel_jobs);
    pretty_assertions::assert_eq!(original.verbose, deserialized.verbose);
}

#[test]
fn test_config_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");

    let mut config = Config::default();
    config
        .set_optimization_level(OptimizationLevel::Aggressive)
        .set_parallel_jobs(12);

    // Save to file
    config.save(&config_path).expect("Failed to save config");
    assert!(config_path.exists());

    // Load from file
    let loaded = Config::from_file(&config_path).expect("Failed to load config");
    pretty_assertions::assert_eq!(config.optimization_level, loaded.optimization_level);
    pretty_assertions::assert_eq!(config.parallel_jobs, loaded.parallel_jobs);
}

// Boundary Value Tests
#[test]
fn test_config_boundary_parallel_jobs() {
    let mut config = Config::new();

    // Test edge values
    config.set_parallel_jobs(0); // Should be valid
    pretty_assertions::assert_eq!(config.parallel_jobs, Some(0));

    config.set_parallel_jobs(1); // Minimum useful value
    pretty_assertions::assert_eq!(config.parallel_jobs, Some(1));

    // The implementation caps at 1000
    config.set_parallel_jobs(usize::MAX); // Maximum value - gets capped to 1000
    pretty_assertions::assert_eq!(config.parallel_jobs, Some(1000));

    // Test at the exact limit
    config.set_parallel_jobs(1000);
    pretty_assertions::assert_eq!(config.parallel_jobs, Some(1000));

    // Test above the limit
    config.set_parallel_jobs(2000);
    pretty_assertions::assert_eq!(config.parallel_jobs, Some(1000));
}

#[test]
fn test_config_with_empty_vectors() {
    let config = Config::default();
    assert!(config.extra_cargo_flags.is_empty());
    assert!(config.extra_rustc_flags.is_empty());
}

#[test]
fn test_config_with_large_vectors() {
    let mut config = Config::default();

    // Test with large number of flags
    for i in 0..1000 {
        config.extra_cargo_flags.push(format!("--flag-{}", i));
        config.extra_rustc_flags.push(format!("-C flag-{}", i));
    }

    pretty_assertions::assert_eq!(config.extra_cargo_flags.len(), 1000);
    pretty_assertions::assert_eq!(config.extra_rustc_flags.len(), 1000);
}

#[test]
fn test_config_custom_linker_paths() {
    let mut config = Config::default();

    // Test various path formats
    let paths = vec![
        "/usr/bin/ld",
        "C:\\Program Files\\LLVM\\bin\\lld.exe",
        "../relative/path/linker",
        "./local-linker",
        "just-a-name",
    ];

    for path in paths {
        config.custom_linker = Some(PathBuf::from(path));
        pretty_assertions::assert_eq!(
            config.custom_linker.as_ref().unwrap().to_str().unwrap(),
            path
        );
    }
}

// Stress Tests
#[test]
fn test_config_massive_serialization() {
    let mut config = Config::default();

    // Create a config with massive amounts of data
    for i in 0..10000 {
        config
            .extra_cargo_flags
            .push(format!("--very-long-flag-name-{}-with-lots-of-data", i));
        config.extra_rustc_flags.push(format!(
            "-C very-long-rustc-flag-{}-with-detailed-options=value",
            i
        ));
    }

    // This should not panic or fail
    let serialized = toml::to_string(&config).expect("Failed to serialize large config");
    assert!(!serialized.is_empty());

    // And we should be able to deserialize it back
    let _deserialized: Config =
        toml::from_str(&serialized).expect("Failed to deserialize large config");
}

#[test]
fn test_config_unicode_in_strings() {
    let mut config = Config::default();

    // Test with various unicode characters
    config.extra_cargo_flags.push("--flag=🦀".to_string());
    config.extra_cargo_flags.push("--测试=value".to_string());
    config.extra_cargo_flags.push("--emoji=🎯🚀⚡".to_string());
    config.target_cpu = Some("🔥cpu".to_string());

    // Should serialize and deserialize correctly
    let serialized = toml::to_string(&config).expect("Failed to serialize unicode config");
    let _deserialized: Config =
        toml::from_str(&serialized).expect("Failed to deserialize unicode config");
}

// Property-Based Test Helpers
#[cfg(test)]
mod property_tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_optimization_level_consistency() {
        // Property: Each optimization level should have consistent behavior
        let levels = vec![
            OptimizationLevel::Conservative,
            OptimizationLevel::Balanced,
            OptimizationLevel::Aggressive,
            OptimizationLevel::Custom,
        ];

        let features = vec![
            OptimizationFeature::FastLinker,
            OptimizationFeature::Incremental,
            OptimizationFeature::ParallelFrontend,
            OptimizationFeature::SplitDebuginfo,
            OptimizationFeature::Sccache,
            OptimizationFeature::NativeCpu,
            OptimizationFeature::ThinLto,
        ];

        for level in &levels {
            let mut enabled_features = HashSet::new();

            for feature in &features {
                if level.should_enable(*feature) {
                    enabled_features.insert(*feature);
                }
            }

            // Property: Conservative should enable fewer features than Balanced
            if *level == OptimizationLevel::Conservative {
                assert!(
                    enabled_features.len() <= 3,
                    "Conservative should enable few features"
                );
            }

            // Property: Aggressive should enable more features than Conservative
            if *level == OptimizationLevel::Aggressive {
                assert!(
                    enabled_features.len() >= 5,
                    "Aggressive should enable many features"
                );
            }

            // Property: Custom should enable no features automatically
            if *level == OptimizationLevel::Custom {
                assert!(
                    enabled_features.is_empty(),
                    "Custom should not enable features automatically"
                );
            }
        }
    }

    #[test]
    fn test_config_builder_immutability() {
        // Property: Builder pattern should not affect original until assignment
        let original = Config::default();
        let mut builder = Config::new();

        builder
            .set_optimization_level(OptimizationLevel::Aggressive)
            .set_parallel_jobs(999)
            .verbose();

        // Original should be unchanged
        pretty_assertions::assert_eq!(original.optimization_level, OptimizationLevel::Balanced);
        pretty_assertions::assert_eq!(original.parallel_jobs, None);
        assert!(!original.verbose);
    }
}

#[test]
fn test_lto_config_values() {
    use LtoConfig::*;

    let configs = vec![Off, Thin, Fat];

    for config in configs {
        // Each config should have a valid string representation
        let serialized = toml::to_string(&config).expect("Failed to serialize LTO config");
        assert!(!serialized.is_empty());

        // Should deserialize back to the same value
        let deserialized: LtoConfig =
            toml::from_str(&serialized).expect("Failed to deserialize LTO config");
        pretty_assertions::assert_eq!(config, deserialized);
    }
}

#[test]
fn test_panic_strategy_values() {
    use PanicStrategy::*;

    let strategies = vec![Unwind, Abort];

    for strategy in strategies {
        let serialized = toml::to_string(&strategy).expect("Failed to serialize panic strategy");
        assert!(!serialized.is_empty());

        let deserialized: PanicStrategy =
            toml::from_str(&serialized).expect("Failed to deserialize panic strategy");
        pretty_assertions::assert_eq!(strategy, deserialized);
    }
}

#[test]
fn test_strip_config_values() {
    use StripConfig::*;

    let configs = vec![None, Debuginfo, Symbols];

    for config in configs {
        let serialized = toml::to_string(&config).expect("Failed to serialize strip config");
        assert!(!serialized.is_empty());

        let deserialized: StripConfig =
            toml::from_str(&serialized).expect("Failed to deserialize strip config");
        pretty_assertions::assert_eq!(config, deserialized);
    }
}

// Error Handling Tests
#[test]
fn test_config_invalid_file_path() {
    // Test loading from non-existent file
    let result = Config::from_file("/this/path/does/not/exist/config.toml");
    assert!(result.is_err());
}

#[test]
fn test_config_invalid_toml_content() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.toml");

    // Write invalid TOML
    std::fs::write(&config_path, "this is not valid toml [[[").unwrap();

    let result = Config::from_file(&config_path);
    assert!(result.is_err());
}

#[test]
fn test_config_permission_denied() {
    // This test is platform-specific and might not work in all environments
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let readonly_dir = temp_dir.path().join("readonly");
        std::fs::create_dir(&readonly_dir).unwrap();

        // Make directory read-only
        let mut perms = std::fs::metadata(&readonly_dir).unwrap().permissions();
        perms.set_mode(0o444);
        std::fs::set_permissions(&readonly_dir, perms).unwrap();

        let config = Config::default();
        let result = config.save(readonly_dir.join("config.toml"));

        // Should fail due to permissions
        assert!(result.is_err());
    }
}
