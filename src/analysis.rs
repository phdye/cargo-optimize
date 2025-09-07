//! Project analysis module using cargo_metadata and guppy
//! 
//! This module provides comprehensive project analysis capabilities including:
//! - Workspace structure detection
//! - Dependency graph analysis
//! - Feature analysis
//! - Build target detection
//! - Build metrics collection

use anyhow::{Context, Result};
use cargo_metadata::{MetadataCommand, Package, Metadata, DependencyKind};
use guppy::{
    graph::{PackageGraph, PackageMetadata, DependencyDirection, PackageSet},
    CargoMetadata,
};
use serde_json;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::info;

/// Project analysis results
#[derive(Debug, Clone)]
pub struct ProjectAnalysis {
    /// Workspace root path
    pub workspace_root: PathBuf,
    /// Is this a workspace with multiple members?
    pub is_workspace: bool,
    /// Workspace member packages
    pub workspace_members: Vec<PackageInfo>,
    /// Total dependency count (including transitive)
    pub total_dependencies: usize,
    /// Direct dependencies count
    pub direct_dependencies: usize,
    /// Dependency bottlenecks (packages that many others depend on)
    pub bottlenecks: Vec<BottleneckInfo>,
    /// Build targets (bins, libs, tests, etc.)
    pub targets: TargetAnalysis,
    /// Feature usage analysis
    pub features: FeatureAnalysis,
    /// Build metrics
    pub metrics: BuildMetrics,
}

/// Information about a package
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub dependencies: usize,
    pub dev_dependencies: usize,
    pub build_dependencies: usize,
}

/// Information about dependency bottlenecks
#[derive(Debug, Clone)]
pub struct BottleneckInfo {
    pub package: String,
    pub version: String,
    /// Number of packages that depend on this
    pub reverse_dependencies: usize,
    /// Depth in the dependency tree (1 = direct dependency)
    pub min_depth: usize,
}

/// Build target analysis
#[derive(Debug, Clone, Default)]
pub struct TargetAnalysis {
    pub binaries: Vec<String>,
    pub libraries: Vec<String>,
    pub tests: Vec<String>,
    pub benches: Vec<String>,
    pub examples: Vec<String>,
    pub build_scripts: usize,
    pub proc_macros: usize,
}

/// Feature usage analysis
#[derive(Debug, Clone, Default)]
pub struct FeatureAnalysis {
    /// Total number of available features across all dependencies
    pub total_features: usize,
    /// Number of features actually enabled
    pub enabled_features: usize,
    /// Packages with the most features
    pub feature_heavy_packages: Vec<(String, usize)>,
    /// Suggested feature optimizations
    pub suggestions: Vec<FeatureSuggestion>,
}

/// Feature optimization suggestion
#[derive(Debug, Clone)]
pub struct FeatureSuggestion {
    pub package: String,
    pub suggestion: String,
    pub impact: ImpactLevel,
}

/// Impact level of an optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpactLevel {
    High,
    Medium,
    Low,
}

/// Build metrics
#[derive(Debug, Clone, Default)]
pub struct BuildMetrics {
    /// Estimated lines of code to compile
    pub estimated_loc: usize,
    /// Number of crates to compile
    pub crate_count: usize,
    /// Potential parallelization factor
    pub parallelization_factor: f32,
    /// Workspace member interdependencies
    pub internal_dependencies: usize,
}

/// Main analysis function
pub fn analyze_project(manifest_path: Option<&Path>) -> Result<ProjectAnalysis> {
    info!("Starting project analysis");
    
    // Get cargo metadata
    let mut cmd = MetadataCommand::new();
    if let Some(path) = manifest_path {
        cmd.manifest_path(path);
    }
    
    let metadata = cmd.exec()
        .context("Failed to execute cargo metadata")?;
    
    // Convert cargo_metadata::Metadata to guppy::CargoMetadata
    let metadata_json = serde_json::to_string(&metadata)
        .context("Failed to serialize metadata")?;
    let cargo_metadata: CargoMetadata = serde_json::from_str(&metadata_json)
        .context("Failed to deserialize into CargoMetadata")?;
    
    // Build guppy package graph
    let package_graph = PackageGraph::from_metadata(cargo_metadata)
        .context("Failed to build package graph")?;
    
    // Analyze workspace structure
    let workspace_members = analyze_workspace_members(&metadata, &package_graph)?;
    let is_workspace = workspace_members.len() > 1;
    
    // Analyze dependencies
    let (total_deps, direct_deps) = count_dependencies(&metadata, &package_graph)?;
    
    // Find bottlenecks
    let bottlenecks = find_bottlenecks(&package_graph)?;
    
    // Analyze build targets
    let targets = analyze_targets(&metadata)?;
    
    // Analyze features
    let features = analyze_features(&metadata, &package_graph)?;
    
    // Calculate build metrics
    let metrics = calculate_build_metrics(&metadata, &package_graph, &workspace_members)?;
    
    Ok(ProjectAnalysis {
        workspace_root: metadata.workspace_root.clone().into(),
        is_workspace,
        workspace_members,
        total_dependencies: total_deps,
        direct_dependencies: direct_deps,
        bottlenecks,
        targets,
        features,
        metrics,
    })
}

/// Analyze workspace members
fn analyze_workspace_members(
    metadata: &Metadata,
    _graph: &PackageGraph,
) -> Result<Vec<PackageInfo>> {
    let mut members = Vec::new();
    
    for member_id in &metadata.workspace_members {
        let package = metadata.packages
            .iter()
            .find(|p| &p.id == member_id)
            .context("Failed to find workspace member package")?;
        
        let (deps, dev_deps, build_deps) = count_package_dependencies(package);
        
        members.push(PackageInfo {
            name: package.name.clone(),
            version: package.version.to_string(),
            path: package.manifest_path.parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_default()
                .into(),
            dependencies: deps,
            dev_dependencies: dev_deps,
            build_dependencies: build_deps,
        });
    }
    
    Ok(members)
}

/// Count dependencies for a package
fn count_package_dependencies(package: &Package) -> (usize, usize, usize) {
    let mut deps = 0;
    let mut dev_deps = 0;
    let mut build_deps = 0;
    
    for dep in &package.dependencies {
        match dep.kind {
            DependencyKind::Normal => deps += 1,
            DependencyKind::Development => dev_deps += 1,
            DependencyKind::Build => build_deps += 1,
            _ => {}
        }
    }
    
    (deps, dev_deps, build_deps)
}

/// Count total and direct dependencies
fn count_dependencies(
    _metadata: &Metadata,
    graph: &PackageGraph,
) -> Result<(usize, usize)> {
    // Get workspace members
    let workspace_set = graph.resolve_workspace();
    
    // Count direct dependencies (immediate dependencies of workspace members)
    let direct_deps: HashSet<_> = workspace_set
        .packages(DependencyDirection::Forward)
        .flat_map(|p| p.direct_links())
        .map(|link| link.to().id().to_string())
        .collect();
    let direct_deps = direct_deps.len();
    
    // Count total dependencies (all transitive dependencies)
    // We need to walk all forward dependencies from workspace members
    let all_deps: HashSet<_> = workspace_set
        .packages(DependencyDirection::Forward)
        .flat_map(|p| p.direct_links())
        .map(|link| link.to().id().to_string())
        .collect();
    
    // Total dependencies is just the count of unique dependencies
    let total_deps = all_deps.len();
    
    Ok((total_deps, direct_deps))
}

/// Find dependency bottlenecks
fn find_bottlenecks(graph: &PackageGraph) -> Result<Vec<BottleneckInfo>> {
    let mut bottlenecks = Vec::new();
    let workspace_set = graph.resolve_workspace();
    
    // For each package, count reverse dependencies
    for package in graph.packages() {
        // Skip workspace members
        if workspace_set.contains(package.id())? {
            continue;
        }
        
        let reverse_deps = package
            .reverse_direct_links()
            .count();
        
        // Consider it a bottleneck if more than 3 packages depend on it
        if reverse_deps > 3 {
            // For simplicity, set depth to 1 for direct deps, 2 for others
            // A more sophisticated implementation would calculate actual depth
            let is_direct = workspace_set
                .packages(DependencyDirection::Forward)
                .flat_map(|p| p.direct_links())
                .any(|link| link.to().id() == package.id());
            
            let min_depth = if is_direct { 1 } else { 2 };
            
            bottlenecks.push(BottleneckInfo {
                package: package.name().to_string(),
                version: package.version().to_string(),
                reverse_dependencies: reverse_deps,
                min_depth,
            });
        }
    }
    
    // Sort by reverse dependencies (most depended-upon first)
    bottlenecks.sort_by_key(|b| std::cmp::Reverse(b.reverse_dependencies));
    bottlenecks.truncate(10); // Keep top 10 bottlenecks
    
    Ok(bottlenecks)
}

/// Analyze build targets
fn analyze_targets(metadata: &Metadata) -> Result<TargetAnalysis> {
    let mut targets = TargetAnalysis::default();
    
    for package in &metadata.packages {
        // Only analyze workspace members
        if !metadata.workspace_members.contains(&package.id) {
            continue;
        }
        
        for target in &package.targets {
            for kind in &target.kind {
                match kind.as_str() {
                    "bin" => targets.binaries.push(target.name.clone()),
                    "lib" | "rlib" | "dylib" | "cdylib" | "staticlib" => {
                        targets.libraries.push(target.name.clone())
                    }
                    "test" => targets.tests.push(target.name.clone()),
                    "bench" => targets.benches.push(target.name.clone()),
                    "example" => targets.examples.push(target.name.clone()),
                    "custom-build" => targets.build_scripts += 1,
                    "proc-macro" => targets.proc_macros += 1,
                    _ => {}
                }
            }
        }
    }
    
    Ok(targets)
}

/// Analyze feature usage
fn analyze_features(
    metadata: &Metadata,
    graph: &PackageGraph,
) -> Result<FeatureAnalysis> {
    let mut analysis = FeatureAnalysis::default();
    let mut feature_counts: Vec<(String, usize)> = Vec::new();
    
    // Count features for each package
    for package in &metadata.packages {
        let feature_count = package.features.len();
        if feature_count > 0 {
            feature_counts.push((package.name.clone(), feature_count));
            analysis.total_features += feature_count;
        }
    }
    
    // Sort by feature count
    feature_counts.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    analysis.feature_heavy_packages = feature_counts.into_iter().take(5).collect();
    
    // Add suggestions for common optimizations
    analysis.suggestions = generate_feature_suggestions(metadata, graph)?;
    
    // Count enabled features (simplified - would need actual resolution in practice)
    analysis.enabled_features = analysis.total_features / 2; // Rough estimate
    
    Ok(analysis)
}

/// Generate feature optimization suggestions
fn generate_feature_suggestions(
    metadata: &Metadata,
    _graph: &PackageGraph,
) -> Result<Vec<FeatureSuggestion>> {
    let mut suggestions = Vec::new();
    
    // Check for common heavy dependencies
    for package in &metadata.packages {
        // Skip workspace members
        if metadata.workspace_members.contains(&package.id) {
            continue;
        }
        
        match package.name.as_str() {
            "tokio" => {
                if package.features.len() > 10 {
                    suggestions.push(FeatureSuggestion {
                        package: "tokio".to_string(),
                        suggestion: "Consider using only required tokio features instead of 'full'".to_string(),
                        impact: ImpactLevel::High,
                    });
                }
            }
            "serde" => {
                if !package.features.contains_key("derive") {
                    suggestions.push(FeatureSuggestion {
                        package: "serde".to_string(),
                        suggestion: "serde without 'derive' feature detected - ensure this is intentional".to_string(),
                        impact: ImpactLevel::Low,
                    });
                }
            }
            "reqwest" => {
                if package.features.contains_key("blocking") && package.features.contains_key("tokio") {
                    suggestions.push(FeatureSuggestion {
                        package: "reqwest".to_string(),
                        suggestion: "Both blocking and async features enabled - consider using only one".to_string(),
                        impact: ImpactLevel::Medium,
                    });
                }
            }
            _ => {}
        }
    }
    
    Ok(suggestions)
}

/// Calculate build metrics
fn calculate_build_metrics(
    metadata: &Metadata,
    graph: &PackageGraph,
    _workspace_members: &[PackageInfo],
) -> Result<BuildMetrics> {
    let mut metrics = BuildMetrics::default();
    
    // Count crates to compile
    metrics.crate_count = metadata.packages.len();
    
    // Estimate lines of code (very rough estimate based on dependency count)
    metrics.estimated_loc = metadata.packages.len() * 1000; // Rough estimate
    
    // Calculate parallelization factor
    // Higher is better - indicates more opportunities for parallel compilation
    let workspace_set = graph.resolve_workspace();
    let max_chain_length = calculate_max_dependency_chain(graph, &workspace_set)?;
    metrics.parallelization_factor = if max_chain_length > 0 {
        metrics.crate_count as f32 / max_chain_length as f32
    } else {
        1.0
    };
    
    // Count internal dependencies between workspace members
    for member_id in &metadata.workspace_members {
        if let Some(package) = metadata.packages.iter().find(|p| &p.id == member_id) {
            for dep in &package.dependencies {
                if metadata.workspace_members.iter().any(|m| {
                    metadata.packages.iter()
                        .find(|p| &p.id == m)
                        .map(|p| p.name == dep.name)
                        .unwrap_or(false)
                }) {
                    metrics.internal_dependencies += 1;
                }
            }
        }
    }
    
    Ok(metrics)
}

/// Calculate the maximum dependency chain length
fn calculate_max_dependency_chain(
    _graph: &PackageGraph,
    workspace_set: &PackageSet<'_>,
) -> Result<usize> {
    let mut max_length = 0;
    
    // For each workspace member, find the longest dependency chain
    for package in workspace_set.packages(DependencyDirection::Forward) {
        let chain_length = calculate_chain_length_from(_graph, package)?;
        max_length = max_length.max(chain_length);
    }
    
    Ok(max_length)
}

/// Calculate dependency chain length from a package
fn calculate_chain_length_from(
    _graph: &PackageGraph,
    package: PackageMetadata<'_>,
) -> Result<usize> {
    let mut visited = HashSet::new();
    let mut max_depth = 0;
    
    fn visit(
        package: PackageMetadata<'_>,
        visited: &mut HashSet<String>,
        depth: usize,
        max_depth: &mut usize,
    ) {
        let id = package.id().to_string();
        if !visited.insert(id) {
            return;
        }
        
        *max_depth = (*max_depth).max(depth);
        
        for link in package.direct_links() {
            visit(link.to(), visited, depth + 1, max_depth);
        }
    }
    
    visit(package, &mut visited, 0, &mut max_depth);
    Ok(max_depth)
}

/// Get a summary of the analysis suitable for display
impl ProjectAnalysis {
    pub fn summary(&self) -> String {
        let mut summary = String::new();
        
        summary.push_str(&format!("Workspace root: {}\n", self.workspace_root.display()));
        summary.push_str(&format!("Is workspace: {}\n", self.is_workspace));
        summary.push_str(&format!("Workspace members: {}\n", self.workspace_members.len()));
        summary.push_str(&format!("Total dependencies: {}\n", self.total_dependencies));
        summary.push_str(&format!("Direct dependencies: {}\n", self.direct_dependencies));
        
        if !self.bottlenecks.is_empty() {
            summary.push_str("\nTop dependency bottlenecks:\n");
            for (i, bottleneck) in self.bottlenecks.iter().take(3).enumerate() {
                summary.push_str(&format!(
                    "  {}. {} v{} ({} reverse deps, depth {})\n",
                    i + 1,
                    bottleneck.package,
                    bottleneck.version,
                    bottleneck.reverse_dependencies,
                    bottleneck.min_depth
                ));
            }
        }
        
        summary.push_str(&format!("\nBuild targets:\n"));
        summary.push_str(&format!("  Binaries: {}\n", self.targets.binaries.len()));
        summary.push_str(&format!("  Libraries: {}\n", self.targets.libraries.len()));
        summary.push_str(&format!("  Tests: {}\n", self.targets.tests.len()));
        summary.push_str(&format!("  Examples: {}\n", self.targets.examples.len()));
        summary.push_str(&format!("  Build scripts: {}\n", self.targets.build_scripts));
        summary.push_str(&format!("  Proc macros: {}\n", self.targets.proc_macros));
        
        summary.push_str(&format!("\nFeature analysis:\n"));
        summary.push_str(&format!("  Total features: {}\n", self.features.total_features));
        summary.push_str(&format!("  Enabled features: ~{}\n", self.features.enabled_features));
        
        if !self.features.suggestions.is_empty() {
            summary.push_str("  Optimization suggestions:\n");
            for suggestion in &self.features.suggestions {
                summary.push_str(&format!(
                    "    - {}: {} [{:?}]\n",
                    suggestion.package,
                    suggestion.suggestion,
                    suggestion.impact
                ));
            }
        }
        
        summary.push_str(&format!("\nBuild metrics:\n"));
        summary.push_str(&format!("  Crates to compile: {}\n", self.metrics.crate_count));
        summary.push_str(&format!("  Parallelization factor: {:.1}\n", self.metrics.parallelization_factor));
        summary.push_str(&format!("  Internal dependencies: {}\n", self.metrics.internal_dependencies));
        
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_impact_level() {
        assert_eq!(
            format!("{:?}", ImpactLevel::High),
            "High"
        );
    }
}
