// Helper utilities for tests
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


/// Create a mock ProjectAnalysis for testing
pub fn create_mock_project_analysis() -> cargo_optimize::analyzer::ProjectAnalysis {
    cargo_optimize::analyzer::ProjectAnalysis {
        complexity: cargo_optimize::analyzer::BuildComplexity {
            loc: 0,
            dependencies: 0,
            test_count: 0,
            bench_count: 0,
            example_count: 0,
            bin_count: 1,
            workspace_members: 1,
            total_size_mb: 0.0,
            factors: vec![],
        },
        dependencies: cargo_optimize::analyzer::DependencyAnalysis {
            direct_deps: vec![],
            transitive_deps: vec![],
            build_deps: vec![],
            dev_deps: vec![],
            total_deps: 0,
            heavy_deps: vec![],
            duplicate_deps: vec![],
        },
        cargo_metadata: None,  // Use None instead of trying to create Metadata
    }
}
