//! Encpsulation of HTTP relevant implementations.

use std::{error, fmt, io};

/// Module internal macro for default error messages in `TryFrom<&str>` implementations for HTTP-
/// type definitions.
macro_rules! http_tryfrm_err {
    ($s:expr) => {{
        let msg = format!("unexpected content: {}", $s);
        return Err(string_to_invalid_data_err(msg));
    }};
}

/// Definition of a generic HTTP version. This also reflects the availability of different
/// implementational stages in this crate.
#[derive(Clone, Debug)]
pub enum Version {
    Html11,
    // Html20,
    // Html30,
}

impl From<Version> for &'static str {
    fn from(val: Version) -> Self {
        match val {
            Version::Html11 => "HTTP/1.1",
            // Version::Html20 => "HTTP/2",
            // Version::Html30 => "HTTP/3",
        }
    }
}

impl TryFrom<&str> for Version {
    type Error = io::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(match s {
            "HTTP/1.1" => Version::Html11,
            // "HTTP/2" => Version::Html20,
            // "HTTP/3" => Version::Html30,
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
    pub content: Vec<String>,
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

impl TryFrom<&str> for Message {
    type Error = io::Error;

    fn try_from(stream: &str) -> Result<Message, Self::Error> {
        let mut lines = stream.split("\r\n").filter(|p| !p.is_empty());

        let startline = if let Some(startline) = lines.next() {
            StartLine::try_from(startline)?
        } else {
            return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
        };

        let content = match startline.version {
            Version::Html11 => {
                let mut content = Vec::<String>::new();
                for line in lines {
                    content.push(line.to_string());
                }
                content
            }
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

impl TryFrom<&str> for StartLine {
    type Error = io::Error;

    fn try_from(stream: &str) -> Result<StartLine, Self::Error> {
        let eof_err = || io::Error::from(io::ErrorKind::UnexpectedEof);

        let mut parts = stream.split(' ');

        let method = Method::try_from(parts.next().ok_or(eof_err())?)?;
        let target = parts.next().ok_or(eof_err())?.to_string();
        let version = Version::try_from(parts.next().ok_or(eof_err())?)?;

        Ok(StartLine {
            method,
            target,
            version,
        })
    }
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
            _ => http_tryfrm_err!(s),
        })
    }
}

fn string_to_invalid_data_err(s: String) -> io::Error {
    let err = Box::<dyn error::Error + Send + Sync>::from(s.as_str());
    io::Error::new(io::ErrorKind::InvalidData, err)
}

// #[derive(Clone, Debug)]
// pub enum StatusCode {
//     Informational(ScInformational),
//     Successful(ScSuccessful),
//     Redirection,
//     ClientError,
//     ServerError,
// }

#[derive(Clone, Debug)]
pub enum ScInformational {
    Continue,
    SwitchingProtocols,
}

impl TryFrom<&str> for ScInformational {
    type Error = io::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(match s {
            "100" => ScInformational::Continue,
            "101" => ScInformational::SwitchingProtocols,
            _ => http_tryfrm_err!(s),
        })
    }
}

#[derive(Clone, Debug)]
pub enum ScSuccessful {
    Ok,
    Created,
    Accepted,
    NonAuthoritativeContent,
    NoContent,
    ResetContent,
    PartialContent,
    MultiStatus,
    AlreadyReported,
}

impl From<ScSuccessful> for &'static str {
    fn from(val: ScSuccessful) -> Self {
        match val {
            ScSuccessful::Ok => "200 OK",
            ScSuccessful::Created => "201 Created",
            ScSuccessful::Accepted => "202 Accepted",
            ScSuccessful::NonAuthoritativeContent => "203 Non-Authoritative Information",
            ScSuccessful::NoContent => "204 No Content",
            ScSuccessful::ResetContent => "205 Reset Content",
            ScSuccessful::PartialContent => "206 Partial Content",
            ScSuccessful::MultiStatus => "207 Multi-Status",
            ScSuccessful::AlreadyReported => "208 Already Reported",
        }
    }
}

impl TryFrom<&str> for ScSuccessful {
    type Error = io::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(match s {
            "200 OK" => ScSuccessful::Ok,
            "201 Created" => ScSuccessful::Created,
            "202 Accepted" => ScSuccessful::Accepted,
            "203 Non-Authoritative Information" => ScSuccessful::NonAuthoritativeContent,
            "204 No Content" => ScSuccessful::NoContent,
            "205 Reset Content" => ScSuccessful::ResetContent,
            "206 Partial Content" => ScSuccessful::PartialContent,
            "207 Multi-Status" => ScSuccessful::MultiStatus,
            "208 Already Reported" => ScSuccessful::AlreadyReported,
            _ => http_tryfrm_err!(s),
        })
    }
}

#[derive(Clone, Debug)]
pub enum ScClientError {
    // BadRequest,
    // Unauthorized,
    // PaymentRequired,
    NotFound,
    // MethodNotAllowed,
    // NotAcceptable,
    // ProxyAuthenticationRequired,
    // RequestTimeout,
    // Conflict
    // ...
}

impl From<ScClientError> for &'static str {
    fn from(val: ScClientError) -> Self {
        match val {
            ScClientError::NotFound => "404 Not Found",
        }
    }
}
