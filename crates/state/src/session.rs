use gpui::{Context, EventEmitter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionState {
    SignedOut,
    SignedIn,
}

pub enum SessionEvent {
    SignedIn,
    SignedOut,
}

pub struct Session {
    state: SessionState,
    name: String,
}

impl EventEmitter<SessionEvent> for Session {}

impl Session {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            state: SessionState::SignedOut,
            name: String::new(),
        }
    }

    pub fn state(&self) -> &SessionState {
        &self.state
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn sign_in(&mut self, name: String, cx: &mut Context<Self>) {
        self.state = SessionState::SignedIn;
        self.name = name;
        cx.emit(SessionEvent::SignedIn);
    }

    pub fn sign_out(&mut self, cx: &mut Context<Self>) {
        self.state = SessionState::SignedOut;
        self.name.clear();
        cx.emit(SessionEvent::SignedOut);
    }
}