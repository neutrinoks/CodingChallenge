//! Encapsules command line interface related implementations.

use clap::Parser;

#[derive(Debug)]
pub enum CtDirective {
    /// Compress text file from given filename and optional fixed output name.
    Pack(String, Option<String>),
    /// Decompress binary file from given filename.
    Unpack(String),
}

impl CtDirective {
    /// Default method to process user input from command line. Method checks whether stdin was used to
    /// path a text to be analyzed or a filename was passed to be read in.
    pub fn parse_input() -> crate::Result<CtDirective> {
        let args = CtArgs::parse();
        CtDirective::try_from(args).map_err(|e| e.into())
    }
}

impl TryFrom<CtArgs> for CtDirective {
    type Error = std::io::Error;

    fn try_from(args: CtArgs) -> Result<CtDirective, Self::Error> {
        if args.pack.is_some() == args.unpack.is_some() {
            let err = std::io::Error::new(
                std::io::ErrorKind::Other,
                "argument error: specifiy either 'pack' or 'unpack'",
            );
            Err(err)
        } else if args.pack.is_some() {
            Ok(CtDirective::Pack(args.pack.unwrap(), args.of))
        } else {
            Ok(CtDirective::Unpack(args.unpack.unwrap()))
        }
    }
}

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct CtArgs {
    /// Name of file to be compressed (packed).
    #[clap(long, action)]
    pub pack: Option<String>,
    /// Name of file to be decompressed (unpacked).
    #[clap(long, action)]
    pub unpack: Option<String>,
    /// Optional fixed output filename, after decompressing a compressed file.
    #[clap(long, action)]
    pub of: Option<String>,
}
