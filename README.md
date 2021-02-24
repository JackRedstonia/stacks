<img src="https://gitlab.com/JackRedstonia/stacks/-/raw/master/stacks.svg">

A work-in-progress 2D game framework purely written in the Rust programming language.

It adopts a model similar to osu!framework and Flutter, where widgets parent other widgets to build user interfaces. It also uses layout rules inspired from Godot.

A few noteworthy features:
- Lazy layout and sizing
- Separate threads for game logic and rendering
- No sweeping garbage collector
- Absolute memory safety
- Short compile times (3-5 seconds on a modern Linux machine, using LLD linker)

## Current state
For graphics, Stacks utilises Skia with Vulkan. Audio is in a usable state, based on the SoLoud audio library.

The repository currently contains a library and a few examples as demos. To run them, execute `cargo run --example <name> --release`.

Currently, the available demos are:
- `button`: features the `Button` widget.
- `paragraph`: features the `Text` widget, used for basic text layout.
- `stacks_demo`: The legacy demo ported to current codebase, kept for nostalgia.

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
MIT license. See the LICENSE file for more information.

## Contributing
This repository is actively reviewing and accepting pull requests & issues, especially those that improve performance and code quality. When participating in this project, please follow the Code of Conduct, as specified by the [`CODE_OF_CONDUCT.md` file](CODE_OF_CONDUCT.md).

Any code you contribute to this repository is to be under the MIT license.

## Note for GitHub users
This repository is mirrored from GitLab. Development and non-`master` branches appear over at https://gitlab.com/JackRedstonia/stacks.
