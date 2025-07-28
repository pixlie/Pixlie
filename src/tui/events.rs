use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum Event {
    Key(KeyEvent),
    Resize(u16, u16),
    Quit,
}

pub struct EventHandler {
    receiver: mpsc::UnboundedReceiver<Event>,
    _sender: mpsc::UnboundedSender<Event>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let event_sender = sender.clone();

        tokio::spawn(async move {
            loop {
                if let Ok(event) = event::poll(Duration::from_millis(100)) {
                    if event {
                        match event::read() {
                            Ok(CrosstermEvent::Key(key)) => {
                                // Handle special key combinations
                                if key.modifiers.contains(KeyModifiers::CONTROL) {
                                    match key.code {
                                        KeyCode::Char('c') => {
                                            let _ = event_sender.send(Event::Quit);
                                            break;
                                        }
                                        KeyCode::Char('q') => {
                                            let _ = event_sender.send(Event::Quit);
                                            break;
                                        }
                                        KeyCode::Char(',') => {
                                            // Send Ctrl+, as a special settings key
                                            let _ = event_sender.send(Event::Key(KeyEvent::new(
                                                KeyCode::F(12), // Use F12 as internal settings key
                                                KeyModifiers::NONE,
                                            )));
                                        }
                                        KeyCode::Char('w') => {
                                            // Send Ctrl+W as a special workspace manager key
                                            let _ = event_sender.send(Event::Key(KeyEvent::new(
                                                KeyCode::F(11), // Use F11 as internal workspace manager key
                                                KeyModifiers::NONE,
                                            )));
                                        }
                                        _ => {
                                            let _ = event_sender.send(Event::Key(key));
                                        }
                                    }
                                } else {
                                    let _ = event_sender.send(Event::Key(key));
                                }
                            }
                            Ok(CrosstermEvent::Resize(w, h)) => {
                                let _ = event_sender.send(Event::Resize(w, h));
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        Self {
            receiver,
            _sender: sender,
        }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.receiver.recv().await
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
