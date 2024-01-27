//! TODO

pub mod http;

use std::net::{SocketAddr, TcpListener, TcpStream};

use http::StreamRead;

/// Crate default Result definition.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Main entry function, that encapsules all the Web-Server's functionality in one method, and to
/// be executed in a main function.
pub fn run_web_server() -> Result<()> {
    let socket = start_listening()?;

    for stream in socket.incoming() {
        handle_client(stream)?;
    }

    Ok(())
}

fn start_listening() -> Result<TcpListener> {
    let addrs = [
        SocketAddr::from(([127, 0, 0, 1], 80)),
        // SocketAddr::from(([127, 0, 0, 1], 443)),
    ];
    TcpListener::bind(&addrs[..]).map_err(Into::into)
}

fn handle_client(stream: std::io::Result<TcpStream>) -> Result<()> {
    match stream {
        Ok(stream) => {
            let mut reader = std::io::BufReader::new(stream);
            let message = http::Message::from_stream(&mut reader)?;
            println!("{message:?}");
        }
        Err(err) => {
            println!("{err:?}");
        }
    }
    Ok(())
}
