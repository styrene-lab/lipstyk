use lipstyk::extension::LipstykExtension;

#[tokio::main]
async fn main() {
    let ext = LipstykExtension::new();

    // The extension handles both direct RPC and tools/call dispatch
    // internally, so MCP compatibility works through the omegon
    // mcp_shim when available, or via the native RPC protocol.
    if let Err(e) = omegon_extension::serve(ext).await {
        eprintln!("lipstyk-agent fatal: {e}");
        std::process::exit(1);
    }
}
