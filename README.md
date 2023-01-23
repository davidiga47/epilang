# Epilang
Epilang is a high-level, general-purpose programming language.

The syntax is quite similar to Rust, while the programmer does not need to worry about memory management.

```rust
// Quick Epilang example
let x = 3;
fn sum(x, y) { x + y };
let result = if (sum(3, x) < 6) { false } else { true };
// returns `true`
result
```

Pass a file path as first argument to run it:
```bash
# At the moment we recommend to use the .rs extension
# in order to have Rust syntax highlighting in text editors
$ epilang path/to/file.rs
```

or pass no args to open an interactive shell:
```bash
# Open interactive Epilang shell
$ epilang
```

## Build from sources
To build this project from source you need [Rust](https://www.rust-lang.org/). To install it follow the instructions on the official [installation page](https://www.rust-lang.org/tools/install).

After installing you must be able to run `cargo` from the command line:
```bash
# Print cargo version
$ cargo -V
cargo 1.66.0 (d65d197ad 2022-11-15)
```

Download Epilang source code from this repository and navigate to the source code directory on the command line:
```bash
git clone https://github.com/epieffe/epilang.git
cd epilang
```

To build the project run the following command:
```bash
$ cargo build --release
```
This command creates an executable file in `targert/release/epilang`. Run this file to start Epilang.

To compile the code and then run the resultant executable all in one command run the following:
``` bash
cargo run
```
