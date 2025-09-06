//! Comprehensive Test Suite for cargo-optimize MVP
//! Phase 1: Foundation & Critical Path Testing
//! 
//! This is the main test harness that runs the complete Phase 1 test suite
//! as specified in the comprehensive testing plan.

use std::time::Instant;

// Import the comprehensive test module
mod comprehensive;

use comprehensive::run_phase1_comprehensive_testing;

fn main() {
    println!("🚀 cargo-optimize MVP Comprehensive Testing Suite");
    println!("=================================================");
    println!("Phase 1: Foundation & Critical Path Testing");
    println!("");
    
    let start_time = Instant::now();
    
    // Run Phase 1 comprehensive testing
    let results = run_phase1_comprehensive_testing();
    
    let total_time = start_time.elapsed();
    
    println!("\n🎯 Phase 1 Testing Complete!");
    println!("==============================");
    println!("Total Execution Time: {:?}", total_time);
    println!("Overall Result: {}", if results.overall_success { "✅ SUCCESS" } else { "⚠️ ISSUES FOUND" });
    println!("Code Coverage: {:.1}%", results.code_coverage_percentage);
    println!("Issues Discovered: {}", results.issues_discovered.len());
    
    if results.overall_success {
        println!("\n✅ Phase 1 PASSED - Ready for Phase 2");
        println!("Next: Quality Assurance & Stability Testing");
    } else {
        println!("\n⚠️ Phase 1 COMPLETED WITH ISSUES");
        println!("Review generated reports before proceeding to Phase 2");
        
        // List critical issues
        let critical_issues: Vec<_> = results.issues_discovered.iter()
            .filter(|i| matches!(i.severity, comprehensive::IssueSeverity::Critical))
            .collect();
            
        if !critical_issues.is_empty() {
            println!("\n🚨 CRITICAL ISSUES (must be resolved):");
            for issue in critical_issues {
                println!("   - {}: {}", issue.test_module, issue.description);
            }
        }
    }
    
    println!("\n📋 Generated Reports:");
    println!("   - issue/mvp/003/phase1/checkpoints/cp10_phase1_complete.yaml");
    println!("   - issue/mvp/003/phase1/phase1_summary_report.md");
    println!("   - issue/mvp/003/phase1/phase1_to_phase2_handoff.md");
    println!("");
    
    // Exit with appropriate code
    if results.overall_success {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
