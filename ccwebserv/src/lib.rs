//! Very simple web server implementation as a coding challenge from John Cricket.

mod app;
mod http;

use app::App;

/// Crate default Result definition.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Main entry function, that encapsules all the Web-Server's functionality in one method, and to
/// be executed in a main function.
pub async fn run_web_server() -> Result<()> {
    let mut app = App::new().await?;
    app.run().await?;
    app.stop().await;
    Ok(())
}
