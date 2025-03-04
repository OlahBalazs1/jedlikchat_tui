use std::ops::Deref;
use std::sync::Arc;
#[derive(Clone)]
pub struct CancelToken {
    state: Arc<bool>,
}
impl CancelToken {
    pub fn new() -> Self {
        Self {
            state: Arc::new(false),
        }
    }
    pub fn set(&self) {
        // SAFETY: Can't point to invalid address
        unsafe {
            let state: *mut bool = Arc::as_ptr(&self.state) as *mut bool;
            *state = true
        }
    }
}
impl Deref for CancelToken {
    type Target = bool;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
