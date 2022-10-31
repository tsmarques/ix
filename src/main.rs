use actix::prelude::*;
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::thread::Thread;
use std::time::Duration;
use std::{thread, time};
use task::MessageWrapper;
use task::TaskBehaviour;

#[macro_use]
mod task;
mod drivers;

type BrokerType = SystemBroker;

fn main() {
    println!("Starting");
    let sys = System::new();

    // @todo set correct value
    let task_barrier = Arc::new(Barrier::new(2));
    let mut task_flag = Arc::new(AtomicBool::new(true));

    let gps_task = drivers::gps::Task::new(task::Context {
        running: Arc::clone(&task_flag),
        barrier: Arc::clone(&task_barrier),
    });

    let nav_task = drivers::navigation::Task {
        ctx: task::Context {
            running: Arc::clone(&task_flag),
            barrier: Arc::clone(&task_barrier),
        },
    };

    sys.block_on(async {
        nav_task.start();
        gps_task.start();
    });
    sys.run().unwrap();
    println!("Done");
}
