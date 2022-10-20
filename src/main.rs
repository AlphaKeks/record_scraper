use std::{
	fs::{File, OpenOptions},
	io::{stdin, ErrorKind, Write},
	time::Duration,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
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
	// get starting id
	println!("ID to start with:");
	let mut starting_id = String::new();

	stdin()
		.read_line(&mut starting_id)
		.expect("Failed to read line. PogO");

	let starting_id = starting_id
		.trim()
		.parse::<u128>()
		.expect("Please input a valid ID.");

	println!("ID to resume at:");
	let mut resume_id = String::new();

	stdin()
		.read_line(&mut resume_id)
		.expect("Failed to read line. PogO");

	let resume_id = resume_id
		.trim()
		.parse::<u128>()
		.expect("Please input a valid ID.");

	// get output file
	println!("path/to/output_file.txt:");
	let mut output_file = String::new();

	stdin()
		.read_line(&mut output_file)
		.expect("Failed to read line. PogO");

	let output_file1 = output_file.trim().to_owned();
	let output_file2 = output_file1.clone();

	let handles = [
		// continuously fetching new records
		tokio::spawn(async move {
			let client = reqwest::Client::new();
			let output_file = output_file1;

			let mut file = match OpenOptions::new()
				.write(true)
				.append(true)
				.open(&output_file)
			{
				Ok(file) => file,
				Err(why) => match why.kind() {
					ErrorKind::NotFound => match File::create(output_file) {
						Ok(file) => file,
						Err(why) => panic!("Failed to create file: {:#?}", why),
					},
					_ => panic!("Failed to open file: {:#?}", why),
				},
			};

			for i in starting_id.. {
				match get_record(i, &client).await {
					Err(_) => {
						println!("No new records, going to sleep for 5 minutes.");
						tokio::time::sleep(Duration::from_secs(60 * 5)).await;
						continue;
					}
					Ok(record) => write_to_file(&mut file, &record, i),
				};
			}
		}),
		// going back in time to get older records
		tokio::spawn(async move {
			let client = reqwest::Client::new();
			let output_file = output_file2;

			let mut file = match OpenOptions::new()
				.write(true)
				.append(true)
				.open(&output_file)
			{
				Ok(file) => file,
				Err(why) => match why.kind() {
					ErrorKind::NotFound => match File::create(output_file) {
						Ok(file) => file,
						Err(why) => panic!("Failed to create file: {:#?}", why),
					},
					_ => panic!("Failed to open file: {:#?}", why),
				},
			};

			for i in (0..resume_id).rev() {
				match get_record(i, &client).await {
					Err(why) => println!("Failed to get record #{}:\n{:#?}", i, why),
					Ok(record) => write_to_file(&mut file, &record, i),
				};
				tokio::time::sleep(Duration::from_millis(727)).await;
			}
		}),
	];

	for handle in handles {
		let _ = handle.await;
	}
}

async fn get_record(id: u128, client: &reqwest::Client) -> Result<Record, String> {
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

fn write_to_file(file: &mut File, record: &Record, i: u128) {
	let json = match serde_json::to_string(record) {
		Err(why) => {
			return println!("Failed to deserialize record #{}\n:{:#?}", i, why);
		}
		Ok(json) => json,
	};

	match writeln!(file, "{}", json) {
		Err(why) => println!("Failed to write to file: {:#?}", why),
		Ok(_) => println!("got record #{i}"),
	}
}
