use crate::*;
use std::cmp::{min, max};
use chrono::{DateTime, Utc, Duration};
use tokio::spawn;
use std::sync::Arc;
use tokio::time::sleep;
use dashmap::DashMap;

// TODO: should we cancel everything when `Scheduler` gets dropped?
// There isn't a good library that serves our needs for this unfortunately.
pub struct Scheduler {
    items: Arc<DashMap<String, Item>>,
}

fn time_left(timestamp: DateTime<Utc>) -> Duration {
    let now = Utc::now();
    max(now - timestamp, Duration::zero())
}

impl Scheduler {
    pub async fn add(&self, at: DateTime<Utc>, item: impl Schedulable + 'static) {
        let id = item.id();
        let ptr = self.items.clone();
        let handle = spawn(async move {
            let mut datetime = at;
            loop {
                let duration = time_left(datetime);
                if duration.is_zero() {
                    break;
                }
                sleep(std::cmp::min(Duration::hours(1), duration).to_std().expect("Scheduler time overflow")).await;

                if time_left(datetime).is_zero() {
                    break;
                } else {
                    let mut item = ptr.get_mut(&id).unwrap();
                    match item.inner.sync().await {
                        Some(Update::Cancel) => {
                            Self::_cancel(&ptr, id);
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
        self.items.insert(item.id(), Item {
            // at,
            inner: Box::new(item),
            handle,
        });
    }

    #[inline]
    pub fn cancel(&self, id: String) -> bool {
        Self::_cancel(&self.items, id)
    }

    fn _cancel(ptr: &DashMap<String, Item>, id: String) -> bool {
        if let Some((_, item)) = ptr.remove(&id) {
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
