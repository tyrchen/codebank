use anyhow::Result;
use clap::{Parser, ValueEnum};
use codebank::{Bank, BankConfig, BankStrategy, CodeBank};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(
    name = "codebank",
    about = "A tool to generate code banks from source code",
    version
)]
struct Cli {
    input: PathBuf,

    /// Output file for the generated code bank (stdout if not provided)
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// Strategy to use for generating the code bank
    #[clap(short, long, value_enum, default_value_t = OutputStrategy::Default)]
    strategy: OutputStrategy,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum OutputStrategy {
    /// Include all code with full content
    Default,
    /// Include all code except tests
    NoTests,
    /// Include only public interfaces, not full implementations
    Summary,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Create a new code bank generator
    let code_bank = CodeBank::try_new()?;

    // Convert the CLI strategy to BankStrategy
    let strategy = match cli.strategy {
        OutputStrategy::Default => BankStrategy::Default,
        OutputStrategy::NoTests => BankStrategy::NoTests,
        OutputStrategy::Summary => BankStrategy::Summary,
    };

    let config = BankConfig::new(cli.input, strategy, vec![]);

    // Generate the code bank
    let content = code_bank.generate(&config)?;

    // Output to file or stdout
    if let Some(output_file) = cli.output {
        fs::write(&output_file, content)?;
        println!("Code bank written to {}", output_file.display());
    } else {
        println!("{}", content);
    }

    Ok(())
}
