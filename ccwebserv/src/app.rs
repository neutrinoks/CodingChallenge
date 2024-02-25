//! States and data that handles the application.

use crate::{
    http::{self, Method},
    Result,
};
use std::{fs, net::SocketAddr, path::PathBuf, sync::Arc};
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
    /// Running flag shared in main tasks.
    stop_signal: Arc<Mutex<bool>>,
}

impl App {
    pub async fn new() -> Result<App> {
        let addrs = [SocketAddr::from(([127, 0, 0, 1], 80))];
        Ok(App {
            listener: Arc::new(TcpListener::bind(&addrs[..]).await?),
            clients: Arc::new(Mutex::new(vec![])),
            stop_signal: Arc::new(Mutex::new(false)),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Spawn both processes and wait for them to the end.
        let mut set = JoinSet::new();

        println!("Prepare listening...");

        let listener = Arc::clone(&self.listener);
        let clients = Arc::clone(&self.clients);
        let stop_signal = Arc::clone(&self.stop_signal);
        set.spawn(async move {
            listen(listener, clients, stop_signal).await;
        });

        println!("Prepare client handling...");

        let clients = Arc::clone(&self.clients);
        let stop_signal = Arc::clone(&self.stop_signal);
        set.spawn(async move {
            handle_clients(clients, stop_signal).await;
        });

        let stop_signal = Arc::clone(&self.stop_signal);
        ctrlc_async::set_async_handler(async move {
            println!("Shutting server down...");
            let mut lock = stop_signal.lock().await;
            *lock = true;
        })?;

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
    stop_signal: Arc<Mutex<bool>>,
) {
    let limit = Duration::from_secs(1);

    loop {
        if let Ok(result) = tokio::time::timeout(limit, listener.accept()).await {
            match result {
                Ok((socket, addr)) => {
                    let clients_clone = Arc::clone(&clients);
                    tokio::spawn(async move {
                        let mut lock = clients_clone.lock().await;
                        lock.push((socket, addr));
                    });
                }
                Err(_err) => {
                    unimplemented!();
                }
            }
        }

        // Check for stop signal
        let lock = stop_signal.lock().await;
        if *lock {
            break;
        }
    }
    println!("Stoped listening");
}

async fn handle_clients(
    clients: Arc<Mutex<Vec<(TcpStream, SocketAddr)>>>,
    stop_signal: Arc<Mutex<bool>>,
) {
    let mut set = JoinSet::new();

    loop {
        let mut lock = clients.lock().await;
        if lock.is_empty() {
            drop(lock);
            tokio::time::sleep(Duration::from_millis(50)).await;
        } else if let Some((stream, addr)) = lock.pop() {
            drop(lock);
            set.spawn(async move {
                let _ =
                    tokio::time::timeout(Duration::from_secs(5), handle_client(stream, addr)).await;
            });
        } else {
            drop(lock);
        }

        // Check for stop signal
        let lock = stop_signal.lock().await;
        if *lock {
            break;
        }
    }

    set.abort_all();
    println!("Stoped handling clients");
}

/// Other main process is handling those clients in our waiting list.
async fn handle_client(mut stream: TcpStream, addr: SocketAddr) -> Result<()> {
    println!("New client at {addr:?}");

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
    let path = get_path(&message.startline)?;
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

fn get_path(startline: &http::StartLine) -> Result<PathBuf> {
    let mut website = website_path()?;
    let mut req_target = website_path()?;

    if startline.target.has_root() {
        let target: PathBuf = startline.target.iter().skip(1).collect();
        req_target.push(target);
    } else {
        req_target.push(&startline.target);
    }
    let req_target = absolutize(req_target)?;

    if (req_target == website) || !req_target.starts_with(&website) {
        website.push("index.html");
        Ok(website)
    } else {
        Ok(req_target)
    }
}

fn absolutize(path: PathBuf) -> Result<PathBuf> {
    let mut iter = path.iter();
    let mut path = PathBuf::from(iter.next().unwrap());

    for dir in iter {
        match dir.to_str() {
            Some(".") => continue,
            Some("..") => {
                if !path.pop() {
                    return Err(format!("path '{path:?}' does not exist").into());
                }
            }
            Some(d) => path.push(d),
            None => return Err("OsStr::to_str() fail".to_string().into()),
        }
    }

    Ok(path)
}

fn website_path() -> Result<PathBuf> {
    let mut website = std::env::current_dir()?;
    website.push("website");
    Ok(website)
}

#[cfg(test)]
mod tests {
    use super::{get_path, website_path};
    use crate::http::StartLine;
    use std::path::PathBuf;

    fn index() -> PathBuf {
        let mut website = website_path().unwrap();
        website.push("index.html");
        website
    }

    fn website() -> String {
        website_path().unwrap().to_str().unwrap().to_string()
    }

    #[test]
    fn path_works() {
        let startline = StartLine::testpath("/");
        assert_eq!(get_path(&startline).unwrap(), index());

        let startline = StartLine::testpath("/index.html");
        assert_eq!(get_path(&startline).unwrap(), index());

        let startline = StartLine::testpath("/img/img.jpg");
        let path: PathBuf = [website().as_str(), "img", "img.jpg"].iter().collect();
        assert_eq!(get_path(&startline).unwrap(), path);
    }

    #[test]
    fn path_cannot_escape_website_directory() {
        let startline = StartLine::testpath("/../forbidden.html");
        assert_eq!(get_path(&startline).unwrap(), index());
    }
}
