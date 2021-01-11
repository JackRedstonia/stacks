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

Audio is a work-in-progress effort, entirely based on GStreamer.

The repository currently contains both a binary and a library. The binary crate is the example code for now. To run it, execute `cargo run --release`.

## Compiling
This project depends on SDL2 and GStreamer. When all these build dependencies are in place, compiling is simply `cargo build --release`.

### Linux
Any nightly Rust toolchain should work. Run the following command to install SDL2 & GStreamer.
```sh
$ apt-get install libsdl2-dev libgstreamer1.0-dev \
      libgstreamer-plugins-base1.0-dev gstreamer1.0-plugins-base \
      gstreamer1.0-plugins-good gstreamer1.0-plugins-bad \
      gstreamer1.0-plugins-ugly gstreamer1.0-libav libgstrtspserver-1.0-dev
```
When shipping your applications, remember to include the SDL2 and GStreamer dynamic libraries.

### Windows
Nightly MSVC toolchain is required. Follow the `sdl2` and `gstreamer` crates' instructions on how to install dependencies.

## License
Mozilla Public License version 2.0. See the LICENSE file for more information.

## Contributing
This repository is actively reviewing and accepting pull requests & issues, especially those that improve performance and code quality. Please follow the same code of conduct as the Rust project when participating.

Any code you contribute to this repository is to be under the same license of Mozilla Public License version 2.0.

## Note for GitHub users
This repository is mirrored from GitLab, at https://gitlab.com/JackRedstonia/stacks.
Development happens and non-master branches appear over there.