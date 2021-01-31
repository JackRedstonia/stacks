<div align="center"><img src="https://gitlab.com/JackRedstonia/stacks/-/raw/master/stacks.svg"></div>

A work-in-progress 2D game framework purely written in the Rust programming language.

It adopts a model similar to osu!framework and Flutter, where widgets parent other widgets to build user interfaces. It also uses layout rules inspired from Godot.

A few noteworthy features:
- Lazy layout and sizing
- Separate threads for game logic and rendering
- No sweeping garbage collector
- Absolute memory safety
- Short compile times (3-5 seconds on a modern Linux machine, using LLD linker)

## Current state
For graphics, Stacks utilises Skia with Vulkan. Only `Immediate` present mode works well for now, `Mailbox`/`Fifo` introduces a very noticeable delay and a fix is unplanned.

Audio is in a usable state, based on the SoLoud audio library.

The repository currently contains a library and one example. To run the example, execute `cargo run --examples stack_demo --release`.

## Compiling
This project depends on SDL2. When build dependencies are in place, compiling is simply `cargo build --release`, and the compiled output should be shipped with the SDL2 dynamic library file.

### Linux
Any Rust toolchain should work. You will also need development packages in order to compile. For example, on Debian and derivatives, run the following command to install SDL2 development packages.
```sh
$ apt-get install libsdl2-dev
```

### Windows
A MSVC toolchain is required (solely to build `skia-bindings`). Follow the `sdl2` crate's instructions on how to install development dependencies.

## License
Mozilla Public License version 2.0. See the LICENSE file for more information.

## Contributing
This repository is actively reviewing and accepting pull requests & issues, especially those that improve performance and code quality. Please follow the same code of conduct as the Rust project when participating.

Any code you contribute to this repository is to be under the same license of Mozilla Public License version 2.0.

## Note for GitHub users
This repository is mirrored from GitLab, at https://gitlab.com/JackRedstonia/stacks.
Development happens and non-master branches appear over there.