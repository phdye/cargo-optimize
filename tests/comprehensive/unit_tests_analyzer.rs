//! Unit tests for the analyzer module

use cargo_optimize::analyzer::{
    BuildComplexity, CodeStats, ComplexityFactor, DependencyAnalysis, DuplicateDependency,
    ProjectAnalysis, ProjectMetadata, Recommendation,
};
use pretty_assertions::assert_eq;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;


#[test]
fn test_analyzer_project_stats() {
    // Create a temporary project structure
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create a basic Cargo.toml
    let cargo_toml = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
"#;
    fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();

    // Create src directory with some code
    let src_dir = project_root.join("src");
    fs::create_dir(&src_dir).unwrap();

    let lib_rs = r#"
// Test library code
pub fn hello() {
    println!("Hello, world!");
}

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
    fs::write(src_dir.join("lib.rs"), lib_rs).unwrap();

    // Calculate code stats
    let stats = CodeStats::calculate(project_root).unwrap();

    assert!(stats.rust_files > 0);
    assert!(stats.rust_lines > 0);
    assert!(stats.total_lines > 0);
}

#[test]
fn test_code_stats_categorization() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create various directories
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::create_dir_all(project_root.join("tests")).unwrap();
    fs::create_dir_all(project_root.join("benches")).unwrap();
    fs::create_dir_all(project_root.join("examples")).unwrap();

    // Create files in each category
    fs::write(project_root.join("src/lib.rs"), "fn main() {}\n").unwrap();
    fs::write(
        project_root.join("tests/test.rs"),
        "#[test]\nfn test() {}\n",
    )
    .unwrap();
    fs::write(project_root.join("benches/bench.rs"), "fn bench() {}\n").unwrap();
    fs::write(
        project_root.join("examples/example.rs"),
        "fn example() {}\n",
    )
    .unwrap();

    let stats = CodeStats::calculate(project_root).unwrap();

    assert_eq!(stats.rust_files, 4);
    assert!(stats.test_files >= 1);
    assert!(stats.bench_files >= 1);
    assert!(stats.example_files >= 1);
}

#[test]
fn test_code_stats_large_project_detection() {
    let mut stats = CodeStats::default();

    // Small project
    stats.rust_lines = 1000;
    assert!(!stats.is_large());

    // Large project
    stats.rust_lines = 15000;
    assert!(stats.is_large());
}

#[test]
fn test_code_stats_test_heavy_detection() {
    let mut stats = CodeStats::default();

    // Not test-heavy
    stats.rust_lines = 1000;
    stats.test_lines = 200;
    assert!(!stats.is_test_heavy());

    // Test-heavy
    stats.test_lines = 600;
    assert!(stats.is_test_heavy());
}

#[test]
fn test_build_complexity_scoring() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create a minimal Cargo.toml
    let cargo_toml = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::write(project_root.join("src/lib.rs"), "").unwrap();

    // Create mock data for testing
    let metadata = ProjectMetadata::load(project_root).unwrap();

    let mut code_stats = CodeStats::default();
    code_stats.rust_lines = 5000; // Medium codebase

    let mut dependencies = DependencyAnalysis {
        total_dependencies: 50,
        direct_dependencies: 10,
        transitive_dependencies: 40,
        proc_macro_count: 5,
        categories: Default::default(),
        heavy_dependencies: vec![],
        has_heavy_dependencies: false,
        duplicates: vec![],
    };

    let complexity = BuildComplexity::calculate(&metadata, &code_stats, &dependencies);

    // Medium codebase should have moderate score
    assert!(complexity.score > 0);
    assert!(complexity.score < 50);

    // Test with large codebase
    code_stats.rust_lines = 60000;
    let complexity_large = BuildComplexity::calculate(&metadata, &code_stats, &dependencies);
    assert!(complexity_large.score > complexity.score);
    assert!(complexity_large.is_large_project);

    // Test with many dependencies
    dependencies.total_dependencies = 250;
    let complexity_deps = BuildComplexity::calculate(&metadata, &code_stats, &dependencies);
    assert!(complexity_deps.score > complexity_large.score);
    assert!(complexity_deps.is_complex);
}

#[test]
fn test_complexity_factors() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create minimal project
    let cargo_toml = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::write(project_root.join("src/lib.rs"), "").unwrap();

    let metadata = ProjectMetadata::load(project_root).unwrap();

    // Test various complexity factors
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

    // Very large codebase
    code_stats.rust_lines = 60000;
    let complexity = BuildComplexity::calculate(&metadata, &code_stats, &dependencies);
    assert!(complexity
        .factors
        .contains(&ComplexityFactor::VeryLargeCodebase));

    // Many dependencies
    code_stats.rust_lines = 1000;
    dependencies.total_dependencies = 250;
    let complexity = BuildComplexity::calculate(&metadata, &code_stats, &dependencies);
    assert!(complexity
        .factors
        .contains(&ComplexityFactor::ManyDependencies));

    // Heavy dependencies
    dependencies.has_heavy_dependencies = true;
    let complexity = BuildComplexity::calculate(&metadata, &code_stats, &dependencies);
    assert!(complexity
        .factors
        .contains(&ComplexityFactor::HeavyDependencies));

    // Many proc macros
    dependencies.proc_macro_count = 25;
    let complexity = BuildComplexity::calculate(&metadata, &code_stats, &dependencies);
    assert!(complexity
        .factors
        .contains(&ComplexityFactor::ManyProcMacros));
}

#[test]
fn test_recommendation_generation() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create project
    let cargo_toml = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = "1.0"
"#;
    fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::write(
        project_root.join("src/lib.rs"),
        "fn main() {}\n".repeat(15000).as_str(),
    )
    .unwrap();

    // Analyze project
    let analysis = ProjectAnalysis::analyze(project_root).unwrap();

    // Large project should get workspace recommendation
    if analysis.code_stats.rust_lines > 10000 {
        assert!(analysis
            .recommendations
            .contains(&Recommendation::SplitWorkspace));
        assert!(analysis
            .recommendations
            .contains(&Recommendation::EnableSccache));
    }
}

#[test]
fn test_recommendation_descriptions() {
    let recommendations = vec![
        Recommendation::SplitWorkspace,
        Recommendation::EnableSccache,
        Recommendation::MinimizeFeatures,
        Recommendation::UseWorkspaceDependencies,
        Recommendation::ConsiderAlternatives,
        Recommendation::OptimizeTests,
        Recommendation::UseNextest,
        Recommendation::CacheProcMacros,
    ];

    for rec in recommendations {
        assert!(!rec.description().is_empty());
    }
}

#[test]
fn test_dependency_analysis_categorization() {
    // This is a mock test since we can't easily create a real metadata
    let mut deps = DependencyAnalysis {
        total_dependencies: 100,
        direct_dependencies: 20,
        transitive_dependencies: 80,
        proc_macro_count: 5,
        categories: Default::default(),
        heavy_dependencies: vec!["tokio".to_string(), "actix-web".to_string()],
        has_heavy_dependencies: true,
        duplicates: vec![DuplicateDependency {
            name: "serde".to_string(),
            versions: vec!["1.0.1".to_string(), "1.0.2".to_string()],
        }],
    };

    assert!(deps.needs_optimization());

    // Test with minimal dependencies
    deps.total_dependencies = 10;
    deps.has_heavy_dependencies = false;
    deps.duplicates.clear();
    assert!(!deps.needs_optimization());
}

#[test]
fn test_workspace_detection() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create a workspace Cargo.toml
    let cargo_toml = r#"
[workspace]
members = ["crate1", "crate2"]

[workspace.package]
version = "0.1.0"
edition = "2021"
"#;
    fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();

    // Create member crates
    for i in 1..=2 {
        let crate_dir = project_root.join(format!("crate{}", i));
        fs::create_dir_all(&crate_dir.join("src")).unwrap();

        let member_toml = format!(
            r#"
[package]
name = "crate{}"
version = "0.1.0"
edition = "2021"
"#,
            i
        );
        fs::write(crate_dir.join("Cargo.toml"), member_toml).unwrap();
        fs::write(crate_dir.join("src/lib.rs"), "").unwrap();
    }

    let analysis = ProjectAnalysis::analyze(project_root).unwrap();
    assert!(analysis.is_workspace());
    assert_eq!(analysis.crate_count(), 2);
}

// Boundary value tests
#[test]
fn test_analyzer_empty_project() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create minimal Cargo.toml
    let cargo_toml = r#"
[package]
name = "empty"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::write(project_root.join("src/lib.rs"), "").unwrap();

    let analysis = ProjectAnalysis::analyze(project_root).unwrap();

    assert_eq!(analysis.code_stats.rust_lines, 0);
    assert_eq!(analysis.dependencies.total_dependencies, 0);
    assert!(!analysis.complexity.is_complex);
    assert!(!analysis.complexity.is_large_project);
}

#[test]
fn test_analyzer_massive_project() {
    let mut code_stats = CodeStats::default();
    code_stats.rust_lines = 1_000_000; // 1 million lines

    let dependencies = DependencyAnalysis {
        total_dependencies: 500,
        direct_dependencies: 100,
        transitive_dependencies: 400,
        proc_macro_count: 50,
        categories: Default::default(),
        heavy_dependencies: vec!["tokio".to_string(); 10],
        has_heavy_dependencies: true,
        duplicates: vec![],
    };

    // This should handle extreme values without panic
    assert!(code_stats.is_large());
    assert!(dependencies.needs_optimization());
}

// Property-based test helpers
#[test]
fn test_complexity_score_properties() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create minimal project
    let cargo_toml = r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::write(project_root.join("src/lib.rs"), "").unwrap();

    let metadata = ProjectMetadata::load(project_root).unwrap();

    // Property: complexity score should increase with size
    let mut code_stats = CodeStats::default();
    let dependencies = DependencyAnalysis {
        total_dependencies: 50,
        direct_dependencies: 10,
        transitive_dependencies: 40,
        proc_macro_count: 5,
        categories: Default::default(),
        heavy_dependencies: vec![],
        has_heavy_dependencies: false,
        duplicates: vec![],
    };

    let sizes = vec![1000, 5000, 10000, 50000, 100000];
    let mut prev_score = 0;

    for size in sizes {
        code_stats.rust_lines = size;
        let complexity = BuildComplexity::calculate(&metadata, &code_stats, &dependencies);

        // Score should generally increase with size
        assert!(complexity.score >= prev_score);
        prev_score = complexity.score;

        // Score should be bounded
        assert!(complexity.score <= 100);
    }
}

#[test]
fn test_test_ratio_calculation() {
    let mut code_stats = CodeStats::default();

    // No code
    code_stats.rust_lines = 0;
    code_stats.test_lines = 0;

    // Create a test project to get real metadata
    let test_project_dir = TempDir::new().unwrap();
    let test_cargo_toml = r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(test_project_dir.path().join("Cargo.toml"), test_cargo_toml).unwrap();
    fs::create_dir_all(test_project_dir.path().join("src")).unwrap();
    fs::write(test_project_dir.path().join("src/lib.rs"), "").unwrap();
    
    let real_metadata = ProjectMetadata::load(test_project_dir.path()).unwrap();

    // Create mock data
    let metadata = ProjectMetadata {
        name: "test".to_string(),
        version: "0.1.0".to_string(),
        root_path: PathBuf::from("."),
        is_workspace: false,
        workspace_members: vec![],
        cargo_metadata: real_metadata.cargo_metadata,
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
    assert_eq!(complexity.test_ratio, 0.0);

    // 50% tests
    code_stats.rust_lines = 1000;
    code_stats.test_lines = 500;
    let complexity = BuildComplexity::calculate(&metadata, &code_stats, &dependencies);
    assert!((complexity.test_ratio - 0.5).abs() < 0.01);

    // All tests
    code_stats.test_lines = 1000;
    let complexity = BuildComplexity::calculate(&metadata, &code_stats, &dependencies);
    assert!((complexity.test_ratio - 1.0).abs() < 0.01);
}

// Error handling tests
#[test]
fn test_analyzer_invalid_project_path() {
    let result = ProjectAnalysis::analyze("/this/path/does/not/exist");
    assert!(result.is_err());
}

#[test]
fn test_metadata_no_cargo_toml() {
    let temp_dir = TempDir::new().unwrap();
    let result = ProjectMetadata::load(temp_dir.path());
    assert!(result.is_err());
}

// Stress tests
#[test]
fn test_analyzer_deeply_nested_project() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create Cargo.toml
    let cargo_toml = r#"
[package]
name = "nested"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();

    // Create deeply nested structure
    let mut current = project_root.join("src");
    for i in 0..10 {
        fs::create_dir_all(&current).unwrap();
        fs::write(
            current.join(format!("mod{}.rs", i)),
            &format!("// Module {}\n", i),
        )
        .unwrap();
        current = current.join(format!("submod{}", i));
    }

    // Should handle deep nesting without stack overflow
    let stats = CodeStats::calculate(project_root).unwrap();
    assert!(stats.rust_files >= 10);
}

#[test]
fn test_analyzer_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let temp_dir = Arc::new(TempDir::new().unwrap());
    let project_root = temp_dir.path();

    // Create project
    let cargo_toml = r#"
[package]
name = "concurrent"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::write(project_root.join("src/lib.rs"), "fn main() {}").unwrap();

    let mut handles = Vec::new();

    // Spawn multiple threads analyzing the same project
    for _ in 0..5 {
        let temp_dir_clone = Arc::clone(&temp_dir);
        let handle = thread::spawn(move || {
            let analysis = ProjectAnalysis::analyze(temp_dir_clone.path()).unwrap();
            assert!(analysis.metadata.name == "concurrent");
        });
        handles.push(handle);
    }

    // All threads should complete successfully
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}
