Invalid Operation Bug
=====================



Prerequisites
-------------

The following section lists the prerequisites for building this game from source.

### Rust

In order to build this game from source, you need a Rust compiler (v1.65 or newer) and the
Cargo package manager, you can get both from [here][rust-get-started].

### Native

The Rust compiler should be enough.

### Web

For building a WASM assembly, you need the Wasm32 target, if you have
`rustup` you can easily add Wasm32 support via:

```sh
rustup target add wasm32-unknown-unknown
```

As well as the `wasm-bindgen` tool at exactly `v0.2.85`, which you can install via Cargo:

```sh
cargo install -f --locked wasm-bindgen-cli --version 0.2.85
```

Additionally, to easily serve such a WASM assembly, you might like some simple
HTTP server that dose just that, such as [simple-http-server], which you can
install via Cargo:

```sh
cargo install simple-http-server
```

[simple-http-server]: https://crates.io/crates/simple-http-server
[rust-get-started]: https://www.rust-lang.org/learn/get-started



Building & Running
------------------


### Native

To run this game natively, just execute the following:

```sh
cargo run
```


### Web

To run this game via WASM in a browser you first have to build the WASM assembly via:

```sh
./build-for-web.sh
```

And then you can serve the generated `target/web-pkg` with whatever plain HTTP server you like, e.g.:

```sh
simple-http-server --index --nocache target/web-pkg
```

And head to <http://localhost:8000>



#### `wasm-bindgen` compatibility

This project depends on `wasm-bindgen` as JS-glue and thus is required as build tool.
Thus, the version of `wasm-bindgen` used as dependency must be compatible
with the version of the CLI tool used as build tool. Which actually means
that the version of the CLI tool must be exactly the same as the version
of the dependency, namely **`v0.2.85`**.


