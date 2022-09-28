use std::collections::VecDeque;

struct Msg {
    s: String,
    seen: bool,
}

#[derive(Default)]
pub struct Messages {
    msg: VecDeque<Msg>,
}

impl Messages {
    pub fn add<T: ToString>(&mut self, msg: T) {
        self.msg.push_back(Msg {
            s: msg.to_string(),
            seen: false,
        });
    }

    pub fn is_empty(&self) -> bool {
        self.msg.is_empty()
    }

    pub fn current(&mut self) -> Option<String> {
        let more = if self.msg.len() > 1 { " -More-" } else { "" };
        if let Some(mut m) = self.msg.get_mut(0) {
            m.seen = true;
            let result = format!("{}{more}", m.s);
            Some(result)
        } else {
            None
        }
    }

    pub fn advance(&mut self) {
        self.msg.pop_front();
    }

    pub fn is_seen(&self) -> bool {
        if let Some(m) = self.msg.front() {
            m.seen
        } else {
            false
        }
    }
}
