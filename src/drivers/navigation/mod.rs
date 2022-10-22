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

pub struct Navigation {
    pub ctx :task::Context,
}

impl Task for Navigation {
    fn get_ctx(&self) -> &task::Context {
        &self.ctx
    }

    fn get_name(&self) -> &str {
        "Navigation Monitor"
    }
}


impl Navigation {
    fn on_main(&mut self, _context: &mut Context<Self>) {
        self.issue_async::<BrokerType, MessageWrapper<u16>>(MessageWrapper { 0: 2} );
    }
}

impl Actor for Navigation {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        subscribe_to!(GpsFix, self, ctx);

        start_main_loop!(1000, ctx);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        println!("{} stopped", self.get_name());
    }
}

impl Handler<MessageWrapper<GpsFix>> for Navigation {
    type Result = ();

    fn handle(&mut self, msg: MessageWrapper<GpsFix>, _ctx: &mut Self::Context) {
        println!("Navigation Received: {:?}", msg.0._header._mgid);
    }
}