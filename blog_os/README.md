# Blog OS (A Minimal Rust Kernel)

[![Build Status](https://github.com/phil-opp/blog_os/workflows/Code/badge.svg?branch=post-02)](https://github.com/phil-opp/blog_os/actions?query=workflow%3A%22Code%22+branch%3Apost-02)

This repository contains the source code for the [A Minimal Rust Kernel][post] post of the [Writing an OS in Rust](https://os.phil-opp.com) series.

[post]: https://os.phil-opp.com/minimal-rust-kernel/

**Check out the [master branch](https://github.com/phil-opp/blog_os) for more information.**

## Prerequisites

This project requires a nightly version of Rust and some additional tools:

```bash
# Install Rust nightly
rustup default nightly
rustup update nightly --force

# Add required components
rustup component add rust-src
rustup component add llvm-tools-preview
```

## Building

You can build the project by running:

```
cargo build
```

**Note:** This project uses a custom target specification (`x86_64-blog_os.json`) to avoid compatibility issues with the bootimage tool.

To create a bootable disk image from the compiled kernel, you need to install the [`bootimage`] tool:

[`bootimage`]: https://github.com/rust-osdev/bootimage

```
cargo install bootimage
```

After installing, you can create the bootable disk image by running:

```
cargo bootimage
```

This creates a bootable disk image in the `target/x86_64-blog_os/debug` directory.

Please file an issue if you have any problems.

## Running

You can run the OS in [QEMU] through:

[QEMU]: https://www.qemu.org/

```
cargo run
```

This will automatically:
1. Build the kernel with the custom target
2. Create a bootable disk image using bootimage
3. Launch QEMU to run the OS

### Prerequisites for Running

1. **Install QEMU:**
   - macOS: `brew install qemu`
   - Linux: `sudo apt-get install qemu-system-x86`
   - Windows: Download from [qemu.org](https://www.qemu.org/download/)

2. **Install bootimage:** Already covered above with `cargo install bootimage`

### Alternative: Manual Run

You can also build and run manually:

```bash
# Build the bootimage
cargo bootimage

# Run with QEMU
qemu-system-x86_64 -drive format=raw,file=target/x86_64-blog_os/debug/bootimage-blog_os.bin
```

You can also write the image to an USB stick for booting it on a real machine. On Linux, the command for this is:

```
dd if=target/x86_64-blog_os/debug/bootimage-blog_os.bin of=/dev/sdX && sync
```

Where `sdX` is the device name of your USB stick. **Be careful** to choose the correct device name, because everything on that device is overwritten.

## Configuration Details

### Custom Target Specification
This project uses `x86_64-blog_os.json` as a custom target specification instead of the built-in `x86_64-unknown-none` to ensure compatibility with bootimage. The custom target:
- Disables hardware features unavailable in early boot (SSE, MMX)
- Uses software floating point
- Disables the red zone for interrupt safety
- Uses the rust-lld linker

### Cargo Configuration
The `.cargo/config.toml` file configures:
- Build target: `x86_64-blog_os.json` (custom target)
- Runner: `bootimage runner` for automatic QEMU launching
- Build-std features for core library compilation

## Troubleshooting

### "runner.sh not found" Error
If you get this error, make sure `.cargo/config.toml` uses `bootimage runner` instead of `./runner.sh`.

### Build Errors with x86_64-unknown-none
The project should use the custom target `x86_64-blog_os.json`. Check that `.cargo/config.toml` has:
```toml
[build]
target = "x86_64-blog_os.json"
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Note that this only applies to this git branch, other branches might be licensed differently.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
