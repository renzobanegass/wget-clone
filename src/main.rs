extern crate clap;

use clap::{Arg, App};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use reqwest::header::{CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE};
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::Path;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Instant;
use console::style as console_style;
use ctrlc;

fn main() {
    let matches = App::new("Rget")
        .version("0.1.0")
        .about("wget clone written in Rust")
        .arg(
            Arg::with_name("URL")
                .required(true)
                .takes_value(true)
                .index(1)
                .help("URL to download"),
        )
        .get_matches();
    let url = matches.value_of("URL").unwrap();

    println!("{}", url);

    let start_time = Instant::now();
    let result = download_with_pause(url, false);
    let duration = start_time.elapsed();

    log_download(url, &result, duration).expect("Failed to log download");

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }
}

fn create_progress_bar(quiet_mode: bool, msg: &str, length: Option<u64>) -> ProgressBar {
    let bar = match quiet_mode {
        true => ProgressBar::hidden(),
        false => match length {
            Some(len) => ProgressBar::new(len),
            None => ProgressBar::new_spinner(),
        },
    };

    bar.set_message(msg.to_string());
    match length.is_some() {
        true => bar.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{msg} {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} eta: {eta}",
                )
                .expect("Failed to set template")
                .progress_chars("=> "),
        ),
        false => bar.set_style(ProgressStyle::default_spinner()),
    };

    bar
}

fn sanitize_filename(filename: &str) -> String {
    filename.chars().filter(|c| c.is_alphanumeric() || *c == '.' || *c == '_').collect()
}

fn download_with_pause(url: &str, quiet_mode: bool) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let original_filename = url.split('/').last().unwrap().to_string();
    let sanitized_filename = sanitize_filename(&original_filename);
    let filepath = Path::new(&sanitized_filename);
    let mut file = if filepath.exists() {
        OpenOptions::new().read(true).write(true).open(filepath)?
    } else {
        File::create(filepath)?
    };

    let downloaded = file.metadata()?.len();
    let range_header = format!("bytes={}-", downloaded);

    let request = client.get(url).header(RANGE, range_header.as_str());
    let mut response = request.send()?;

    if response.status().is_success() || response.status() == 206 {
        print(
            format!(
                "HTTP request sent... {}",
                custom_style(format!("{}", response.status()), "green")
            ),
            quiet_mode,
        );

        let headers = response.headers().clone();
        let total_size = headers
            .get(CONTENT_LENGTH)
            .and_then(|ct_len| ct_len.to_str().ok())
            .and_then(|s| s.parse().ok())
            .or_else(|| {
                headers
                    .get(CONTENT_RANGE)
                    .and_then(|ct_range| {
                        let ct_range = ct_range.to_str().ok()?;
                        let total_size = ct_range.split('/').last()?;
                        total_size.parse().ok()
                    })
            })
            .unwrap_or(0);

        let ct_type = headers
            .get(CONTENT_TYPE)
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("application/octet-stream");

        print(
            format!("Type: {}", custom_style(ct_type.to_string(), "green")),
            quiet_mode,
        );

        let bar = create_progress_bar(quiet_mode, &sanitized_filename, Some(total_size));
        bar.set_position(downloaded);

        let chunk_size = 1024 * 64; // 64 KB chunks

        let mut buffer = vec![0; chunk_size];

        let paused = Arc::new(AtomicBool::new(false));
        let p = paused.clone();
        ctrlc::set_handler(move || {
            p.store(true, Ordering::SeqCst);
        }).expect("Error setting Ctrl-C handler");

        loop {
            if paused.load(Ordering::SeqCst) {
                println!("\nDownload paused. You can resume it by running the program again.");
                break;
            }
            let bytes_read = response.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            file.seek(SeekFrom::End(0))?;
            file.write_all(&buffer[..bytes_read])?;
            bar.inc(bytes_read as u64);
        }

        bar.finish();
    }

    Ok(())
}

fn print(msg: String, quiet_mode: bool) {
    if !quiet_mode {
        println!("{}", msg);
    }
}

fn custom_style(msg: String, color: &str) -> console::StyledObject<String> {
    match color {
        "green" => console_style(msg).green(),
        "red" => console_style(msg).red(),
        _ => console_style(msg),
    }
}

fn log_download(url: &str, result: &Result<(), Box<dyn Error>>, duration: std::time::Duration) -> Result<(), Box<dyn Error>> {
    let log_message = match result {
        Ok(_) => format!("SUCCESS: {} ({} seconds)\n", url, duration.as_secs()),
        Err(e) => format!("FAILED: {} ({}) ({} seconds)\n", url, e, duration.as_secs()),
    };

    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("download.log")?;

    log_file.write_all(log_message.as_bytes())?;
    Ok(())
}
