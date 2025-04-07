# Zero Lua

Zero Lua is a Rust bindings to Lua 5.4 with zero overhead in most parts. Thanks to Rust borrow checker and RAII, Zero Lua is able to safely expose stack-based Lua API to the user. When using Zero Lua without unsafe code you should never run into any UB.

Zero Lua use a virtual frame to achieve memory safety with zero cost. Each frame has a starting point in a Lua stack. The frame below this starting point is a parent frame. The parent frame always mutable borrowed by a child frame. Each child frame responsible to release all of its value before release a mutable borrow to the parent frame.

> [!WARNING]
> Zero Lua can only be used with unwinding panic. You will get a compile error if you use abort-on-panic.

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
