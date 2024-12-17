# cargo-dep-analysis

A quick-and-dirty tool to call attention to potentially-unused crates in your Rust project.

## Usage

```bash
# Build cargo-dep-analysis
mkdir -p ~/git
cd ~/git
git clone https://github.com/nick1udwig/cargo-dep-analysis
cd cargo-dep-analysis
cargo build

# Use cargo-dep-analysis
cd ~/path/to/my/rust/project
~/git/cargo-dep-analysis/target/debug/cargo-dep-analysis
```

There will be false positives, so I'd recommend running a `grep -r` on each hit to confirm it doesn't occur.
E.g., say I have a potentially unused crate `foo`.
I should then run
```bash
grep -r foo --include="*.rs"
```
to confirm that that crate is not used in my project.

In particular one known failure case is build deps and `build.rs`, which will be tagged as potentially unused when they are, in fact, used.
