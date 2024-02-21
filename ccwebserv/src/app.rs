//! States and data that handles the application.

use crate::{
    http::{self, Method},
    Result,
};
use std::{fs, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    task::JoinSet,
    time::Duration,
};

/// The application itself.
#[derive(Debug)]
pub struct App {
    /// TCP-Listener.
    listener: Arc<TcpListener>,
    /// Connected clients to be processed.
    clients: Arc<Mutex<Vec<(TcpStream, SocketAddr)>>>,
    /// Running flag (otherwise stop listening and handling).
    running: Arc<bool>,
}

impl App {
    pub async fn new() -> Result<App> {
        let addrs = [SocketAddr::from(([127, 0, 0, 1], 80))];
        Ok(App {
            listener: Arc::new(TcpListener::bind(&addrs[..]).await?),
            clients: Arc::new(Mutex::new(vec![])),
            running: Arc::new(true),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Spawn both processes and wait for them to the end.
        let mut set = JoinSet::new();

        println!("prepare to listen");

        let listener = Arc::clone(&self.listener);
        let clients = Arc::clone(&self.clients);
        let running = Arc::clone(&self.running);
        set.spawn(async move {
            listen(listener, clients, running).await;
        });

        println!("prepare to handle clients");

        let clients = Arc::clone(&self.clients);
        let running = Arc::clone(&self.running);
        set.spawn(async move {
            handle_clients(clients, running).await;
        });

        set.join_next().await;
        set.join_next().await;

        Ok(())
    }

    pub async fn stop(self) {}
}

/// One main process is listening.
async fn listen(
    listener: Arc<TcpListener>,
    clients: Arc<Mutex<Vec<(TcpStream, SocketAddr)>>>,
    running: Arc<bool>,
) {
    let mut running_int = true;
    while running_int {
        match listener.accept().await {
            Ok((socket, addr)) => {
                println!("incoming connection...");
                let clients_clone = Arc::clone(&clients);
                tokio::spawn(async move {
                    let mut lock = clients_clone.lock().await;
                    lock.push((socket, addr));
                    println!("added a new client from addr");
                });
            }
            Err(_err) => {
                unimplemented!();
            }
        }
        running_int = *running;
    }
}

async fn handle_clients(clients: Arc<Mutex<Vec<(TcpStream, SocketAddr)>>>, running: Arc<bool>) {
    let mut set = JoinSet::new();
    let mut running_int = true;

    while running_int {
        let mut lock = clients.lock().await;
        if lock.is_empty() {
            drop(lock);
            tokio::time::sleep(Duration::from_millis(50)).await;
        } else if let Some((stream, addr)) = lock.pop() {
            drop(lock);
            set.spawn(async move {
                let _ = tokio::time::timeout(Duration::from_secs(5), handle_client(stream, addr)).await;
            });
            println!("spawned a new handle-client task");
        }
        running_int = *running;
    }

    set.abort_all();
}

/// Other main process is handling those clients in our waiting list.
async fn handle_client(mut stream: TcpStream, _addr: SocketAddr) -> Result<()> {
    let mut buffer = vec![0u8; 1024];
    let mut receiving = true;
    let mut n_bytes = 0;

    let ends_with = |buffer: &[u8], n_bytes: usize| -> bool {
        if n_bytes > 4 {
            if buffer[n_bytes - 4] != b'\r' {
                return false;
            }
            if buffer[n_bytes - 3] != b'\n' {
                return false;
            }
            if buffer[n_bytes - 2] != b'\r' {
                return false;
            }
            if buffer[n_bytes - 1] != b'\n' {
                return false;
            }
            true
        } else {
            false
        }
    };

    stream.readable().await?;
    while receiving {
        n_bytes += stream.read(&mut buffer).await?;
        receiving = !ends_with(&buffer[..], n_bytes);
    }
    let message = http::Message::try_from(std::str::from_utf8(&buffer[..])?)?;

    match message.startline.method {
        Method::Get => {
            get_request(&message, &mut stream).await?;
        }
        _ => return Err(format!("message: {message:?} / not supported").into()),
    }

    Ok(())
}

/// Simple method to process file content returning.
async fn get_request(message: &http::Message, stream: &mut TcpStream) -> Result<()> {
    let path = if message.startline.target == "/" {
        std::path::PathBuf::from("website/index.html")
    } else {
        std::path::PathBuf::from(format!("website{}", message.startline.target).as_str())
    };

    let exists = path.exists();
    let version = Into::<&str>::into(message.startline.version.clone()).to_string();
    let stcode = if exists {
        Into::<&str>::into(http::ScSuccessful::Ok)
    } else {
        Into::<&str>::into(http::ScClientError::NotFound)
    };

    let response = format!("{version} {stcode}\r\n\r\n");
    let _ = stream.write_all(response.as_bytes()).await;

    if exists {
        let file = fs::read_to_string(&path)?;
        let _ = stream.write_all(file.as_bytes()).await;
    }

    Ok(())
}
