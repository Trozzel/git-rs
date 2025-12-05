use clap::{Parser, Subcommand};
use git_rs::commands;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "git-rs")]
#[command(about = "A Rust implementation of Git", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Git repository
    Init {
        /// Directory to initialize (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Compute object ID and optionally create a blob from a file
    HashObject {
        /// The file to hash
        file: PathBuf,
        /// Actually write the object into the object database
        #[arg(short = 'w', long)]
        write: bool,
    },
    /// Provide content or type and size information for repository objects
    CatFile {
        /// Show object type
        #[arg(short = 't', long, group = "mode")]
        r#type: bool,
        /// Show object size
        #[arg(short = 's', long, group = "mode")]
        size: bool,
        /// Pretty-print object content
        #[arg(short = 'p', long, group = "mode")]
        pretty_print: bool,
        /// The object hash
        object: String,
    },
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => {
            commands::init(&path)?;
        }
        Commands::HashObject { file, write } => {
            let hash = commands::hash_object(&file, write, None)?;
            println!("{}", hash);
        }
        Commands::CatFile {
            r#type,
            size,
            pretty_print,
            object,
        } => {
            if r#type {
                let obj_type = commands::cat_file_type(&object, None)?;
                println!("{}", obj_type);
            } else if size {
                let obj_size = commands::cat_file_size(&object, None)?;
                println!("{}", obj_size);
            } else if pretty_print {
                commands::cat_file(&object, None)?;
            } else {
                eprintln!("Error: must specify one of -t, -s, or -p");
                process::exit(1);
            }
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
