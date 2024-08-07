// SPDX-License-Identifier: GPL-3.0-or-later
// (C) Copyright 2023-2024 Greg Whiteley

use clap::{Parser, ArgAction::Append};
use histo_log::Error;
use regex::Regex;
use rust_decimal::Decimal;

/// Quick and dirty analyzer for file generating histograms from
/// log files and similar text-files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Options {
    #[command(subcommand)]
    command: Commands,

    /// Graph width in chars
    #[arg(long, global=true)]
    width: Option<usize>,

    /// Include frequencies (counts)
    #[arg(long, default_value_t=false, global=true)]
    show_counts: bool,

    /// Input file(s), or if omitted use stdin.
    ///
    /// Use '-' for stdin
    #[arg(action = Append, global=true)]
    input: Vec<String>,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Simple histogram of frequencies
    Simple(Simple),
    /// Simple histogram of data selected (extracted) by regex.
    Select(Select),
    /// Plot distribution of difference betewen adjacent time stamps.
    TimeDiff(TimeDiff),

    /// Plot distribution of difference between scoped "in and out" matches, matching regex-match fields for in and out matches.  For example open() -> n can match close(n)
    Scoped(Scoped),
}

#[derive(clap::Args, Debug)]
struct OptionalMatchArgs {
    /// Optional regex to match values - ie filter out values that
    /// don't match
    #[arg(long = "match", value_parser = regexp)]
    match_: Option<Regex>,
}

#[derive(clap::Args, Debug)]
struct Simple {
    #[command(flatten)]
    optional_match: OptionalMatchArgs,
}

/// Simple histogram of data selected (extracted) by regex.
///
/// If a line doesn't match it is dropped from the histogram
#[derive(clap::Args, Debug)]
struct Select {
    /// regex to select value to plot.
    ///
    /// Must include a capture - ie parens () to extract the time
    /// field.
    ///
    /// If there are multiple captures the first will be used
    /// unless one is named "select".
    #[arg(value_parser = regexp_with_one_match)]
    selector: Regex,
}

// Common implementation shared via flatten
#[derive(clap::Args, Debug)]
struct TimeSelector {
    /// Optional regex to extract time values for comparison.
    /// Currently only supports decimal numbers for times.
    ///
    /// Must include a capture - ie parens () to extract the time
    /// field.
    ///
    /// If there are multiple captures the first will be used
    /// unless one is named "time".
    ///
    ///  - eg "(.*) (?<time>\d+\.\d+)$"
    #[arg(long, value_name="regexp", value_parser = regexp_with_one_match, default_value=r"^(\d+\.\d+)")]
    time_select: Regex,

    /// Divide time series up by buckets of this length
    #[arg(long, value_parser=parse_decimal)]
    time_delta: Option<Decimal>,
}

#[derive(clap::Args, Debug)]
struct TimeDiff {
    #[command(flatten)]
    time_selector: TimeSelector,

    #[command(flatten)]
    optional_match: OptionalMatchArgs,
}

#[derive(clap::Args, Debug)]
struct Scoped {
    #[command(flatten)]
    selections: ScopedSelections,

    #[command(flatten)]
    time_selector: TimeSelector,
}

#[derive(clap::Args, Debug)]
#[command(group(clap::ArgGroup::new("one_required")
                .multiple(false)
                .required(true)
                .args(["scope_in", "scope_match"])))]
struct ScopedSelections {

    /// Regex to match for in and out entries in order to determine start/end time.
    ///
    /// TODO - work out what's going on
    #[arg(short = 'm', long, value_name="regexp", value_parser = regexp, conflicts_with_all(["scope_in", "scope_out"]))]
    scope_match: Option<Regex>,

    /// Regex to match in entries in order to determine start time.
    #[arg(short = 'i', long, value_name="regexp", value_parser = regexp, requires("scope_out"))]
    scope_in: Option<Regex>,

    /// Regex to match out entries in order to determine end time.
    #[arg(short = 'o', long, value_name="regexp", value_parser = regexp, requires("scope_in"))]
    scope_out: Option<Regex>,
}

fn regexp_with_one_match(s: &str) -> Result<Regex, String> {
    let re = regexp(s)?;
    // captures_len == 1 for the implicit "all" capture, > 1 for one match
    if re.captures_len() <= 1 {
        return Err(String::from("Need at least one regex match"));
    }
    Ok(re)
}

fn regexp(s: &str) -> Result<Regex, String> {
    let re = Regex::new(s)
        .map_err(|e| e.to_string())?;
    Ok(re)
}

fn parse_decimal(s: &str) -> Result<Decimal, String> {
    use std::str::FromStr;
    if let Ok(d) = Decimal::from_str(s) {
        return Ok(d)
    }
    Err(format!("Failed to parse {} as decimal", s))
}

fn print_histo(data: std::collections::BTreeMap<String, i64>, args: &Options) -> Result<(), Error> {
    if data.is_empty() {
        return Err(Error::no_data());
    }
    let g = histo_log::graph::Histogram::new_it(&mut data.into_iter())
        .set_show_counts(args.show_counts)
        .set_opt_width(args.width).draw()?;
    println!("{}", g);
    Ok(())
}

fn print_time_histo(data: std::collections::BTreeMap<Decimal, i64>, args: &Options) -> Result<(), Error> {
    if data.is_empty() {
        return Err(Error::no_data());
    }
    let g = histo_log::graph::Histogram::new_it(&mut data.into_iter().map(|(v,c)| (v.to_string(), c) ))
        .set_show_counts(args.show_counts)
        .set_opt_width(args.width).draw()?;
    println!("{}", g);
    Ok(())
}

fn handle_time_buckets(data: Vec<Decimal>, args: &Options) -> Result<(), Error> {
    if data.is_empty() {
        return Err(Error::no_data());
    }

    let time_delta = match &args.command {
        Commands::Simple(_) | Commands::Select(_) => { None }
        Commands::TimeDiff(a) => { a.time_selector.time_delta }
        Commands::Scoped(a) => { a.time_selector.time_delta }
    };

    let data = histo_log::graph::Buckets::default()
        .set_delta_opt(time_delta)
        .analyse(&data)
        .generate(&data);
    print_time_histo(data, args)
}

// parse options, and split off the input which we will consume just once
// to avoid having to clone
fn parse_options() -> (Options, Vec<String>) {
    let mut args = Options::parse();
    let input = args.input.split_off(0);
    (args, input)
}

// hmm,... the errors returned through main don't print via display - so print it manually
fn main() -> Result<(), Box<dyn std::error::Error>> {
    run().map_err(|e| { println!("{}\n", e); e} )?;
    Ok(())
}

fn run() -> Result<(), Error> {
    let (args, input) = parse_options();

    match &args.command {
        Commands::Simple(a) => {
            let data = histo_log::data::simple_load_w_filter(input, &a.optional_match.match_);
            print_histo(data, &args)?;
        },
        Commands::Select(a) => {
            let data = histo_log::data::select_load(input, &a.selector);
            print_histo(data, &args)?;
        },
        Commands::TimeDiff(a) => {
            let data = histo_log::data::time_diff_load(input, &a.time_selector.time_select, &a.optional_match.match_);
            handle_time_buckets(data, &args)?;
        }
        Commands::Scoped(a) => {
            let in_match = a.selections.scope_in.as_ref().or(a.selections.scope_match.as_ref());
            let out_match = a.selections.scope_out.as_ref().or(a.selections.scope_match.as_ref());
            let data = histo_log::data::scoped_match_time_load(
                input, &a.time_selector.time_select,
                in_match.expect("Must have either --scope-match or --scope-in"),
                out_match.expect("Must have either --scope-match or --scope-out"))?;
            handle_time_buckets(data, &args)?;
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
