const SYNC_BYTE_VALUE: u8 = 0x47;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate byteorder;
extern crate bitflags;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use byteorder::{BigEndian, ReadBytesExt};

struct TransportPacket {
    transport_error_indicator: bool,
    payload_unit_start_indicator: bool,
    transport_priority: bool,
    pid: u16,
    final_byte: u8,
}

impl TransportPacket {
    fn new(buf: &[u8]) {
        if buf.len() < 24 {
            error!("Length of buffer is less than 24 bytes ({})!", buf.len());
        }
    }
}

fn main() {
    env_logger::init().unwrap();
    let args: Vec<String> = env::args().collect();
    let mut read_byte = [0; 1];

    if args.len() < 2 {
        error!("Not enough parameters! Usage: {} FILE", args[0]);
        return;
    }

    let path = Path::new(&args[1]);
    let printable_path = path.display();

    let mut file = match File::open(&path) {
        Err(why) => {
            error!("couldn't open '{}': {}", printable_path,
                                             why.description());
            return;
        },
        Ok(file) => file,
    };

    info!("Starting to read {}", printable_path);

    while read_byte[0] != SYNC_BYTE_VALUE {
        match file.read(&mut read_byte) {
            Err(why) => {
                error!("Failed to read {} byte(s): {}", read_byte.len(),
                                                      why.description());
                return;
            },
            Ok(count) => {
                error!("Read {} byte(s)!", count);
                count
            }
        };
    }

    error!("Test: {:#x}", read_byte[0]);
}
