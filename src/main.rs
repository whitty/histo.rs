use asciigraph::Graph;
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

    let mut g1 = Graph::default();

    g1.set_1d_data(vec![0, 1, 1, 0, 2, 0, 1, 2, 0, 0, 0, 1, 0, 1000])
    //    .set_y_min(0)
    //    .set_y_max(4)
    //    .set_plot_height(20)
        .set_block_width(3)
    //    .set_y_label_margin(1)
    //    .set_title(String::from("HEllo 23123123"))
        .set_paddings([1;4])
        .set_big_title(true)
        .set_x_axis_label(String::from("x_axis_label\nxz"))
        .set_y_axis_label(String::from("y_axis_label\nyy"));
    let x = g1.draw();

    println!("{}", x);
}

// Claps' built-in self test
#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Options::command().debug_assert()
}
