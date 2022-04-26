///

pub mod hub;
pub mod client;
pub mod keys;
pub mod opt;

mod event;
pub use event::Event;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
