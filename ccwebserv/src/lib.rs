//! Very simple web server implementation as a coding challenge from John Cricket.
//!
//! # Next Steps
//!
//! - Implement basic HTML <-> Stream transformations (TX & RX)
//! - Sent simple response:
//!   ```
//!   HTTP/1.1 200 OK\r\n\r\nRequested path: <the path>\r\n
//!   ```
//! - If path is matching, e.g. '/' or '/index.html' return content of this page

pub mod http;

use std::{
    fs,
    io::{self, Write},
    net::{SocketAddr, TcpListener, TcpStream},
};

use http::{Method, StreamRead};

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
    let addrs = [SocketAddr::from(([127, 0, 0, 1], 80))];
    TcpListener::bind(&addrs[..]).map_err(Into::into)
}

fn handle_client(stream: io::Result<TcpStream>) -> Result<()> {
    match stream {
        Ok(stream) => {
            let mut reader = io::BufReader::new(stream);
            let message = http::Message::from_stream(&mut reader)?;
            let mut stream = reader.into_inner();

            match message.startline.method {
                Method::Get => {
                    get_request(&message, &mut stream)?;
                }
                _ => return Err(format!("message: {message:?} / not supported").into()),
            }
        }
        Err(err) => {
            println!("{err:?}");
        }
    }
    Ok(())
}

fn get_request(message: &http::Message, stream: &mut TcpStream) -> Result<()> {
    let path = if message.startline.target == "/" {
        std::path::PathBuf::from("website/index.html")
    } else {
        std::path::PathBuf::from(format!("website{}", message.startline.target).as_str())
    };

    let exists = path.exists();
    let version: String = message.startline.version.clone().into();
    let stcode = if exists {
        Into::<String>::into(http::ScSuccessful::Ok)
    } else {
        Into::<String>::into(http::ScClientError::NotFound)
    };

    let response = format!("{version} {stcode}\r\n\r\n");
    stream.write_all(response.as_bytes())?;

    if exists {
        let file = fs::read_to_string(&path)?;
        stream.write_all(file.as_bytes())?;
    }

    Ok(())
}
