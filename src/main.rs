use std::io;
use std::process::{Command, ExitStatus};

use clap::Parser;

#[derive(Parser)]
struct Arguments {
	/// Batch size.
	#[arg(short = 's', long, default_value_t = 1)]
	pub size: usize,

	/// Do not skip blanks.
	#[arg(short = 'b', long, default_value_t = false)]
	pub blanks: bool,

	/// Trim data.
	#[arg(short = 't', long, default_value_t = false)]
	pub trim: bool,

	/// Delimiter used while joining data.
	#[arg(short = 'j', long, default_value = ",")]
	pub join_delimiter: String,

	/// Abort on error.
	#[arg(short = 'a', long, default_value_t = false)]
	pub abort_on_error: bool,

	/// Use tag while processing command arguments.
	#[arg(short = 'g', long, default_value = "{}")]
	pub tag: String,

	/// Command to pass the data to.
	#[arg(trailing_var_arg = true)]
	pub command_args: Vec<String>,
}

struct DynReader {
	reader: Box<dyn io::BufRead>,
}

impl DynReader {
	pub fn from_reader(reader: Box<dyn io::BufRead>) -> Self {
		DynReader { reader }
	}

	pub fn read_line(&mut self) -> Result<Option<String>, io::Error> {
		let mut buffer = String::new();
		match self.reader.read_line(&mut buffer) {
			Ok(0) => Ok(None),
			Ok(_) => Ok(Some(buffer)),
			Err(e) => Err(e),
		}
	}
}

struct Batcher {
	reader: DynReader,
	size: usize,
	skip_blanks: bool,
	trim: bool,
}

impl Batcher {
	pub fn new(reader: DynReader, size: usize, skip_blanks: bool, trim: bool) -> Self {
		Batcher {
			reader,
			size,
			skip_blanks,
			trim,
		}
	}

	pub fn next(&mut self) -> Result<Option<Vec<String>>, io::Error> {
		let mut batch = Vec::with_capacity(self.size);

		while batch.len() < self.size {
			if let Some(line) = self.reader.read_line()? {
				let line = if self.trim {
					line.trim()
				} else {
					line.trim_matches('\n').trim_matches('\r')
				};

				if self.skip_blanks && line.is_empty() {
					continue;
				}

				batch.push(line.to_owned());
			} else {
				break;
			}
		}

		if batch.is_empty() {
			Ok(None)
		} else {
			Ok(Some(batch))
		}
	}
}

pub enum ExecutorError {
	Invalid,
	Missing,
}

struct Executor {
	program_name: String,
	arguments: Vec<String>,
	tag: String,
}

impl Executor {
	pub fn from_command<Tt>(command: &mut Vec<String>, tag: Tt) -> Result<Self, ExecutorError>
	where
		Tt: ToString,
	{
		if command.is_empty() {
			return Err(ExecutorError::Missing);
		}
		let program_name = command.remove(0);
		if program_name.is_empty() {
			return Err(ExecutorError::Invalid);
		}

		Ok(Executor {
			program_name,
			arguments: command.to_vec(),
			tag: tag.to_string(),
		})
	}

	pub fn execute(&self, batch: &str) -> io::Result<ExitStatus> {
		let prepared_args: Vec<String> = self
			.arguments
			.iter()
			.map(|arg| arg.replace(self.tag.as_str(), batch))
			.collect();

		Command::new(self.program_name.as_str())
			.args(prepared_args.into_iter())
			.status()
	}
}

fn main() {
	let mut args = Arguments::parse();

	let executor = match Executor::from_command(&mut args.command_args, &args.tag) {
		Ok(executor) => executor,
		Err(ExecutorError::Invalid) => {
			eprintln!("Invalid command.");
			std::process::exit(exitcode::USAGE);
		},
		Err(ExecutorError::Missing) => {
			eprintln!("Missing command.");
			std::process::exit(exitcode::USAGE);
		},
	};

	let reader = DynReader::from_reader(Box::new(io::stdin().lock()));
	let mut batcher = Batcher::new(reader, args.size, !args.blanks, args.trim);
	while let Ok(Some(batch)) = batcher.next() {
		if executor
			.execute(batch.join(args.join_delimiter.as_str()).as_str())
			.is_err()
		{
			std::process::exit(exitcode::UNAVAILABLE);
		}
	}
}
