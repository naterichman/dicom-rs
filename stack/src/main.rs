use clap::Parser;
use std::path::PathBuf;
use snafu::{Report, Whatever};
use dicom_object::{open_file, DefaultDicomObject};
use dicom_stack::process_dicoms;

#[derive(Debug, Parser)]
#[command(version)]
struct Cli {
    /// The input DICOM file(s)
    #[clap(required = true)]
    files: Vec<PathBuf>,

    /// Output path
    #[clap(long = "out", short = 'o')]
    out: Option<PathBuf>

}

fn main() {
    run().unwrap_or_else(|e| {
        eprintln!("{}", Report::from_error(e));
        std::process::exit(-2);
    });
}

fn run() -> Result<(), Whatever> {
    let Cli {
        files,
        out
    } = Cli::parse();
    println!("hello");
    let out_dcm = process_dicoms(files);
    println!("{:?}", out_dcm);
    Ok(())
}
