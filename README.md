# Gate Demo
A simple 2D platformer game that demonstrates the use of two Rust crates:
[Gate](https://crates.io/crates/gate) and [Collider](https://crates.io/crates/gate_build).

### Build Instructions

First, if you have not already done so, install the Rust programming language.
Instructions may be found at <https://www.rust-lang.org/>.

Next, since this game depends on Gate which depends on SDL2,
install the SDL2 development libraries.
Instructions may be found at <https://github.com/Rust-SDL2/rust-sdl2#sdl20-development-libraries>.
You need to setup SDL2_Image and SDL2_Mixer as well.
This is pretty easy on Linux, more difficult on Windows.
OpenGL 3.0 or higher is required.

With `gate_demo` set as the current directory, invoke `cargo run` on the command line
(cargo is the package management system for Rust, and should have been installed
when you installed Rust).
This will build and run the game.

### License

The Gate Demo source code is licensed under the
[GNU General Public License v3.0](https://www.gnu.org/licenses/gpl-3.0.en.html).
The Gate Demo assets, found in the `src_assets/` directory, are licensed under the
[Creative Commons Attribution-NonCommercial-ShareAlike 4.0 License](https://creativecommons.org/licenses/by-nc-sa/4.0/).
