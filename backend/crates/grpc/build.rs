use glob::{glob, PatternError};
use tonic_prost_build;

fn find_protos(pat: &str) -> Result<Vec<String>, PatternError> {
    Ok(glob(pat)?
        .filter_map(Result::ok)
        .map(|p| p.to_string_lossy().into_owned())
        .collect())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut protos: Vec<String> = find_protos("../../../proto/proto/**/*.proto")?;
    protos.extend(find_protos("../../../proto/vendor/**/*.proto")?);

    let includes = vec![
        "../../../proto/proto".to_string(),
        "../../../proto/vendor".to_string(),
    ];

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &protos.iter().map(String::as_str).collect::<Vec<&str>>(),
            &includes.iter().map(String::as_str).collect::<Vec<&str>>(),
        )?;
    Ok(())
}
