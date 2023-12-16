//! Encapsules command line interface related implementations.

use clap::Parser;

/// The whole input data for main function (parameters and text to be processed).
#[derive(Debug)]
pub struct CtInput {
    pub content: String,
}

impl CtInput {
    /// Default method to process user input from command line. Method checks whether stdin was used to
    /// path a text to be analyzed or a filename was passed to be read in.
    pub fn parse_input() -> crate::Result<CtInput> {
        // let (args, content) = if io::stdin().is_terminal() {
        //     // No usage of stdin, a filename should be provided.
        //     let args = CtInput::parse();
        //     let content = if let Some(file) = &args.file {
        //         // Check file size and decide for reading in completely or buffered.
        //         Content::read_to_string(file)?
        //     } else {
        //         return Err(String::from("No input file or data was provided").into());
        //     };
        //     (args, content)
        // } else {
        //     // Stdin provides content input, no filename should be provided.
        //     let mut content = String::new();
        //     let mut reader = BufReader::new(io::stdin());
        //     reader.read_to_string(&mut content)?;
        //     // ...
        // };
        // Ok(CcWcInput { args, content })
        let args = CtArgs::parse();
        CtInput::try_from(args).map_err(|e| e.into())
    }
}

impl TryFrom<CtArgs> for CtInput {
    type Error = std::io::Error;

    fn try_from(args: CtArgs) -> Result<CtInput, Self::Error> {
        Ok(CtInput {
            content: std::fs::read_to_string(args.filename)?,
        })
    }
}

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct CtArgs {
    #[clap(long, action)]
    pub filename: String,
}
