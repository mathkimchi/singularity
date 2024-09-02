use super::{Request, SubappInterface};

pub struct TempWritingBox {
    text: String,

    requests: Vec<Request>,
}
impl TempWritingBox {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            requests: vec![Request::SetName("Writing Box".to_string())],
        }
    }
}
impl SubappInterface for TempWritingBox {
    fn inform_event(&mut self, event: super::Event) {}

    fn dump_requests(&mut self) -> Vec<super::Request> {
        std::mem::take(&mut self.requests)
    }
}
impl Default for TempWritingBox {
    fn default() -> Self {
        Self::new()
    }
}
