use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::thread::Thread;
use std::time::Duration;
use std::{thread, time};

use actix::prelude::*;
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use imc::DevDataText::DevDataText;
use imc::GpsFix::GpsFix;
use imc::Message::Message;
use serialport::SerialPort;

use crate::drivers::gps::nmea::{Sentence, State};
use crate::ix::Parameter;
use crate::BrokerType;
use crate::MessageWrapper;
use crate::TaskBehaviour;
use crate::{ix, task};

mod field_reader;
mod nmea;
mod sentences;
mod tests;

#[derive(Default)]
pub struct Configuration {
    pub io_dev: Parameter<String>,
    pub baud: Parameter<u32>,
    pub io_timeout: Parameter<u64>,
}

// Task fields' definition
pub struct Task {
    pub ctx: task::Context,
    pub fix: GpsFix,
    pub parser: nmea::Parser,
    pub io: Option<Box<dyn SerialPort>>,
    pub bfr: String,
    cfg: Configuration,
}

// Task Trait implementation

impl TaskBehaviour for Task {
    fn get_ctx(&self) -> &task::Context {
        &self.ctx
    }

    fn get_name(&self) -> &str {
        "GPS Driver"
    }

    fn register_configuration(&mut self) {
        self.cfg
            .io_dev
            .name("IO Device")
            .default(String::from("/dev/ttyUSB0"))
            .description("IO port used to connect to the device");

        self.cfg
            .baud
            .name("IO - Baud Rate")
            .default(115200)
            .description("Baud rate applied to the device");

        self.cfg
            .io_timeout
            .name("IO - Communications Timeout")
            .default(10)
            .description("In milliseconds");
    }
}

// Task specific behaviour (main loop, etc)

impl Task {
    pub fn new(context: task::Context) -> Task {
        Task {
            ctx: context,
            fix: Default::default(),
            parser: nmea::Parser::new(),
            io: None,
            bfr: String::from(""),
            cfg: Default::default(),
        }
    }

    fn handle_latitude(&mut self, lat_field: Option<f64>, ns_field: Option<String>) -> bool {
        if lat_field.is_some() && ns_field.is_some() {
            let ns = ns_field.unwrap();

            self.fix._lat = lat_field.unwrap();
            if ns == "S" {
                self.fix._lat = -self.fix._lat;
            }

            return true;
        }
        false
    }

    fn handle_longitude(&mut self, lon_field: Option<f64>, ew_field: Option<String>) -> bool {
        if lon_field.is_some() && ew_field.is_some() {
            let ew = ew_field.unwrap();

            self.fix._lon = lon_field.unwrap();
            if ew == "W" {
                self.fix._lon = -self.fix._lon;
            }

            return true;
        }
        false
    }

    /// Handle sentence and feed corresponding IMC messages
    fn handle_sentence(&mut self, sentence: Sentence) {
        match sentence {
            Sentence::Invalid => println!("ERROR: unknown sentence"),
            /// Handle GGA Sentence
            Sentence::GGA(m) => {
                println!("debug: GGA");
                if m.validity == 1 {
                    self.fix._type = imc::GpsFix::TypeEnum::GFT_STANDALONE as u8;
                    self.fix._validity |= (imc::GpsFix::ValidityBits::GFV_VALID_POS as u16);
                } else if m.validity == 2 {
                    self.fix._type = imc::GpsFix::TypeEnum::GFT_DIFFERENTIAL as u8;
                    self.fix._validity |= (imc::GpsFix::ValidityBits::GFV_VALID_POS as u16);
                }

                if self.handle_latitude(m.lat, m.ns)
                    && self.handle_longitude(m.lon, m.ew)
                    && m.alt.is_some()
                    && m.sat.is_some()
                {
                    if let Some(gsep) = m.gsep {
                        self.fix._height += gsep;
                    }

                    self.fix._lat = self.fix._lat.to_radians();
                    self.fix._lon = self.fix._lon.to_radians();
                    self.fix._validity |= (imc::GpsFix::ValidityBits::GFV_VALID_POS as u16);
                } else {
                    self.fix._validity &= !(imc::GpsFix::ValidityBits::GFV_VALID_POS as u16);
                }

                if let Some(hdop) = m.hdop {
                    self.fix._hdop = hdop;
                    self.fix._validity |= (imc::GpsFix::ValidityBits::GFV_VALID_HDOP as u16);
                }
            }
            Sentence::VTG(m) => {
                // @fixme: magnetic or true?
                if let Some(cog) = m.cog_true {
                    //@todo normalize angles
                    self.fix._cog = cog.to_radians();
                    self.fix._validity |= (imc::GpsFix::ValidityBits::GFV_VALID_COG as u16);
                }

                if let Some(sog) = m.sog_kph {
                    // to mps
                    self.fix._sog = sog * 1000.0 / 3600.0;
                    self.fix._validity |= (imc::GpsFix::ValidityBits::GFV_VALID_SOG as u16);
                }
            }
            Sentence::RMC(m) => println!("RMC"),
            Sentence::ZDA(m) => {
                if m.utc.is_some() {
                    self.fix._validity |= (imc::GpsFix::ValidityBits::GFV_VALID_TIME as u16);
                }

                if m.day.is_some() && m.month.is_some() && m.year.is_some() {
                    self.fix._validity |= (imc::GpsFix::ValidityBits::GFV_VALID_TIME as u16);
                }
            }
        }
    }

    /// Main loop
    fn on_main(&mut self, _context: &mut Context<Self>) {
        let mut serial_buf: Vec<u8> = vec![0; 1024];
        self.io
            .as_mut()
            .unwrap()
            .read(serial_buf.as_mut_slice())
            .expect("Found no data!");

        for b in serial_buf {
            let c = b as char;
            if c == '\n' {
                /// Log received sentence
                let mut log = DevDataText::new();
                log._value = self.bfr.clone();
                send_message!(self, imc::DevDataText::DevDataText, log);

                // reset state
                self.parser.reset();
                self.bfr.clear();
                continue;
            }

            self.bfr.push(c);
            match self.parser.push(c) {
                Ok(sentence) => self.handle_sentence(sentence),
                Err(State::InvalidId(id)) => println!("unsupported sentence id: {}", id),
                Err(State::InvalidFields) => println!("ERROR: invalid message fields"),
                Err(State::ChecksumMismatch { expected, received }) => println!(
                    "ERROR: mismatch: expected {}, received {}",
                    expected, received
                ),
                _ => {}
            }
        }
    }
}

// Task lifecycle

impl Actor for Task {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.io = Some(
            serialport::new(self.cfg.io_dev.get(), *self.cfg.baud.get())
                .timeout(Duration::from_millis(*self.cfg.io_timeout.get()))
                .open()
                .expect("Failed to open port"),
        );

        /// go
        start_main_loop!(1000, ctx);
    }
}
