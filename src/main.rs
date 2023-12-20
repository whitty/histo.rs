use clap::{Parser, ArgAction::Append};

/// Quick and dirty analyzer for file generating histograms from
/// log files and similar text-files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Options {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, default_value_t=40, global=true)]
    height: usize,

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

    match &args.command {
        Commands::Simple(_) => {
            let data = histo::data::simple_load(args.input);
            let g = histo::graph::Histogram::new_it(&mut data.iter().map(|(x,v)| (*v, x.to_string())))
                .set_auto_geometry(args.height).draw();
            println!("{}", g);
        }
    }
}

// Claps' built-in self test
#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Options::command().debug_assert()
}
