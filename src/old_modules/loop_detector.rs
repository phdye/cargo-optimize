//! Loop detection and timeout utilities for preventing infinite loops

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Loop detector that can detect and break infinite loops
pub struct LoopDetector {
    /// Maximum iterations allowed
    max_iterations: usize,
    /// Current iteration count
    current_iterations: Arc<AtomicUsize>,
    /// Timeout duration
    timeout: Duration,
    /// Start time
    start_time: Instant,
    /// Flag to indicate loop should stop
    should_stop: Arc<AtomicBool>,
    /// Context for debugging
    context: String,
}

impl LoopDetector {
    /// Create a new loop detector
    pub fn new(context: impl Into<String>) -> Self {
        Self {
            max_iterations: 1000,  // Default max iterations
            current_iterations: Arc::new(AtomicUsize::new(0)),
            timeout: Duration::from_secs(30),  // Default 30 second timeout
            start_time: Instant::now(),
            should_stop: Arc::new(AtomicBool::new(false)),
            context: context.into(),
        }
    }

    /// Set maximum iterations
    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    /// Set timeout duration
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Start monitoring in a background thread
    pub fn start_monitoring(&self) {
        // Skip background monitoring in test mode to avoid hanging threads
        if cfg!(test) || std::env::var("CARGO_TEST").is_ok() {
            return;
        }
        
        let should_stop = self.should_stop.clone();
        let timeout = self.timeout;
        let context = self.context.clone();
        
        thread::spawn(move || {
            thread::sleep(timeout);
            if !should_stop.load(Ordering::Relaxed) {
                eprintln!("WARNING: Operation '{}' exceeded timeout of {:?}", context, timeout);
                should_stop.store(true, Ordering::Relaxed);
            }
        });
    }

    /// Check if we should continue the loop
    pub fn should_continue(&self) -> bool {
        // Check if we've exceeded max iterations
        let iterations = self.current_iterations.fetch_add(1, Ordering::Relaxed);
        if iterations >= self.max_iterations {
            eprintln!("WARNING: Loop in '{}' exceeded max iterations ({})", 
                     self.context, self.max_iterations);
            self.should_stop.store(true, Ordering::Relaxed);
            return false;
        }

        // Check if timeout has been exceeded
        if self.start_time.elapsed() > self.timeout {
            eprintln!("WARNING: Loop in '{}' exceeded timeout ({:?})", 
                     self.context, self.timeout);
            self.should_stop.store(true, Ordering::Relaxed);
            return false;
        }

        // Check if we've been told to stop
        !self.should_stop.load(Ordering::Relaxed)
    }

    /// Mark the operation as complete
    pub fn complete(&self) {
        self.should_stop.store(true, Ordering::Relaxed);
    }

    /// Get the number of iterations performed
    pub fn iterations(&self) -> usize {
        self.current_iterations.load(Ordering::Relaxed)
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// Timeout guard that ensures operations complete within a time limit
pub struct TimeoutGuard {
    context: String,
    timeout: Duration,
    start_time: Instant,
    completed: Arc<AtomicBool>,
}

impl TimeoutGuard {
    /// Create a new timeout guard
    pub fn new(context: impl Into<String>, timeout: Duration) -> Self {
        let guard = Self {
            context: context.into(),
            timeout,
            start_time: Instant::now(),
            completed: Arc::new(AtomicBool::new(false)),
        };

        // Skip background monitoring in test mode to avoid hanging threads
        if !cfg!(test) && std::env::var("CARGO_TEST").is_err() {
            // Start monitoring thread
            let completed = guard.completed.clone();
            let context = guard.context.clone();
            let timeout = guard.timeout;
            
            thread::spawn(move || {
                thread::sleep(timeout);
                if !completed.load(Ordering::Relaxed) {
                    eprintln!("ERROR: Operation '{}' timed out after {:?}", context, timeout);
                    // In a real implementation, we might want to panic or take other action
                }
            });
        }

        guard
    }

    /// Mark the operation as complete
    pub fn complete(self) {
        self.completed.store(true, Ordering::Relaxed);
    }

    /// Check if the timeout has been exceeded
    pub fn is_expired(&self) -> bool {
        self.start_time.elapsed() > self.timeout
    }
}

impl Drop for TimeoutGuard {
    fn drop(&mut self) {
        if !self.completed.load(Ordering::Relaxed) {
            let elapsed = self.start_time.elapsed();
            if elapsed > self.timeout {
                eprintln!("WARNING: Operation '{}' exceeded timeout. Elapsed: {:?}, Limit: {:?}",
                         self.context, elapsed, self.timeout);
            }
        }
    }
}

/// Rate limiter to prevent operations from running too frequently
pub struct RateLimiter {
    last_execution: Arc<std::sync::Mutex<Option<Instant>>>,
    min_interval: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(min_interval: Duration) -> Self {
        Self {
            last_execution: Arc::new(std::sync::Mutex::new(None)),
            min_interval,
        }
    }

    /// Check if we can execute now
    pub fn can_execute(&self) -> bool {
        let now = Instant::now();
        let mut last_execution = self.last_execution.lock().unwrap();
        
        match *last_execution {
            None => {
                // First execution
                *last_execution = Some(now);
                true
            }
            Some(last) => {
                if now.duration_since(last) >= self.min_interval {
                    *last_execution = Some(now);
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Wait until we can execute
    pub fn wait_until_ready(&self) {
        while !self.can_execute() {
            thread::sleep(Duration::from_millis(10));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_detector_max_iterations() {
        let detector = LoopDetector::new("test")
            .with_max_iterations(10);
        
        let mut count = 0;
        while detector.should_continue() {
            count += 1;
            if count > 20 {
                panic!("Loop detector failed to stop");
            }
        }
        
        assert_eq!(count, 10);
    }

    #[test]
    fn test_timeout_guard() {
        let guard = TimeoutGuard::new("test", Duration::from_millis(100));
        thread::sleep(Duration::from_millis(50));
        assert!(!guard.is_expired());
        guard.complete();
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(Duration::from_millis(100));
        
        assert!(limiter.can_execute());
        assert!(!limiter.can_execute()); // Should be rate limited
        
        thread::sleep(Duration::from_millis(110));
        assert!(limiter.can_execute());
    }
}
