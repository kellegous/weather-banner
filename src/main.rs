use clap::{Parser, Subcommand};
use std::error::Error;
use weather_banner::{render, Data};

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: Command,

    #[clap(long, default_value_t = String::from("data"))]
    data_dir: String,
}

#[derive(Subcommand, Debug)]
enum Command {
    Render(render::Args),
}

impl Command {
    fn execute(&self, data: &Data) -> Result<(), Box<dyn Error>> {
        match self {
            Command::Render(args) => render::execute(data, args),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let data = Data::from(&args.data_dir)?;
    args.command.execute(&data)?;
    Ok(())
}
