use std::collections::VecDeque;

#[derive(Default)]
pub struct Messages {
    msg: VecDeque<String>,
}

impl Messages {
    pub fn add<T: ToString>(&mut self, msg: T) {
        self.msg.push_back(msg.to_string());
    }

    pub fn current(&self) -> Option<&String> {
        self.msg.front()
    }

    pub fn advance(&mut self) {
        self.msg.pop_front();
    }
}
