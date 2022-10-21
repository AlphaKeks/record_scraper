use std::io::Write;

enum UserInput {
	ID(u32),
	Path(String),
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Record {
	pub id: Option<i32>,
	pub steamid64: Option<String>,
	pub player_name: Option<String>,
	pub steam_id: Option<String>,
	pub server_id: Option<i32>,
	pub map_id: Option<i32>,
	pub stage: Option<i32>,
	pub mode: Option<String>,
	pub tickrate: Option<i32>,
	pub time: Option<f32>,
	pub teleports: Option<i32>,
	pub created_on: Option<String>,
	pub updated_on: Option<String>,
	pub updated_by: Option<i32>,
	pub record_filter_id: Option<i32>,
	pub server_name: Option<String>,
	pub map_name: Option<String>,
	pub points: Option<i32>,
	pub replay_id: Option<i32>,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
	let (start_fw, start_bw) = (
		get_input("Please input an ID to start at:", true),
		get_input("Please input an ID to go backwards from:", true),
	);

	if let UserInput::Path(output_file) = get_input("/path/to/output.txt", false) {
		let file1 = output_file.clone();
		let file2 = output_file;

		let handles = [
			// spawn thread to fetch new records
			tokio::spawn(async move {
				let delay = std::time::Duration::from_millis(727);
				let sleep_duration = std::time::Duration::from_secs(60 * 5);

				if let UserInput::ID(start_fw) = start_fw {
					fetch_records(file1, start_fw.., delay, sleep_duration).await;
				}
			}),
			// spawn thread to go backwards and fetch older records
			tokio::spawn(async move {
				let delay = std::time::Duration::from_millis(727);
				let sleep_duration = std::time::Duration::from_secs(60 * 5);

				if let UserInput::ID(start_bw) = start_bw {
					fetch_records(file2, (0..start_bw).rev(), delay, sleep_duration).await;
				}
			}),
		];

		for handle in handles {
			let _ = handle.await;
		}
	}
}

fn get_input<'a>(msg: &'a str, parse: bool) -> UserInput {
	println!("{}", msg);
	let mut input = String::new();
	std::io::stdin()
		.read_line(&mut input)
		.expect("Failed to read line. PogO");
	let input = input.trim();

	if parse {
		UserInput::ID(input.parse::<u32>().expect("Please input a valid ID."))
	} else {
		UserInput::Path(input.to_owned())
	}
}

fn write_to_file(file: &mut std::fs::File, entry: Record, idx: &u32) {
	let json = match serde_json::to_string(&entry) {
		Ok(json) => json,
		Err(why) => {
			println!("Failed to deserialize record #{idx}\n{:#?}", why);
			return;
		}
	};

	match writeln!(file, "{}", json) {
		Ok(_) => println!("wrote record #{idx} to disk."),
		Err(why) => println!("failed to write record #{idx} to disk.\n{:#?}", why),
	}
}

async fn fetch_records<T>(
	output_file: String,
	range: T,
	delay: std::time::Duration,          // delay between each request
	sleep_duration: std::time::Duration, // time to wait if fetching a record was unsuccessful
) where
	T: Iterator<Item = u32>,
{
	let client = reqwest::Client::new();
	let mut file = match std::fs::OpenOptions::new()
		.write(true)
		.append(true)
		.open(&output_file) // try to open file as writable
	{
		Ok(file) => file,
		Err(why) => match why.kind() {
			// create file if it doesn't exist
			std::io::ErrorKind::NotFound => match std::fs::File::create(output_file) {
				Ok(file) => file,
				Err(why) => panic!("Failed to create file: {:#?}", why),
			},
			kind => panic!("Failed to open file: {:#?}\n{:#?}", kind, why),
		},
	};

	// fetch records for given range
	for i in range {
		match get_record(&i, &client).await {
			Ok(record) => write_to_file(&mut file, record, &i),
			Err(_) => {
				println!(
					"no records found; sleeping for {} seconds.",
					&sleep_duration.as_secs()
				);
				// no records found anymore, sleep for x seconds
				tokio::time::sleep(sleep_duration).await;
				continue;
			}
		}

		// delay after each request to not get rate limited
		tokio::time::sleep(delay).await;
	}
}

async fn get_record(id: &u32, client: &reqwest::Client) -> Result<Record, String> {
	match client
		.get(format!("https://kztimerglobal.com/api/v2/records/{id}"))
		.send()
		.await
	{
		Err(why) => Err(why.to_string()),
		Ok(data) => match data.json::<Record>().await {
			Err(why) => Err(why.to_string()),
			Ok(record) => Ok(record),
		},
	}
}
