use anyhow::Result;
use tokio::sync::mpsc::UnboundedSender;

trait Handler {
    type Item;

    fn get_sender(&self) -> UnboundedSender<Self::Item>;
    fn do_work(&self) -> Result<()>;
    fn connect_to(&self, other: Self) -> Result<()>;
}

struct Streamline {}
