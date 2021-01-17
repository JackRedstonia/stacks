# Stacks
A work-in-progress 2D game framework purely written in the Rust programming language.

It adopts a model similar to osu!framework and Flutter, where widgets parent other widgets to build user interfaces. It also uses layout rules inspired from Godot.

A few noteworthy features:
- Lazy layout and sizing
- Separate threads for game logic and rendering
- No sweeping garbage collector
- Absolute memory safety
- Short compile times (3-5 seconds on a modern Linux machine)

## Current state
For graphics, Stacks utilises Skia with Vulkan. Only `Immediate` present mode works well for now, `Mailbox`/`Fifo` introduces a very noticeable delay and a fix is unplanned.

Audio is in a to-do status.

The repository currently contains both a binary and a library. The binary crate is the example code for now. To run it, execute `cargo run --release`.

## Compiling
This project depends on SDL2. When all these build dependencies are in place, compiling is simply `cargo build --release`, and the compiled output is a single binary, no dynamic libraries.

### Linux
Any nightly Rust toolchain should work. You will also need development packages in order to compile. For example, on Debian and derivatives, run the following command to install SDL2 development packages and its dependencies.
```sh
$ apt-get install libsdl2-dev libsdl2-mixer-dev pkg-config libdrm-dev libgbm-dev
```

### Windows
Nightly MSVC toolchain is required. Follow the `sdl2` crate's instructions on how to install dependencies.

## License
Mozilla Public License version 2.0. See the LICENSE file for more information.

## Contributing
This repository is actively reviewing and accepting pull requests & issues, especially those that improve performance and code quality. Please follow the same code of conduct as the Rust project when participating.

Any code you contribute to this repository is to be under the same license of Mozilla Public License version 2.0.

## Note for GitHub users
This repository is mirrored from GitLab, at https://gitlab.com/JackRedstonia/stacks.
Development happens and non-master branches appear over there.