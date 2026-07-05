pub mod action;
pub mod mapper;
pub mod reducer;

pub use action::Action;
pub use mapper::map_event_to_action;
pub use reducer::{ReduceResult, reduce};
