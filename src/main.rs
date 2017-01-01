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
use std::io::SeekFrom;
use std::io::Result;
use std::path::Path;
use std::fmt;
use byteorder::{BigEndian, ReadBytesExt};

struct TransportPacket {
    transport_error_indicator: bool,
    payload_unit_start_indicator: bool,
    transport_priority: bool,
    pid: u16,
    transport_scrambling_control: u8,
    adaptation_field_control: u8,
    continuity_counter: u8,
    final_byte: u8,
}

impl TransportPacket {
    pub fn new(buf: &[u8]) -> TransportPacket {
        if buf.len() < 188 {
            error!("Length of buffer is less than 188 bytes ({})!", buf.len());
        }

        TransportPacket {
            transport_error_indicator: (buf[1] & 0b10000000) != 0,
            payload_unit_start_indicator: (buf[1] & 0b01000000) != 0,
            transport_priority: (buf[1] & 0b00100000) != 0,
            pid: (((buf[1] as u16) << 8) | (buf[2] as u16)) & 0b0001111111111111,
            transport_scrambling_control: (buf[3] & 0b11000000) >> 6,
            adaptation_field_control: (buf[3] & 0b00110000) >> 4,
            continuity_counter: buf[3] & 0b1111,
            final_byte: buf[3]
        }
    }
}

impl fmt::Display for TransportPacket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"transport packet: \n\
               \ttransport_error_indicator: {}\n\
               \tpayload_unit_start_indicator: {}\n\
               \ttransport_priority: {}\n\
               \tpid: {:#x}\n\
               \ttransport_scrambling_control: 0b{:02b}\n\
               \tadaptation_field_control: 0b{:02b}\n\
               \tcontinuity_counter: {}\n\
               \tfinal_byte: {:08b}",
               self.transport_error_indicator,
               self.payload_unit_start_indicator,
               self.transport_priority,
               self.pid,
               self.transport_scrambling_control,
               self.adaptation_field_control,
               self.continuity_counter,
               self.final_byte)
    }
}

fn read_transport_packet(file: &mut File) -> Result<TransportPacket> {
    let mut read_byte = [0; 1];
    let mut transport_packet_buf = [0; 188];

    while read_byte[0] != SYNC_BYTE_VALUE {
        match file.read(&mut read_byte) {
            Err(why) => {
                error!("Failed to read {} byte(s): {}", read_byte.len(),
                                                      why.description());
                return Err(why);
            },
            Ok(count) => {
                info!("Read {} byte(s)!", count);
                count
            }
        };
    }

    error!("Test: {:#x}", read_byte[0]);

    // FIXME: remove this after we switch to buffered stuff/peeking for the sync byte
    match file.seek(SeekFrom::Current(-1)) {
        Err(why) => {
            error!("Failed to seek: {}", why.description());
            return Err(why);
        },
        Ok(pos) => {
            error!("Seeked to position {}!", pos);
        }
    }

    // Read our fabulous MPEG-TS packet!
    match file.read(&mut transport_packet_buf) {
        Err(why) => {
            error!("Failed to read {} byte(s): {}", transport_packet_buf.len(),
                                                  why.description());
            return Err(why);
        },
        Ok(count) => {
            error!("Read {} byte(s)!", count);
            count
        }
    };

    return Ok(TransportPacket::new(&transport_packet_buf));
}

fn main() {
    env_logger::init().unwrap();
    let args: Vec<String> = env::args().collect();
    let mut read_byte = [0; 1];
    let mut transport_packet_buf = [0; 188];

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

    // FIXME: remove this after we switch to buffered stuff/peeking for the sync byte
    match file.seek(SeekFrom::Current(-1)) {
        Err(why) => {
            error!("Failed to seek: {}", why.description());
            return;
        },
        Ok(pos) => {
            error!("Seeked to position {}!", pos);
        }
    }

    // Read our fabulous MPEG-TS packet!
    match file.read(&mut transport_packet_buf) {
        Err(why) => {
            error!("Failed to read {} byte(s): {}", transport_packet_buf.len(),
                                                  why.description());
            return;
        },
        Ok(count) => {
            error!("Read {} byte(s)!", count);
            count
        }
    };

    let transport_packet = TransportPacket::new(&transport_packet_buf);
    error!("packet parsed:\n{}", transport_packet);

    match read_transport_packet(&mut file) {
        Err(why) => {
            error!("Failure at reading a transport packet: {}", why);
        },
        Ok(tp) => {
            error!("packet parsed:\n{}", tp);
        }
    }
}
