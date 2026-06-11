use glob::glob;
pub use tonic_prost_build;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos: Vec<String> = glob("../../../proto/proto/**/*.proto")?
        .filter_map(Result::ok)
        .map(|p| p.to_string_lossy().into_owned())
        .collect();

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