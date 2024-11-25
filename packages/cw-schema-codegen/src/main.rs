#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate log;

use clap::{Parser, ValueEnum};
use std::{
    collections::HashSet,
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

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

fn generate_defs<W>(
    output: &mut W,
    language: Language,
    schema: &cw_schema::Schema,
    add_imports: bool,
) -> anyhow::Result<()>
where
    W: io::Write,
{
    let cw_schema::Schema::V1(schema) = schema else {
        bail!("Only schema version 1 is supported")
    };

    schema.definitions.iter().try_for_each(|node| {
        debug!("Processing node: {node:?}");

        match language {
            Language::Rust => cw_schema_codegen::rust::process_node(output, schema, node),
            Language::Typescript => {
                cw_schema_codegen::typescript::process_node(output, schema, node, add_imports)
            }
            Language::Go | Language::Python => todo!(),
        }
    })?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .with_level(log::LevelFilter::Info)
        .env()
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
    let schema: cosmwasm_schema::JsonCwApi = serde_json::from_str(&schema)?;

    let mut output = opts.output()?;

    if let Some(ref instantiate) = schema.instantiate {
        generate_defs(&mut output, opts.language, instantiate, true)?;
    }

    if let Some(ref execute) = schema.execute {
        generate_defs(&mut output, opts.language, execute, false)?;
    }

    if let Some(ref query) = schema.query {
        generate_defs(&mut output, opts.language, query, false)?;
    }

    if let Some(ref migrate) = schema.migrate {
        generate_defs(&mut output, opts.language, migrate, false)?;
    }

    if let Some(ref sudo) = schema.sudo {
        generate_defs(&mut output, opts.language, sudo, false)?;
    }

    if let Some(ref responses) = schema.responses {
        let responses = responses
            .iter()
            .map(|(_name, response)| response)
            .collect::<HashSet<_>>();

        for response in responses {
            generate_defs(&mut output, opts.language, response, false)?;
        }
    }

    Ok(())
}
