trait Session {
    fn init(&mut self);
    fn handle(&mut self, event: Event);
}

enum Event {
    NoOp,
}
