# Rust-tinyraytracer [under implementation]

This repository is a learning project based on
[ssloy/tinyraytracer](https://github.com/ssloy/tinyraytracer). This project is
under [DO WHAT THE FUCK YOU WANT TO PUBLIC
LICENSE](https://en.wikipedia.org/wiki/WTFPL).

The purpose of this project is mainly about Rust language rather than ray
tracing. Therefore, this implementation of this project is separate into two
step,which are 'transfer' from C++ to Rust, and reform code into 'Rusty' way.

## Code transform

In the original project, there was a header file provided for step 1 to 9. In
that header file, several basic data type were provided, including 3-dimension
vector itself and operation override for vectors. For the convenience, Rust
package [vek](https://crates.io/crates/vek) is imported for 3-dimension
vectors and operations.

In the homework branch of original project, [stb
library](https://github.com/nothings/stb) was included for processing JPEG
files. For the simply of this repository, these part is not included yet.

During the code translation, one potential bug in the original project was
[fixed](https://github.com/Mayrixon/rust-tinyraytracer/commit/9591aff1c93467762984b9cee032ee0025ff84a6).
The final translated code could be found
[here](https://github.com/Mayrixon/rust-tinyraytracer/commit/50c208eeaeff842e6e4d0bf26a6a8c73d4b8bf85).
