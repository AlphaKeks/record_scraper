use gokz_rs::global_api::get_record;
use std::{io::Write, process, time::Duration};

const SLEEP_TIME: Duration = Duration::from_millis(727);

enum UserInput {
	ID(u32),
	Path(String),
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
	let start_id = match get_input("Which ID do you want to start at?", true) {
		UserInput::ID(id) => id,
		UserInput::Path(_) => unreachable!(),
	};

	let count = match get_input("How many records do you want to fetch?", true) {
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
				std::io::ErrorKind::NotFound => match std::fs::File::create(&output_file) {
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

	// why can I not do this bro fuck you
	// let range: std::ops::Range<u32> = match count {
	// 	0 => start_id..,
	// 	n => start_id..n,
	// };

	for i in start_id.. {
		let record = match get_record(&i, &client).await {
			Ok(record) => record,
			Err(_) => {
				println!("Reached most recent record (#{}). Sleeping for 5 minutes.", &i);
				std::thread::sleep(Duration::from_secs(60 * 5));
				continue;
			},
		};

		write_to_file(record, &mut output_file);

		if count != 0 && i - start_id == count {
			break;
		}

		// sleep to avoid rate limiting
		std::thread::sleep(SLEEP_TIME);
	}
}

/// Gets input from stdin and parses it if needed.
fn get_input<'a>(msg: &'a str, is_number: bool) -> UserInput {
	println!("{}", msg);
	let mut input = String::new();
	std::io::stdin().read_line(&mut input).expect("Failed to read line. PogO");
	let input = input.trim();

	if is_number {
		let id = match input.parse::<u32>() {
			Ok(id) => id,
			Err(why) => {
				println!(
					"`{}` is not a valid input. Please input a positive 32-bit integer.\n{:?}",
					input, why
				);
				process::exit(1);
			},
		};

		return UserInput::ID(id);
	} else {
		return UserInput::Path(input.to_owned());
	}
}

fn write_to_file(
	record: gokz_rs::global_api::records::top::Response,
	output_file: &mut std::fs::File,
) {
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
