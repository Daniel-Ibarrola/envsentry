use clap::Parser;
use envsentry::run;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The path to the environment file containing all the env variables
    /// that will be checked for
    #[arg(short, long)]
    env_file: String,

    /// The path to the directory containing the source code which will be
    /// checked for missing environment variables
    #[arg(short, long)]
    src_dir: String,
}

fn main() {
    let args = Args::parse();
    println!("Env Sentry");
    run(args.env_file, args.src_dir);
}