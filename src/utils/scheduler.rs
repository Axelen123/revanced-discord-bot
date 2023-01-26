use crate::*;
use std::cmp::{min, max};
use chrono::{DateTime, Utc, Duration};
use tokio::spawn;
use tokio::sync::Mutex;
use tokio::time::sleep;

// There isn't a good library that serves our needs for this unfortunately.
pub struct Scheduler {
    // TODO: i need dashmap here or smth hhhhhh
    items: HashMap<String, tokio::sync::Mutex<Item>>,
}

fn time_left(timestamp: DateTime<Utc>) -> Duration {
    let now = Utc::now();
    max((now - timestamp), Duration::zero())
}

impl Scheduler {
    pub async fn add(&mut self, at: DateTime<Utc>, item: impl Schedulable) {
        let id = item.id();
        let handle = spawn(async {
            loop {
                let mut datetime = at;
                let duration = time_left(datetime);
                if duration.is_zero() {
                    break;
                }
                sleep(std::cmp::min(Duration::hours(1), duration).to_std().expect("Scheduler time overflow")).await;

                if time_left(datetime).is_zero() {
                    break;
                } else {
                    let mut lock = self.items[&id].lock().await;
                    match lock.inner.sync().await {
                        Some(Update::Cancel) => {
                            self.cancel(id);
                            return;
                        },
                        Some(Update::Reschedule(v)) => {
                            datetime = v;
                        },
                        None => {},
                    }
                }
            }


        });
        self.items[&item.id()] = Mutex::new(Item {
            // at,
            inner: Box::new(item),
            handle,
        });
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
    // at: DateTime<Utc>,
    handle: JoinHandle<()>,
    inner: Box<dyn Schedulable>,
}

pub enum Update {
    Cancel,
    Reschedule(DateTime<Utc>),
}

#[serenity::async_trait]
pub trait Schedulable: Send + Sync {
    // TODO: perhaps this shouldn't be part of this trait at all...
    fn id(&self) -> String;
    /// Called once every hour.
    async fn sync(&mut self) -> Option<Update>;

    async fn execute(&mut self);
}
