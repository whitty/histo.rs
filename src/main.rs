use clap::{Parser, ArgAction::Append};

/// Quick and dirty analyzer for file generating histograms from
/// log files and similar text-files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Options {
    #[command(subcommand)]
    command: Commands,

    /// Input file(s)
    #[arg(action = Append, global=true)]
    input: Vec<String>,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Simple histogram of frequencies
    Simple(Simple),
}

#[derive(clap::Args, Debug)]
struct Simple {
    /// Optional regex to match values
    #[arg(long = "match")]
    match_: Option<String>,
}

fn main() {
    let args = Options::parse();

    println!("{:?}", args);
    match &args.command {
        Commands::Simple(xx) => {
            println!("Hello {:?} match=${:?}", args.input, xx.match_);
            let data = histo::data::simple_load(args.input);
            let h = histo::graph::Histogram::new_it(&mut data.iter().map(|(x,v)| (*v, x.to_string())));
            println!("{}", h.draw());
        }
    }
}

// Claps' built-in self test
#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Options::command().debug_assert()
}
