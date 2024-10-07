#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate log;

use clap::{Parser, ValueEnum};
use std::{
    fs::File,
    io::{self, Write},
    path::PathBuf,
};
use strum::Display;

#[derive(Clone, Copy, Default, Display, ValueEnum)]
#[strum(serialize_all = "kebab-case")]
pub enum Language {
    #[default]
    Rust,
    Go,
    Typescript,
}

#[derive(Parser)]
#[clap(about, author, version)]
/// Official CosmWasm codegen tool
struct Opts {
    #[clap(default_value_t, long, short)]
    /// Programming language to generate code for
    language: Language,

    #[clap(long, short)]
    /// Path to the schema file
    file: PathBuf,

    #[clap(long, short)]
    /// Path to the output file
    output: Option<PathBuf>,
}

impl Opts {
    fn output(&self) -> anyhow::Result<impl Write> {
        let io_out = if let Some(ref path) = self.output {
            either::Left(File::create(path)?)
        } else {
            either::Right(io::stdout().lock())
        };

        Ok(io_out)
    }
}

fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()?;

    let opts: Opts = Opts::parse();
    info!("Generating code for {} from {:?}", opts.language, opts.file);

    let schema = std::fs::read_to_string(&opts.file)?;
    let schema: cw_schema::Schema = serde_json::from_str(&schema)?;
    let cw_schema::Schema::V1(schema) = schema else {
        bail!("Unsupported schema version");
    };

    Ok(())
}
