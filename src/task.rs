use actix::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};

#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct MessageWrapper<T>(pub T);

/// Helper macro to send messages to the bus
macro_rules! send_message {
    ($self:ident, $t:ty, $data:expr) => {
        $self.issue_system_async::<MessageWrapper<$t>>(MessageWrapper { 0: $data });
    };
}

/// Helper macro to subscribe to a given message type
macro_rules! subscribe_to {
    ($t:ty, $self:ident, $ctx:expr) => {
        $self.subscribe_sync::<BrokerType, MessageWrapper<$t>>($ctx)
    };
}

/// Helper macro to define main callback
macro_rules! start_main_loop {
    ($millis:expr, $ctx:expr) => {
        IntervalFunc::new(Duration::from_millis($millis), Self::on_main)
            .finish()
            .spawn($ctx);
    };
}

macro_rules! consumer {
    ($t:ty) => {
        Handler<MessageWrapper<$ty>>
    }
}

pub struct Context {
    pub barrier: Arc<Barrier>,
    pub running: Arc<AtomicBool>,
}

pub trait TaskBehaviour {
    fn get_ctx(&self) -> &Context;
    fn get_name(&self) -> &str;

    fn wait_start(&self) {
        self.get_ctx().barrier.wait();
    }
    fn is_running(&self) -> bool {
        self.get_ctx().running.load(Ordering::Relaxed)
    }
}
