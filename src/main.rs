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
use std::path::Path;
use std::fmt;
use byteorder::{BigEndian, ReadBytesExt};

#[derive(Debug)]
struct AdaptationFieldHeader {
    discontinuity_indicator: bool,
    random_access_indicator: bool,
    elementary_stream_priority_indicator: bool,
    pcr_flag: bool,
    opcr_flag: bool,
    splicing_point_flag: bool,
    transport_private_data_flag: bool,
    adaptation_field_extension_flag: bool,
}

#[derive(Debug)]
struct AdaptationField {
    length: u8,
    header: Option<AdaptationFieldHeader>,
}

struct TransportPacketHeader {
    transport_error_indicator: bool,
    payload_unit_start_indicator: bool,
    transport_priority: bool,
    pid: u16,
    transport_scrambling_control: u8,
    adaptation_field_control: u8,
    continuity_counter: u8,
    final_byte: u8,
}

impl TransportPacketHeader {
    pub fn is_pat(&self) -> bool {
        return self.pid == 0x00;
    }

    pub fn has_adaptation_field(&self) -> bool {
        return (self.adaptation_field_control & 0b10) != 0;
    }

    pub fn has_payload(&self) -> bool {
        return (self.adaptation_field_control & 0b01) != 0
    }
}

impl fmt::Display for TransportPacketHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"\ttransport_error_indicator: {}\n\
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

struct TransportPacket {
    header: TransportPacketHeader,
    adaptation_field: Option<AdaptationField>,
}

impl TransportPacket {
    pub fn new(buf: &[u8]) -> Result<TransportPacket, String> {
        if buf.len() < 188 {
            return Err("Length of buffer is less than 188 bytes!".to_string());
        }

        let header = TransportPacketHeader {
            transport_error_indicator: (buf[1] & 0b10000000) != 0,
            payload_unit_start_indicator: (buf[1] & 0b01000000) != 0,
            transport_priority: (buf[1] & 0b00100000) != 0,
            pid: (((buf[1] as u16) << 8) | (buf[2] as u16)) & 0b0001111111111111,
            transport_scrambling_control: (buf[3] & 0b11000000) >> 6,
            adaptation_field_control: (buf[3] & 0b00110000) >> 4,
            continuity_counter: buf[3] & 0b1111,
            final_byte: buf[3],
        };

        let mut af: Option<AdaptationField> = None;

        if header.has_adaptation_field() {
            let af_data = &buf[4 ..];

            let mut af_header: Option<AdaptationFieldHeader> = None;

            let af_length = af_data[0];

            match header.has_payload() {
                true => {
                    if af_length > 182 {
                        return Err(format!("AdaptationField length too large! ({})", af_length));
                    }
                },
                false => {
                    if af_length != 183 {
                        return Err(format!("No payload and AdaptationField length not exactly 183! ({})", af_length));
                    }
                },
            };

            if af_length > 0 {
                af_header = Some(AdaptationFieldHeader{
                    discontinuity_indicator:              (af_data[1] & 0b10000000) != 0,
                    random_access_indicator:              (af_data[1] & 0b01000000) != 0,
                    elementary_stream_priority_indicator: (af_data[1] & 0b00100000) != 0,
                    pcr_flag:                             (af_data[1] & 0b00010000) != 0,
                    opcr_flag:                            (af_data[1] & 0b00001000) != 0,
                    splicing_point_flag:                  (af_data[1] & 0b00000100) != 0,
                    transport_private_data_flag:          (af_data[1] & 0b00000010) != 0,
                    adaptation_field_extension_flag:      (af_data[1] & 0b00000001) != 0,
                });
            }

            af = Some(AdaptationField {
                length: af_length,
                header: af_header,
            });
        }

        if header.has_payload() {

        }

        return Ok(TransportPacket {
            header: header,
            adaptation_field: af,
        });
    }

    pub fn is_pat(&self) -> bool {
        return self.header.is_pat();
    }

    pub fn has_adaptation_field(&self) -> bool {
        return self.header.has_adaptation_field();
    }

    pub fn has_payload(&self) -> bool {
        return self.header.has_payload();
    }
}

impl fmt::Display for TransportPacket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "transport packet:\n\
                   header:\n{}\n\
                   adaptation_field:\n\t{:?}", self.header, self.adaptation_field.as_ref())
    }
}

fn read_transport_packet(file: &mut File) -> Result<TransportPacket, std::io::Error> {
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

    match TransportPacket::new(&transport_packet_buf) {
        Err(why) => {
            error!("Failed to parse a transport packet: {}", why);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, why));
        },
        Ok(tp) => {
            return Ok(tp);
        }
    }
}

fn main() {
    env_logger::init().unwrap();
    let args: Vec<String> = env::args().collect();
    let mut failed_to_read = false;

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

    while !failed_to_read {
        match read_transport_packet(&mut file) {
            Err(why) => {
                error!("Failure at reading a transport packet: {}", why);
                failed_to_read = true;
            },
            Ok(tp) => {
                if tp.is_pat() {
                    error!("PAT FOUND:\n{}", tp);
                }
            }
        }
    }
}
