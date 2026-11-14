use crate::error::Result;
use crate::print::{PageSize, PrintFormat};
use clap::Args;

/// Print a file to PDF or Word (A4 & US Letter).
#[derive(Args, Debug)]
pub struct PrintArgs {
    /// File to print (markdown or plain text)
    input: String,
    /// Output format: pdf | word
    #[arg(short = 'f', long = "format", default_value = "pdf")]
    format: String,
    /// Page size: a4 | letter
    #[arg(short = 's', long = "size", default_value = "a4")]
    size: String,
    /// Output file path
    #[arg(short = 'o', long = "out")]
    out: Option<String>,
}

pub fn handle(args: PrintArgs) -> Result<()> {
    let format = PrintFormat::from_str(&args.format)?;
    let size = PageSize::from_str(&args.size)?;
    let input = std::path::Path::new(&args.input);
    let out = args.out.as_deref().map(std::path::Path::new);
    crate::print::print_file(input, format, size, out)
}
