#![cfg(unix)]
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::from_utf8;

pub struct OsInfo();
const PROC: &str = "proc";

impl OsInfo {
    /// The first element is the name of the distro and the second is the version
    pub fn distro_info() -> (String, String) {
        let file = File::open("/etc/os-release").expect("could not open version dir");
        let reader = BufReader::new(file);
        let mut counter = 0;
        let mut name = String::from("");
        let mut version = String::from("");

        for line in reader.lines() {
            if counter == 2 {
                break;
            }

            match line {
                Ok(data) => {
                    let text: Vec<_> = data.split('=').collect();

                    if text[0].to_lowercase().trim() == "name" {
                        name = text[1].to_string();
                        counter += 1;
                    } else if text[0].to_lowercase().trim() == "version" {
                        version = text[1][1..text[1].len() - 1].to_string();
                        counter += 1;
                    }
                }
                Err(_) => {}
            }
        }

        (name, version)
    }

    pub fn kernal_version() -> String {
        let dir = format!("/{}/version", PROC);

        let file = File::open(dir).expect("could not open version dir");
        let mut reader = BufReader::new(file);
        let mut buf = vec![];
        let _ = reader.read_until(b'(', &mut buf);

        let start = "Linux version ".len();
        let end = buf.len() - 2;
        let version = from_utf8(&buf[start..end]).expect("Failed to convert bytes to string");

        version.to_string()
    }

    pub fn number_of_cores() -> u8 {
        let dir = format!("/{}/cpuinfo", PROC);
        let file = File::open(dir).expect("could not open version dir");
        let reader = BufReader::new(file);
        let mut counter: u8 = 0;

        for line in reader.lines() {
            match line {
                Ok(data) => {
                    let text: Vec<_> = data.split(":").collect();

                    if text[0].trim() == "processor" {
                        counter += 1;
                    }
                }
                Err(_) => {}
            }
        }
        counter
    }
}
