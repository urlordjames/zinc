use clap::{Parser, Subcommand};
use std::path::{PathBuf, Path};

#[derive(Parser)]
#[clap(author, version)]
#[clap(propagate_version = true)]
struct Args {
	#[clap(subcommand)]
	command: Commands
}

#[derive(Subcommand)]
enum Commands {
	Build {
		input_file: PathBuf,

		#[clap(short)]
		output_file: Option<PathBuf>,
	
		#[clap(long)]
		optimize: bool
	},
	Run {
		input_file: PathBuf,

		#[clap(long)]
		optimize: bool
	},
	Exec {
		code: String,

		#[clap(long)]
		optimize: bool
	},
	RunSafe {
		input_file: PathBuf
	},
	ExecSafe {
		code: String
	}
}

#[cfg(target_os = "windows")]
const DEFAULT_FILE_NAME: &'static str = "bruh.exe";

#[cfg(not(target_os = "windows"))]
const DEFAULT_FILE_NAME: &'static str = "bruh";

fn main() {
	let args = Args::parse();

	match args.command {
		Commands::Build { input_file, output_file, optimize } => {
			match output_file {
				Some(output_path) => {
					zink::build_executable(&input_file, &output_path, optimize);
				},
				None => {
					let output_path = Path::new(DEFAULT_FILE_NAME);
					zink::build_executable(&input_file, output_path, optimize);
				}
			}
		},
		Commands::Run { input_file, optimize } => {
			let code = std::fs::read_to_string(input_file).expect("cannot read file");
			zink::run_jit(&code, optimize);
		},
		Commands::Exec { code, optimize } => {
			zink::run_jit(&code, optimize);
		},
		Commands::RunSafe { input_file } => {
			let code = std::fs::read_to_string(input_file).expect("cannot read file");
			print!("{}", zink::run_interpreter(&code));
		}
		Commands::ExecSafe { code } => {
			print!("{}", zink::run_interpreter(&code));
		}
	}
}
