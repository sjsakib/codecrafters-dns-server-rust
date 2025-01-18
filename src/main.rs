use codecrafters_dns_server::{DnsServer, ServerConfig};
use clap::Parser;

/// A toy DNS server
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// address of resolver to use
    #[arg(short, long)]
    resolver: Option<String>,

    /// port to listen on
    #[arg(short, long, default_value_t = 2053)]
    port: u16,
}

fn main() {
    let args = Args::parse();

    let server = DnsServer::init(ServerConfig {
        port: args.port,
        resolver: args.resolver,
    });

    server.listen();
}
