use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
}

pub struct EventLoop {
    rx: mpsc::Receiver<AppEvent>,
    _tx: mpsc::Sender<AppEvent>,
    _handle: thread::JoinHandle<()>,
}

impl EventLoop {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel();
        let tx2 = tx.clone();
        let handle = thread::spawn(move || {
            loop {
                if event::poll(tick_rate).unwrap_or(false) {
                    match event::read() {
                        Ok(CrosstermEvent::Key(k)) => { let _ = tx2.send(AppEvent::Key(k)); }
                        Ok(CrosstermEvent::Mouse(m)) => { let _ = tx2.send(AppEvent::Mouse(m)); }
                        Ok(CrosstermEvent::Resize(w, h)) => { let _ = tx2.send(AppEvent::Resize(w, h)); }
                        _ => {}
                    }
                } else {
                    let _ = tx2.send(AppEvent::Tick);
                }
            }
        });
        Self { rx, _tx: tx, _handle: handle }
    }

    pub fn next(&self) -> Option<AppEvent> {
        self.rx.recv().ok()
    }
}
