use actix::prelude::*;
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::thread::Thread;
use std::time::Duration;
use std::{thread, time};

use crate::ix;
use crate::task;
use crate::BrokerType;
use crate::MessageWrapper;
use crate::TaskBehaviour;

#[derive(Default)]
pub struct Configuration {
    pub out_path: ix::Parameter<String>,
}

pub struct Task {
    pub ctx: task::Context,
    cfg: Configuration,
}

impl TaskBehaviour for Task {
    fn get_ctx(&self) -> &task::Context {
        &self.ctx
    }

    fn get_name(&self) -> &str {
        "Navigation Monitor"
    }

    fn register_configuration(&mut self) {
        self.cfg
            .out_path
            .name("Log path")
            .default(String::from("out/log.lsf"))
            .description("Path to log data to");
    }
}

impl Task {
    pub fn new(context: task::Context) -> Task {
        Task {
            ctx: context,
            cfg: Default::default(),
        }
    }

    fn on_main(&mut self, _context: &mut Context<Self>) {}
}

impl Actor for Task {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        subscribe_to!(imc::GpsFix, self, ctx);
        subscribe_to!(imc::DevDataText, self, ctx);

        start_main_loop!(1000, ctx);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        println!("{} stopped", self.get_name());
    }
}

impl Handler<MessageWrapper<imc::GpsFix>> for Task {
    type Result = ();

    fn handle(&mut self, msg: MessageWrapper<imc::GpsFix>, _ctx: &mut Self::Context) {
        println!("logger: {:?}", msg.0._header._mgid);
    }
}

impl Handler<MessageWrapper<imc::DevDataText>> for Task {
    type Result = ();

    fn handle(&mut self, msg: MessageWrapper<imc::DevDataText>, _ctx: &mut Self::Context) {
        println!("logger: {}", msg.0._value);
    }
}
