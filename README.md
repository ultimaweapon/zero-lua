# Zero Lua

Zero Lua is a Rust bindings to Lua 5.4 with 100% safety guarantee and zero overhead in most parts. Thanks to Rust borrow checker and RAII, Zero Lua able to expose stack-based Lua API to the user without losing safety guarantee. When using Zero Lua without unsafe code you should never run into any UB.

## Development

### Generate compile_commands.json (Linux and macOS)

This step is required for [clangd](https://clangd.llvm.org/) to work properly. Install [Bear](https://github.com/rizsotto/Bear) then run the following command:

```sh
bear -- cargo build
```

## License

This project is licensed under either of

- Apache License, Version 2.0,
- MIT license

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
