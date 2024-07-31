extern crate clap;

use clap::{Arg, App};
use indicatif::{ProgressBar, ProgressStyle, HumanBytes};
use reqwest::blocking::Client;
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use console::style as console_style;

fn main() {
    let matches = App::new("Rget")
        .version("0.1.0")
        .about("wget clone written in Rust")
        .arg(
            Arg::with_name("URL")
                .required(true)
                .takes_value(true)
                .index(1)
                .help("url to download"),
        )
        .get_matches();
    let url = matches.value_of("URL").unwrap();
    println!("{}", url);

    if let Err(e) = download(url, false) {
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

fn download(target: &str, quiet_mode: bool) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let mut response = client.get(target).send()?;

    print(
        format!(
            "HTTP request sent... {}",
            custom_style(format!("{}", response.status()), "green")
        ),
        quiet_mode,
    );
    if response.status().is_success() {
        let headers = response.headers().clone();
        let ct_len: Option<u64> = headers
            .get(CONTENT_LENGTH)
            .and_then(|ct_len| ct_len.to_str().ok())
            .and_then(|s| s.parse().ok());

        let ct_type = headers
            .get(CONTENT_TYPE)
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("application/octet-stream");

        match ct_len {
            Some(len) => {
                print(
                    format!(
                        "Length: {} ({})",
                        custom_style(len.to_string(), "green"),
                        custom_style(format!("{}", HumanBytes(len)), "red")
                    ),
                    quiet_mode,
                );
            }
            None => {
                print(format!("Length: {}", custom_style("unknown".to_string(), "red")), quiet_mode);
            }
        }

        print(format!("Type: {}", custom_style(ct_type.to_string(), "green")), quiet_mode);

        let fname = target.split('/').last().unwrap().to_string();
        print(
            format!("Saving to: {}", custom_style(fname.clone(), "green")),
            quiet_mode,
        );

        let chunk_size = match ct_len {
            Some(x) => x as usize / 99,
            None => 1024usize, // default chunk size
        };

        let mut buf = Vec::new();
        let bar = create_progress_bar(quiet_mode, &fname, ct_len);

        loop {
            let mut buffer = vec![0; chunk_size];
            let bcount = response.read(&mut buffer[..])?;
            buffer.truncate(bcount);
            if !buffer.is_empty() {
                buf.extend(buffer.into_boxed_slice().into_vec().iter().cloned());
                bar.inc(bcount as u64);
            } else {
                break;
            }
        }

        bar.finish();
        save_to_file(&buf, &fname)?;
    }

    Ok(())
}

fn save_to_file(buffer: &[u8], filename: &str) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(filename)?;
    file.write_all(buffer)?;
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
