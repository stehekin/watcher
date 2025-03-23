use anyhow::Result;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

struct Handler<T, H>
where
    T: prost::Message + 'static,
    H: HandlerType<T>,
{
    sender: UnboundedSender<T>,
    receiver: UnboundedReceiver<T>,
    next: Option<UnboundedSender<T>>,
    worker: H,
}

impl<T: prost::Message, H: HandlerType<T>> Handler<T, H> {
    fn get_sender(&self) -> UnboundedSender<T> {
        self.sender.clone()
    }

    pub(crate) fn connect<O: HandlerType<T>>(&mut self, other: Handler<T, O>) {
        self.next = Some(other.get_sender());
    }

    pub(crate) async fn do_work(&mut self) -> Result<()> {
        while let Some(mut message) = self.receiver.recv().await {
            let message = self.worker.enrich(&mut message);
            if let Some(next) = self.next.clone() {
                next.send(message)?
            }
        }

        anyhow::bail!("streamline closed")
    }
}

trait HandlerType<T: prost::Message> {
    fn enrich(&self, entity: &mut T) -> T;
}
