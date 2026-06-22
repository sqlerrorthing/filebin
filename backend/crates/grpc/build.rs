use glob::{PatternError, glob};
use std::env;

fn find_protos(pat: &str) -> Result<Vec<String>, PatternError> {
    Ok(glob(pat)?
        .filter_map(Result::ok)
        .map(|p| p.to_string_lossy().into_owned())
        .collect())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = env::var("CARGO_FEATURE_PROTO_IN_ROOT")
        .map(|_| "../../proto")
        .unwrap_or("../../../proto");

    let mut protos: Vec<String> = find_protos(&format!("{proto_root}/proto/**/*.proto"))?;
    protos.extend(find_protos(&format!("{proto_root}/vendor/**/*.proto"))?);

    let includes = [
        format!("{proto_root}/proto"),
        format!("{proto_root}/vendor"),
    ];

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(false)
        .bytes(".folder.v1")
        .extern_path(".google.protobuf", "::pbjson_types")
        .compile_well_known_types(true)
        .compile_protos(
            &protos.iter().map(String::as_str).collect::<Vec<&str>>(),
            &includes.iter().map(String::as_str).collect::<Vec<&str>>(),
        )?;
    Ok(())
}
