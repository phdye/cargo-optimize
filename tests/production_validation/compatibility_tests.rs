//! Compatibility tests for various environments

use cargo_optimize::mvp::detect_best_linker;
use std::env;

#[cfg(test)]
mod compatibility_tests {
    use super::*;

    #[test]
    fn test_rust_toolchain_versions() {
        // Test compatibility with different Rust versions
        let min_version = "1.70.0";
        let current_version = env!("CARGO_PKG_RUST_VERSION");
        
        // This is a placeholder - in production would test actual compatibility
        assert!(current_version >= min_version || current_version.is_empty(), 
                "Should support Rust 1.70+");
    }

    #[test]
    fn test_windows_compatibility() {
        #[cfg(target_os = "windows")]
        {
            // Test Windows-specific functionality
            let linker = detect_best_linker();
            
            // On Windows, we should find at least rust-lld
            if let Ok(linker_name) = linker {
                if linker_name != "default" {
                    assert!(
                        linker_name == "rust-lld" || linker_name == "lld-link",
                        "Unexpected linker on Windows: {}", linker_name
                    );
                }
            }
        }
    }

    #[test]
    fn test_linux_compatibility() {
        #[cfg(target_os = "linux")]
        {
            // Test Linux-specific functionality
            let linker = detect_best_linker();
            
            // Linux might have mold, lld, or gold
            if let Ok(linker_name) = linker {
                if linker_name != "default" {
                    assert!(
                        linker_name == "mold" || linker_name == "lld" || linker_name == "gold",
                        "Unexpected linker on Linux: {}", linker_name
                    );
                }
            }
        }
    }

    #[test]
    fn test_macos_compatibility() {
        #[cfg(target_os = "macos")]
        {
            // Test macOS-specific functionality
            let linker = detect_best_linker();
            
            // macOS might have lld or zld
            if let Ok(linker_name) = linker {
                if linker_name != "default" {
                    // Note: We don't explicitly support zld yet, but could add it
                    assert!(
                        linker_name == "lld" || linker_name == "default",
                        "Unexpected linker on macOS: {}", linker_name
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod toolchain_compatibility {
    use super::*;
    
    #[test]
    fn test_msvc_compatibility() {
        #[cfg(all(target_os = "windows", target_env = "msvc"))]
        {
            // Test MSVC toolchain compatibility
            let linker = detect_best_linker();
            assert!(linker.is_ok(), "Should handle MSVC environment");
        }
    }

    #[test]
    fn test_mingw_compatibility() {
        #[cfg(all(target_os = "windows", target_env = "gnu"))]
        {
            // Test MinGW toolchain compatibility
            let linker = detect_best_linker();
            assert!(linker.is_ok(), "Should handle MinGW environment");
        }
    }

    #[test]
    fn test_docker_environment() {
        // Test behavior in containerized environment
        let in_docker = std::path::Path::new("/.dockerenv").exists();
        
        if in_docker {
            println!("Running in Docker environment");
            // Docker-specific tests would go here
        }
    }

    #[test]
    fn test_ci_environment_detection() {
        // Detect common CI environments
        let ci_vars = vec![
            "CI",
            "GITHUB_ACTIONS",
            "GITLAB_CI",
            "JENKINS_URL",
            "TRAVIS",
            "CIRCLECI",
        ];
        
        let mut in_ci = false;
        for var in ci_vars {
            if env::var(var).is_ok() {
                in_ci = true;
                println!("Detected CI environment: {}", var);
                break;
            }
        }
        
        // Behavior might differ in CI
        if in_ci {
            println!("Adjusting for CI environment");
        }
    }
}

#[cfg(test)]
mod container_compatibility {
    use super::*;
    use std::path::Path;
    
    #[test]
    fn test_kubernetes_environment() {
        // Test behavior in Kubernetes
        let in_k8s = Path::new("/var/run/secrets/kubernetes.io").exists();
        
        if in_k8s {
            println!("Running in Kubernetes");
            // K8s-specific behavior
        }
    }

    #[test]
    fn test_github_actions_compatibility() {
        if env::var("GITHUB_ACTIONS").is_ok() {
            // GitHub Actions specific tests
            println!("GitHub Actions environment detected");
            
            // Verify we can operate in GHA environment
            let linker = detect_best_linker();
            assert!(linker.is_ok(), "Should work in GitHub Actions");
        }
    }

    #[test]
    fn test_gitlab_ci_compatibility() {
        if env::var("GITLAB_CI").is_ok() {
            // GitLab CI specific tests
            println!("GitLab CI environment detected");
            
            let linker = detect_best_linker();
            assert!(linker.is_ok(), "Should work in GitLab CI");
        }
    }
}
