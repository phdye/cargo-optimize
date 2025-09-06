# cargo-optimize MVP: Just the Linker

## The Problem
You're waiting 2+ minutes for builds when you could be waiting 30 seconds.

## The Solution
One line that picks the fastest linker on your system. That's it.

## Usage

In any Rust project's `build.rs`:
```rust
fn main() {
    cargo_optimize::auto_configure();
}
```

Or just call it once to generate `.cargo/config.toml`:
```rust
cargo_optimize::auto_configure();
```

## What It Does

1. Checks if you have mold (fastest)
2. Falls back to lld (very fast)  
3. Falls back to gold (fast)
4. Uses default if none found

That's literally it. No magic. No complexity.

## Testing the MVP

```bash
# 1. Build and test the MVP
cargo build
cargo test --test mvp_test -- --nocapture

# 2. Benchmark on a real project
cd /path/to/any/rust/project
bash /path/to/cargo-optimize/benchmark_mvp.sh
```

## Why This Works

- **mold**: 50-70% faster linking
- **lld**: 30-50% faster linking
- **gold**: 20-30% faster linking

One config change. Massive improvement.

## Next Steps (After MVP Works)

Once we prove this simple optimization works:
1. Add Windows/Mac support
2. Add profile-specific configs
3. Add more optimizations

But first, let's prove the simplest case works.

## The Code

The entire implementation is ~50 lines in `src/lib.rs`. 
No dependencies. No complexity. Just results.
