pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

pub mod app;
pub mod ui;
pub mod data;
pub mod models;
pub mod utils;

// Re-export commonly used types
pub use app::state::{App, HttpMethod};
pub use data::{AppConfig, History, CollectionManager};
