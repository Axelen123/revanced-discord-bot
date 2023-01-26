use crate::*;
use chrono::{DateTime, Utc};

// There isn't a good library that serves our needs for this unfortunately.
pub struct Scheduler {
    items: HashMap<String, Item>,
}

impl Scheduler {
    pub async fn add(&mut self, at: DateTime<Utc>, item: impl Schedulable) {
        self.items[&item.id()] = Item {
            at,
            inner: Box::new(item),
            handle: panic!("lol"),
        };
    }

    pub fn cancel(&mut self, id: String) -> bool {
        if let Some(item) = self.items.remove(&id) {
            item.handle.abort();
            return true;
        }
        false
    }
}

struct Item {
    at: DateTime<Utc>,
    handle: JoinHandle<()>,
    inner: Box<dyn Schedulable>,
}

pub enum Update {
    Cancel,
    Reschedule(DateTime<Utc>),
}

#[serenity::async_trait]
pub trait Schedulable {
    // TODO: perhaps this shouldn't be part of this trait at all...
    fn id(&self) -> String;
    /// Called once every hour.
    async fn sync(&mut self);

    async fn execute(&mut self);
}
