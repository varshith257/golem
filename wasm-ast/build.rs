use std::{env, process::Command, io::Result};

fn main() -> Result<()> {

    // if let Ok(protoc_path) = env::var("PROTOC") {
    //     println!("Using PROTOC from environment: {}", protoc_path);
    // } else {
    //     println!("PROTOC environment variable is not set.");
    // }

    // // Check for `protoc` binary
    // let protoc_path = env::var("PROTOC").unwrap_or_else(|_| "protoc".to_string());
    // println!("Checking for protoc binary: {}", protoc_path);

    // if Command::new(&protoc_path).output().is_err() {
    //     panic!(
    //         "Could not find `protoc`. Please ensure it is installed or set the `PROTOC` environment variable \
    //         to the path of the `protoc` binary. Download it from: \
    //         https://github.com/protocolbuffers/protobuf/releases"
    //     );
    // }

    let mut config = prost_build::Config::new();
    config.type_attribute(".", "#[cfg(feature = \"protobuf\")]");
    config.type_attribute(
        ".",
        "#[cfg_attr(feature=\"bincode\", derive(bincode::Encode, bincode::Decode))]",
    );
    config.compile_protos(&["proto/wasm/ast/type.proto"], &["proto/"])?;
    Ok(())
}
