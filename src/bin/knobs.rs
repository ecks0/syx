use knobs::Result;
use knobs::cli::Cli;
use knobs::policy::Policy;
use knobs::table;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let cli = Cli::from_args(&args)?;
    let policy = Policy::from_cli(&cli);
    policy.apply();
    table::print();
    Ok(())
}
