use openergy_ipc::Event;
use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, Weak,
    },
};
#[derive(Default)]
pub struct EventQueue {
    queue: Mutex<VecDeque<Event>>,
    pub status: AtomicBool,
    pub logs: AtomicBool,
}
impl EventQueue {
    pub fn push(&self, event: Event) {
        if let Ok(mut q) = self.queue.lock() {
            if q.len() == 256 {
                q.pop_front();
            }
            q.push_back(event);
        }
    }
    pub fn drain(&self) -> Vec<Event> {
        self.queue.lock().map(|mut q| q.drain(..).collect()).unwrap_or_default()
    }
}
#[derive(Default)]
pub struct EventBus {
    clients: Mutex<Vec<Weak<EventQueue>>>,
}
impl EventBus {
    pub fn client(&self) -> Arc<EventQueue> {
        let q = Arc::new(EventQueue::default());
        if let Ok(mut clients) = self.clients.lock() {
            clients.retain(|s| s.strong_count() > 0);
            clients.push(Arc::downgrade(&q));
        }
        q
    }
    pub fn publish(&self, event: Event) {
        if let Ok(mut clients) = self.clients.lock() {
            clients.retain(|s| {
                let Some(q) = s.upgrade() else { return false };
                let enabled = if event.event == "log.line" {
                    q.logs.load(Ordering::Relaxed)
                } else {
                    q.status.load(Ordering::Relaxed)
                };
                if enabled {
                    q.push(event.clone());
                }
                true
            });
        }
    }
}
