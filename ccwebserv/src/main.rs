//! Main executable is just using the library's implementation.

#[tokio::main]
async fn main() -> ccwebserv::Result<()> {
    ccwebserv::run_web_server().await
}
