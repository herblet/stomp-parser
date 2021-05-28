mod frames;
pub mod headers;

pub use frames::client;
pub use frames::server;

#[cfg(test)]
mod test {

    use super::headers::AckType;
    #[test]
    fn ack_display() {
        let s = format!("Prefix: {}", AckType::ClientIndividual);

        assert_eq!("Prefix: client-individual", s);
    }
}
