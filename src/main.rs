use gokz_rs::global_api::{get_record, records::top::Response as RecordResponse};
use std::{
	fs::File,
	io::{self, Write},
	process,
	time::Duration,
};

// delay between each request
const SLEEP_TIME: Duration = Duration::from_millis(727);

enum UserInput {
	ID(usize),
	Path(String),
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
	let start_id: usize = match get_input("Which ID do you want to start at?", true) {
		UserInput::ID(id) => id,
		UserInput::Path(_) => unreachable!(),
	};

	let count: usize = match get_input("How many records do you want to fetch?", true) {
		UserInput::ID(count) => count,
		UserInput::Path(_) => unreachable!(),
	};

	let output_file = match get_input("Please specify an output file.", false) {
		UserInput::Path(path) => path,
		UserInput::ID(_) => unreachable!(),
	};

	let mut output_file =
		match std::fs::OpenOptions::new().write(true).append(true).open(&output_file) {
			// file was found
			Ok(file) => file,
			// file was not found
			Err(why) => match why.kind() {
				// try to create the file instead
				io::ErrorKind::NotFound => match File::create(&output_file) {
					Ok(file) => {
						println!("Successfully created `{}`.", &output_file);
						file
					},
					Err(why) => {
						println!(
							"{} was not found and also failed to be created.\n{:?}",
							&output_file, why
						);
						process::exit(1);
					},
				},
				why => {
					println!("{} is not a valid file.\n{:?}", output_file, why);
					process::exit(1);
				},
			},
		};

	let client = reqwest::Client::new();

	let range = match count {
		0 => start_id..usize::MAX, // not truly an infinite loop, but should be enough anyway
		n => start_id..(start_id + n),
	};

	for i in range {
		let record = match get_record(&(i as u32), &client).await {
			Ok(record) => record,
			Err(_) => {
				println!("Record #{} not found. Sleeping for 5 minutes.", &i);
				std::thread::sleep(Duration::from_secs(60 * 5));
				continue;
			},
		};

		write_to_file(record, &mut output_file);

		// sleep to avoid rate limiting
		std::thread::sleep(SLEEP_TIME);
	}
}

// Gets input from stdin and parses it if needed.
fn get_input<'a>(msg: &'a str, is_number: bool) -> UserInput {
	println!("{}", msg);
	let mut input = String::new();
	if let Err(why) = io::stdin().read_line(&mut input) {
		println!("Failed to read from stdin. PogO\n{:?}", why);
		process::exit(1);
	}
	let input = input.trim();

	if is_number {
		let Ok(id) = input.parse::<usize>() else {
			println!(
				"`{}` is not a valid input. Please input a positive integer.",
				input
			);
			process::exit(1);
		};

		return UserInput::ID(id);
	} else {
		return UserInput::Path(input.to_owned());
	}
}

fn write_to_file(record: RecordResponse, output_file: &mut File) {
	println!("Writing record #{} to disk...", record.id);

	let json = match serde_json::to_string(&record) {
		Ok(json) => json,
		Err(why) => {
			return println!("Failed to deserialize record #{}.\n{:?}", record.id, why);
		},
	};

	match writeln!(output_file, "{}", json) {
		Ok(_) => println!("Done."),
		Err(why) => println!("Failed to write record #{} to disk.\n{:?}", record.id, why),
	}
}
