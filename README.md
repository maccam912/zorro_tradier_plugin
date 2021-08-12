# Building for Zorro

Zorro expects a 32-big dll. To build for 32-bit on Windows, add the target `i686-pc-windows-msvc` and build with `cargo build --target=i686-pc-windows-msvc`.

Then copy the DLL to the `Plugin` folder for Zorro.

# Cross compiling

Also included  is a cross.toml that will let you use (https://github.com/rust-embedded/cross)[cross] to build on any system. Install with `cargo install cross` and build with `cross build --target=i686-pc-windows-gnu`

Then copy the DLL to the `Plugin` folder for Zorro.