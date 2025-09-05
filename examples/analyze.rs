//! Example: Analyze a Rust project

use cargo_optimize::analyzer::ProjectAnalysis;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get project path from command line or use current directory
    let project_path = env::args().nth(1).unwrap_or_else(|| ".".to_string());

    println!("Analyzing project at: {}\n", project_path);

    // Analyze the project
    let analysis = ProjectAnalysis::analyze(&project_path)?;

    // Display project information
    println!("📦 Project Information");
    println!("=====================");
    println!("Name: {}", analysis.metadata.name);
    println!("Version: {}", analysis.metadata.version);
    println!(
        "Type: {}",
        if analysis.is_workspace() {
            format!("Workspace ({} members)", analysis.crate_count())
        } else {
            "Single crate".to_string()
        }
    );

    // Display code statistics
    println!("\n📊 Code Statistics");
    println!("==================");
    println!("Total lines: {}", analysis.code_stats.total_lines);
    println!(
        "Rust code: {} lines in {} files",
        analysis.code_stats.rust_lines, analysis.code_stats.rust_files
    );
    println!(
        "Test code: {} lines in {} files",
        analysis.code_stats.test_lines, analysis.code_stats.test_files
    );

    if analysis.code_stats.bench_files > 0 {
        println!(
            "Benchmark code: {} lines in {} files",
            analysis.code_stats.bench_lines, analysis.code_stats.bench_files
        );
    }

    if analysis.code_stats.example_files > 0 {
        println!(
            "Example code: {} lines in {} files",
            analysis.code_stats.example_lines, analysis.code_stats.example_files
        );
    }

    // Display dependency information
    println!("\n📚 Dependencies");
    println!("===============");
    println!("Total: {}", analysis.dependencies.total_dependencies);
    println!("├─ Direct: {}", analysis.dependencies.direct_dependencies);
    println!(
        "└─ Transitive: {}",
        analysis.dependencies.transitive_dependencies
    );

    if analysis.dependencies.proc_macro_count > 0 {
        println!("Proc macros: {}", analysis.dependencies.proc_macro_count);
    }

    if !analysis.dependencies.heavy_dependencies.is_empty() {
        println!("\n⚠️  Heavy dependencies detected:");
        for dep in &analysis.dependencies.heavy_dependencies
            [..5.min(analysis.dependencies.heavy_dependencies.len())]
        {
            println!("  - {}", dep);
        }
        if analysis.dependencies.heavy_dependencies.len() > 5 {
            println!(
                "  ... and {} more",
                analysis.dependencies.heavy_dependencies.len() - 5
            );
        }
    }

    if !analysis.dependencies.duplicates.is_empty() {
        println!("\n⚠️  Duplicate dependencies:");
        for dup in
            &analysis.dependencies.duplicates[..3.min(analysis.dependencies.duplicates.len())]
        {
            println!("  - {}: {} versions", dup.name, dup.versions.len());
        }
    }

    // Display build complexity
    println!("\n🔧 Build Complexity");
    println!("===================");
    let complexity_bar = "█".repeat((analysis.complexity.score / 5) as usize);
    let empty_bar = "░".repeat(20 - (analysis.complexity.score / 5) as usize);
    println!(
        "Score: {}/100 [{}{}]",
        analysis.complexity.score, complexity_bar, empty_bar
    );
    println!(
        "Estimated build time: ~{}s",
        analysis.complexity.estimated_build_time
    );

    if analysis.complexity.is_large_project {
        println!("Classification: Large project");
    } else if analysis.complexity.is_complex {
        println!("Classification: Complex build");
    } else {
        println!("Classification: Standard project");
    }

    // Display recommendations
    if !analysis.recommendations.is_empty() {
        println!("\n💡 Optimization Recommendations");
        println!("================================");
        for (i, rec) in analysis.recommendations.iter().enumerate() {
            println!("{}. {}", i + 1, rec.description());
        }
    } else {
        println!("\n✅ No specific optimizations recommended - project is well-structured!");
    }

    Ok(())
}
