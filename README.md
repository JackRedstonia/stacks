<img src="https://gitlab.com/JackRedstonia/stacks/-/raw/master/stacks.svg">

A work-in-progress 2D game framework purely written in the Rust programming language.

It adopts a model similar to osu!framework and Flutter, where widgets parent other widgets to build user interfaces. It also uses layout rules inspired from Godot.

## Current state
Stacks' is in an "experimental" phase, and as such its implementation and API rapidly change. ***Very*** big and ***very*** breaking changes are to be expected. Large portions of Stacks are not documented yet, but anywhere `unsafe` is used, at least a `SAFETY: ...` comment is expected.

Stacks uses Skia, Winit and OpenGL. Audio is in a usable state, based on the SoLoud audio library, though it is planned to be replaced by a new audio library written in Rust in the future due to it being written in C++ and several showstopping bugs preventing production use. Once that happens, only `.ogg` files will have planned support.

The repository currently contains a library and a few examples as demos. To run them, execute `cargo run --example <name> --release`.

Currently, the available demos are:
- `paragraph`: features the `Text` widget, used for basic text layout.
- `stacks_demo`: The legacy demo ported to current codebase, kept for nostalgia.
- `textedit`: features the `TextEdit` widget and nothing else. Click anywhere to focus on the widget and begin typing.
- `ui`: features the `Button` and `Slider` widget.

## Compiling
This project manages its dependencies with Cargo, compiling is simply `cargo build --release`.

### Linux
Latest versions of Rust stable should work - it's best to do your own testing for your needs.

### Windows
A MSVC toolchain is required (solely to build `skia-bindings`).

## License
For code, see the LICENSE file. For the Fira Sans fonts bundled, see the fonts' respective licenses.

## Contributing
This repository is actively reviewing and accepting pull requests & issues, especially those that add documentation, improve performance or code quality. Any code you contribute to this repository is to be under the GNU Affero GPL license version 3, no exceptions. Assets may go under another license, but it must be permissive or copyleft.

## Note for GitHub users
This repository is mirrored from GitLab. Development and non-`master` branches appear over at https://gitlab.com/JackRedstonia/stacks.
