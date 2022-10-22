use std::thread::Thread;
use std::{thread, time};
use std::sync::{Arc, Barrier};
use std::sync::atomic::{AtomicBool, Ordering};
use actix::prelude::*;
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use std::time::Duration;

use crate::task;
use crate::Task;
use crate::MessageWrapper;
use crate::BrokerType;

use imc::Message::Message;
use imc::GpsFix::GpsFix;

mod nmea;
mod sentences;


// Task fields' definition
pub struct GpsDriver {
    pub ctx :task::Context,
    pub fix :GpsFix,
    pub parser :nmea::Parser,
}

// Task Trait implementation

impl Task for GpsDriver {
    fn get_ctx(&self) -> &task::Context {
        &self.ctx
    }

    fn get_name(&self) -> &str {
        "GPS Driver"
    }
}

// Task specific behaviour (main loop, etc)

impl GpsDriver {
    pub fn new(context: task::Context) -> GpsDriver {
       GpsDriver {
           ctx: context,
           fix: Default::default(),
           parser: nmea::Parser::new(),
       }
    }

    fn on_main(&mut self, _context: &mut Context<Self>) {
        send_message!(self, GpsFix, GpsFix::new());
    }
}

// Task lifecycle

impl Actor for GpsDriver {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        /// register subscriptions
        subscribe_to!(u16, self, ctx);

        /// go
        start_main_loop!(1000, ctx);
    }
}

// Consumers

impl Handler<MessageWrapper<u16>> for GpsDriver {
    type Result = ();

    fn handle(&mut self, msg: MessageWrapper<u16>, _ctx: &mut Self::Context) {
        println!("Gps Received: {:?}", msg.0);
    }
}