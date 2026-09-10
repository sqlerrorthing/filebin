pub mod rabbitmq;
pub mod stream;

#[cfg(test)]
mod tests {
    #[test]
    fn panic() {
        assert_eq!(2, 4)
    }

    #[test]
    fn success() {}
}
