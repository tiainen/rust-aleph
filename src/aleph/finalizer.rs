use aleph_bft::FinalizationHandler;
use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use log::{debug, error};

use super::data::Data;

#[derive(Clone)]
pub struct SimpleFinalizationHandler {
    sender: UnboundedSender<Data>,
}

impl FinalizationHandler<Data> for SimpleFinalizationHandler {
    fn data_finalized(&mut self, data: Data) {
        debug!("SimpleFinalizationHandler::data_finalized");

        if let Err(e) = self.sender.unbounded_send(data) {
            error!("Error while sending data from FinalizationHandler: {e:?}");
        }
    }
}

impl SimpleFinalizationHandler {
    pub fn new() -> (Self, UnboundedReceiver<Data>) {
        let (sender, receiver) = mpsc::unbounded();
        ( Self { sender }, receiver)
    }
}
