# Components

https://docs.espressif.com/projects/rust/book/getting-started/toolchain.html

Rust compiler fork:
- For regular ESP32 => See `espup`, it installs the Rust fork (and linkers etc)
- For Expressif ESP32s => `rustup toolchain install stable --component rust-src`

# Install the `channel = "esp"` toolchain:

```sh
cargo install espup --locked
espup install

# ... later
espup update
```
