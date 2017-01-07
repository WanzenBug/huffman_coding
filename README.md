# huffman_coding

Small library for pure huffman coding in Rust.

This library exposes a reader for decoding encoded data and a writer for encoding data. 

[Documentation](https://docs.rs/huffman-coding)

## Usage
First add this library as dependency to your cargo manifest
```TOML
huffman_coding = "0.1.0"
```

Then, import the library at the start of your main/library
```Rust
extern crate huffman_coding;
```

Finally you can use the exported structs as you please
```Rust
use std::io::Write;
let pseudo_data = vec![0, 0, 1, 2, 2];
let tree = HuffmanTree::new(&pseudo_data[..]);

let mut vec = Vec::new();
{
    let mut writer = HuffmanWriter::new(&mut vec, &tree);
    assert!(writer.write(&[0, 0, 1, 1, 2, 2, 2, 2]).is_ok())
}
```

## Binaries
There are two small example binaries, one for encoding a file, the other for decoding a file.
As these require command line parsing, they are gated behind the feature flag `bin`. To build them, use
`cargo build --features "bin"`
