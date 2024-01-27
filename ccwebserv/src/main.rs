//! Main executable is just using the library's implementation.

fn main() -> ccwebserv::Result<()> {
    ccwebserv::run_web_server()
}
