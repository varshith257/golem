# wasm-rpc

## golem-wasm-rpc

Defines data types for [Golem](https://golem.cloud)'s remote function invocation and conversions between them.

- `WitValue` is the WIT-defined generic data type capable of representing an arbitrary value, generated by `wit-bindgen`
- A builder and an extractor API for `WitValue`
- `Value` is a recursive Rust type which is more convenient to work with than `WitValue`. Conversion between `WitValue`
  and `Value` is implemented in both directions.
- Protobuf message types for describing values and types, and a protobuf version of `WitValue` itself and conversion
  from and to `Value` and `WitValue`
- JSON representation of WIT values, as defined in [the Golem docs](https://learn.golem.cloud/docs/template-interface).
- Conversion of `Value` to and from `wasmtime` values

The JSON representation requires additional type information which can be extracted using
the [golem-wasm-ast](https://crates.io/crates/golem-wasm-ast) crate.

## Host and stub mode

The `golem-wasm-rpc` crate can be both used in host and guest environments:

To compile the host version:

```shell
cargo build -p wasm-rpc --no-default-features --features host
```

To compile the guest version, has minimal dependencies and feature set to be used in generated stubs:

```shell
cargo component build -p wasm-rpc --no-default-features --features stub
```

## Feature flags

- `arbitrary` adds an `Arbitrary` instance for `Value`
- `json` adds conversion functions for mapping of a WIT value and type definition to/from JSON
- `protobuf` adds the protobuf message types
- `wasmtime` adds conversion to `wasmtime` `Val` values
- `host` enables all features: `arbitrary`, `json`, `protobuf`, `typeinfo`, and `wasmtime`
- `stub` is to be used in generated WASM stubs and disables all features, and generates guest bindings instead of host
  bindings

## golem-wasm-rpc-stubgen

The `golem-wasm-rpc-stubgen` is a CLI tool to generate the RPC stubs from a component's WIT definition.

## Generate

```shell
Usage: wasm-rpc-stubgen generate [OPTIONS] --source-wit-root <SOURCE_WIT_ROOT> --dest-crate-root <DEST_CRATE_ROOT>

Options:
  -s, --source-wit-root <SOURCE_WIT_ROOT>                
  -d, --dest-crate-root <DEST_CRATE_ROOT>                
  -w, --world <WORLD>                                    
      --stub-crate-version <STUB_CRATE_VERSION>          [default: 0.0.1]
      --wasm-rpc-path-override <WASM_RPC_PATH_OVERRIDE>  
  -h, --help                                             Print help
  -V, --version                                          Print version
```

- `source-wit-root`: The root directory of the component's WIT definition to be called via RPC
- `dest-crate-root`: The target path to generate a new stub crate to
- `world`: The world name to be used in the generated stub crate. If there is only a single world in the source root
  package, no need to specify.
- `stub-crate-version`: The crate version of the generated stub crate
- `wasm-rpc-path-override`: The path to the `wasm-rpc` crate to be used in the generated stub crate. If not specified,
  the latest version of `wasm-rpc` will be used.

The command creates a new Rust crate that is ready to be compiled with

```shell
cargo component build --release
```

The resulting WASM component implements the **stub interface** corresponding to the source interface, found in the
target directory's
`wit/_stub.wit` file. This WASM component is to be composed together with another component that calls the original
interface via WASM RPC.

## Build

```
Usage: wasm-rpc-stubgen build [OPTIONS] --source-wit-root <SOURCE_WIT_ROOT> --dest-wasm <DEST_WASM> --dest-wit-root <DEST_WIT_ROOT>

Options:
  -s, --source-wit-root <SOURCE_WIT_ROOT>                
      --dest-wasm <DEST_WASM>                            
      --dest-wit-root <DEST_WIT_ROOT>                    
  -w, --world <WORLD>                                    
      --stub-crate-version <STUB_CRATE_VERSION>          [default: 0.0.1]
      --wasm-rpc-path-override <WASM_RPC_PATH_OVERRIDE>  
  -h, --help                                             Print help
  -V, --version                                          Print version
```

- `source-wit-root`: The root directory of the component's WIT definition to be called via RPC
- `dest-wasm`: The name of the stub WASM file to be generated
- `dest-wit-root`: The directory name where the generated WIT files should be placed
- `world`: The world name to be used in the generated stub crate. If there is only a single world in the source root
  package, no need to specify.
- `stub-crate-version`: The crate version of the generated stub crate
- `wasm-rpc-path-override`: The path to the `wasm-rpc` crate to be used in the generated stub crate. If not specified,
  the latest version of `wasm-rpc` will be used. It needs to be an **absolute path**.

## Add stub WIT dependency

```shell
Usage: wasm-rpc-stubgen add-stub-dependency [OPTIONS] --stub-wit-root <STUB_WIT_ROOT> --dest-wit-root <DEST_WIT_ROOT>

Options:
  -s, --stub-wit-root <STUB_WIT_ROOT>  
  -d, --dest-wit-root <DEST_WIT_ROOT>  
  -o, --overwrite                      
  -u, --update-cargo-toml                
  -h, --help                           Print help
  -V, --version                        Print version
```

The command merges a generated RPC stub as a WIT dependency into an other component's WIT root.

- `stub-wit-root`: The WIT root generated by either `generate` or `build` command
- `dest-wit-root`: The WIT root of the component where the stub should be added as a dependency
- `overwrite`: This command would not do anything if it detects that it would change an existing WIT file's contents at
  the destination. With this flag, it can be forced to overwrite those files.
- `update-cargo-toml`: Enables updating the Cargo.toml file in the parent directory of `dest-wit-root` with the copied
  dependencies.

## Compose the stub with the caller component

```shell
Usage: wasm-rpc-stubgen compose --source-wasm <SOURCE_WASM> --stub-wasm <STUB_WASM> --dest-wasm <DEST_WASM>

Options:
      --source-wasm <SOURCE_WASM>  
      --stub-wasm <STUB_WASM>      
      --dest-wasm <DEST_WASM>      
  -h, --help                       Print help
  -V, --version                    Print version
```

The command composes a caller component's WASM (which uses the generated stub to call a remote worker) with the
generated stub WASM, writing out a composed WASM which no longer depends on the stub interface, ready to use.

- `source-wasm`: The WASM file of the caller component
- `stub-wasm`: The WASM file of the generated stub. Multiple stubs can be listed.
- `dest-wasm`: The name of the composed WASM file to be generated

## Initialize cargo make tasks for a workspace

```shell
Usage: wasm-rpc-stubgen initialize-workspace [OPTIONS] --targets <TARGETS> --callers <CALLERS>

Options:
      --targets <TARGETS>
          List of subprojects to be called via RPC
      --callers <CALLERS>
          List of subprojects using the generated stubs for calling remote workers
      --wasm-rpc-path-override <WASM_RPC_PATH_OVERRIDE>
```

When both the target and the caller components are in the same Cargo workspace, this command can initialize a `cargo-make` file with dependent tasks
performing the stub generation, WIT merging and WASM composition.

Once the workspace is initialized, the following two commands become available:

```shell
cargo make build-flow
cargo make release-build-flow
```
