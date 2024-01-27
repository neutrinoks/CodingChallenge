//! Encpsulation of HTTP relevant implementations.

use std::{error, fmt, io};

/// Clinical trail.
pub trait StreamRead<R: io::BufRead> {
    fn from_stream(stream: &mut R) -> Result<Self, io::Error>
    where
        Self: Sized;
}

/// Definition of a generic HTTP version. This also reflects the availability of different
/// implementational stages in this crate.
#[derive(Clone, Debug)]
pub enum Version {
    Html11,
    Html20,
    Html30,
}

impl TryFrom<&str> for Version {
    type Error = io::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(match s {
            "HTTP/1.1" => Version::Html11,
            "HTTP/2" => Version::Html20,
            "HTTP/3" => Version::Html30,
            _ => {
                let msg = format!("unexpected content: {s}");
                return Err(string_to_invalid_data_err(msg));
            }
        })
    }
}

#[derive(Clone)]
pub struct Message {
    pub startline: StartLine,
    pub content: MessageContent,
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "Message {{\n    {:?}\n    {:?}\n}}",
            self.startline, self.content
        )
    }
}

impl<R: io::BufRead> StreamRead<R> for Message {
    fn from_stream(stream: &mut R) -> Result<Message, io::Error> {
        let startline = StartLine::from_stream(stream)?;

        let content = match startline.version {
            Version::Html11 => {
                let mut content = Vec::<String>::new();
                loop {
                    let mut line = String::new();
                    stream.read_line(&mut line)?;
                    if line.ends_with('\n') {
                        line.pop();
                    }
                    if line.ends_with('\r') {
                        line.pop();
                    }
                    if line.is_empty() {
                        break;
                    } else {
                        content.push(line);
                    }
                }
                MessageContent::Http11(content)
            }
            _ => unimplemented!(),
        };

        Ok(Message { startline, content })
    }
}

#[derive(Debug, Clone)]
pub struct StartLine {
    pub method: Method,
    pub target: String,
    pub version: Version,
}

impl<R: io::BufRead> StreamRead<R> for StartLine {
    fn from_stream(stream: &mut R) -> Result<StartLine, io::Error> {
        let eof_err = |msg| io::Error::new(io::ErrorKind::UnexpectedEof, msg);

        let mut line = String::new();
        stream.read_line(&mut line)?;
        if line.ends_with('\n') {
            line.pop();
        }
        if line.ends_with('\r') {
            line.pop();
        }
        let mut parts = line.split(' ');

        let method = Method::try_from(
            parts
                .next()
                .ok_or(eof_err("could not interpret HTTP method"))?,
        )?;
        let target = String::from(
            parts
                .next()
                .ok_or(eof_err("could not interpret HTTP request-target"))?,
        );
        let version = Version::try_from(
            parts
                .next()
                .ok_or(eof_err("could not interpret HTTP version"))?,
        )?;

        Ok(StartLine {
            method,
            target,
            version,
        })
    }
}

#[derive(Clone, Debug)]
pub enum MessageContent {
    Http11(Vec<String>),
    Http20,
    Http30,
}

#[derive(Clone, Debug)]
pub enum Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
}

impl TryFrom<&str> for Method {
    type Error = io::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(match s {
            "GET" => Method::Get,
            "HEAD" => Method::Head,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "DELETE" => Method::Delete,
            "CONNECT" => Method::Connect,
            "OTIONS" => Method::Options,
            "TRACE" => Method::Trace,
            _ => {
                let msg = format!("unexpected content: {s}");
                return Err(string_to_invalid_data_err(msg));
            }
        })
    }
}

fn string_to_invalid_data_err(s: String) -> io::Error {
    let err = Box::<dyn error::Error + Send + Sync>::from(s.as_str());
    io::Error::new(io::ErrorKind::InvalidData, err)
}
