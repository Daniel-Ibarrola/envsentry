use clap::Parser;
use envsentry::run;
use std::path::Path;

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

    let env_file = Path::new(&args.env_file);
    if !env_file.exists() && !env_file.is_file() {
        println!("The env file does not exist or is not a file");
        return;
    }

    let src_dir = Path::new(&args.src_dir);
    if !src_dir.exists() && !src_dir.is_dir() {
        println!("The src dir does not exist or is not a directory");
        return;
    }

    println!("Running envsentry...");
    run(env_file, src_dir);
}
