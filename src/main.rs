use clap::{Parser, ArgAction::Append};
use regex::Regex;

/// Quick and dirty analyzer for file generating histograms from
/// log files and similar text-files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Options {
    #[command(subcommand)]
    command: Commands,

    /// Graph height in lines
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
    TimeDiff(TimeDiff),
}

#[derive(clap::Args, Debug)]
struct Simple {
    /// Optional regex to match values
    #[arg(long = "match")]
    match_: Option<String>,
}

#[derive(clap::Args, Debug)]
struct TimeDiff {
    /// Optional regex to extract time values for comparison
    ///
    /// Must include a capture - ie parens () to extract the time
    /// field.
    ///
    /// If there are multiple captures the first will be used
    /// unless one is named "time"
    ///
    ///  - eg "(.*) (?<time>\d+\.\d+)$"
    #[arg(long, value_name="regexp", value_parser = regexp_with_one_match, default_value=r"^(\d+\.\d+)")]
    time_select: Regex,
}

fn regexp_with_one_match(s: &str) -> Result<Regex, String> {
    let re = Regex::new(s)
        .map_err(|e| e.to_string())?;
    // TODO check the number of matches
    Ok(re)
}

// hmm,... the no data error doesn't print properly - so print it manually
fn no_data_err() -> Result<(), histo::error::Error> {
    let err = histo::Error::no_data();
    println!("{}", err.to_string());
    Err(err)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Options::parse();

    match &args.command {
        Commands::Simple(_) => {
            let data = histo::data::simple_load(args.input);
            if data.is_empty() {
                no_data_err()?;
            }
            let g = histo::graph::Histogram::new_it(&mut data.into_iter())
                .set_auto_geometry(args.height).draw();
            println!("{}", g);
        },
        Commands::TimeDiff(a) => {
            let data = histo::data::time_diff_load(args.input, &a.time_select);
            if data.is_empty() {
                no_data_err()?;
            }
            let data = histo::graph::Buckets::default()
                .analyse(&data)
                .generate(&data);
            let g = histo::graph::Histogram::new_it(&mut data.into_iter())
                .set_auto_geometry(args.height).draw();
            println!("{}", g);
        }
    }

    Ok(())
}

// Claps' built-in self test
#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Options::command().debug_assert()
}
