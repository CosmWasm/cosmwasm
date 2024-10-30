#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate log;

use clap::{Parser, ValueEnum};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, ValueEnum)]
pub enum Language {
    #[default]
    Rust,
    Go,
    Python,
    Typescript,
}

#[derive(Parser)]
#[clap(about, author, version)]
/// Official CosmWasm codegen tool
struct Opts {
    #[clap(default_value_t, long, short, value_enum)]
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
    info!(
        "Generating code for {:?} from {:?}",
        opts.language, opts.file
    );

    ensure!(opts.file.exists(), "Schema file does not exist");
    ensure!(
        matches!(
            opts.language,
            Language::Rust | Language::Go | Language::Typescript
        ),
        "Only Rust, TypeScript, and Go code generation is supported at the moment"
    );

    let schema = fs::read_to_string(&opts.file)?;
    let schema: cw_schema::Schema = serde_json::from_str(&schema)?;
    let cw_schema::Schema::V1(schema) = schema else {
        bail!("Unsupported schema version");
    };

    let mut output = opts.output()?;

    schema.definitions.iter().try_for_each(|node| {
        debug!("Processing node: {node:?}");
        cw_schema_codegen::rust::process_node(&mut output, &schema, node)
    })?;

    Ok(())
}
