use lipstyk::extension::LipstykExtension;

#[tokio::main]
async fn main() {
    let ext = LipstykExtension::new();
    let mcp_mode = std::env::args().any(|a| a == "--mcp");

    let result = if mcp_mode {
        omegon_extension::mcp_shim::serve_mcp(ext).await
    } else {
        omegon_extension::serve(ext).await
    };

    if let Err(e) = result {
        eprintln!("lipstyk-agent fatal: {e}");
        std::process::exit(1);
    }
}
