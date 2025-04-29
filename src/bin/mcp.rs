use clap::{Parser, Subcommand};
use codebank::CodeBankMcp;
use rmcp::{
    ServiceExt,
    transport::{SseServer, sse_server::SseServerConfig, stdio},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run in stdio mode
    Stdio,
    /// Run in SSE mode
    Sse {
        /// Port for the SSE server to bind to
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the tracing subscriber with file and stdout logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Stdio => run_stdio_mode().await?,
        Commands::Sse { port } => run_sse_mode(port).await?,
    }

    Ok(())
}

async fn run_stdio_mode() -> anyhow::Result<()> {
    tracing::info!("Starting CodeBank MCP server in stdio mode");

    // Create an instance of our CodeBankMcp router
    let service = CodeBankMcp::new().serve(stdio()).await.inspect_err(|e| {
        tracing::error!("serving error: {:?}", e);
    })?;

    service.waiting().await?;

    Ok(())
}

async fn run_sse_mode(port: u16) -> anyhow::Result<()> {
    tracing::info!("Starting CodeBank MCP server in SSE mode on port {}", port);

    let addr = format!("0.0.0.0:{}", port);
    // Store bind address and cancellation token separately
    let bind_addr: std::net::SocketAddr = addr.parse()?;
    let ct_main = tokio_util::sync::CancellationToken::new();

    let config = SseServerConfig {
        bind: bind_addr, // Use stored address
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
        // Clone the token for the config
        ct: ct_main.clone(),
    };

    let sse_server = SseServer::serve_with_config(config).await?;

    let service_ct = sse_server.with_service(CodeBankMcp::new);

    tokio::signal::ctrl_c().await?;
    tracing::info!("Ctrl-C received, shutting down...");
    service_ct.cancel(); // Cancel the service
    // Cancel the server itself using the main token
    ct_main.cancel();

    Ok(())
}
