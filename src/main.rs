// SPDX-License-Identifier: GPL-3.0-or-later
// (C) Copyright 2023-2024 Greg Whiteley

use clap::{Parser, ArgAction::Append};
use regex::Regex;
use rust_decimal::Decimal;

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

    /// Plot distribution of difference between scoped "in and out" matches.
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

// hmm,... the no data error doesn't print properly - so print it manually
fn no_data_err() -> Result<(), histo_log::error::Error> {
    let err = histo_log::Error::no_data();
    println!("{}", err);
    Err(err)
}

fn print_histo(data: std::collections::BTreeMap<String, i64>, args: &Options) -> Result<(), histo_log::error::Error> {
    if data.is_empty() {
        no_data_err()?;
    }
    let g = histo_log::graph::Histogram::new_it(&mut data.into_iter())
        .set_auto_geometry(args.height).draw();
    println!("{}", g);
    Ok(())
}

fn print_time_histo(data: std::collections::BTreeMap<Decimal, i64>, args: &Options) -> Result<(), histo_log::error::Error> {
    if data.is_empty() {
        no_data_err()?;
    }
    let g = histo_log::graph::Histogram::new_it(&mut data.into_iter().map(|(v,c)| (v.to_string(), c) ))
        .set_auto_geometry(args.height).draw();
    println!("{}", g);
    Ok(())
}

fn handle_time_buckets(data: Vec<Decimal>, args: &Options) -> Result<(), histo_log::error::Error> {
    if data.is_empty() {
        no_data_err()?;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Options::parse();

    match &args.command {
        Commands::Simple(a) => {
            let data = histo_log::data::simple_load_w_filter(args.input.clone(), &a.optional_match.match_);
            print_histo(data, &args)?;
        },
        Commands::Select(a) => {
            let data = histo_log::data::select_load(args.input.clone(), &a.selector);
            print_histo(data, &args)?;
        },
        Commands::TimeDiff(a) => {
            let data = histo_log::data::time_diff_load(args.input.clone(), &a.time_selector.time_select, &a.optional_match.match_);
            handle_time_buckets(data, &args)?;
        }
        Commands::Scoped(a) => {
            let data = histo_log::data::scoped_time_load(args.input.clone(), &a.time_selector.time_select,
                                                     a.selections.scope_in.as_ref().expect("Must exist --scope-match not yet implemented"),
                                                     a.selections.scope_out.as_ref().expect("Must exist --scope-match not yet implemented"));
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
