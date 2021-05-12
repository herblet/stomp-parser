pub mod frames;
pub mod headers;

pub use frames::client::*;
pub use frames::server::*;
pub use headers::*;

#[cfg(test)]
mod test {

    use super::AckType;
    #[test]
    fn ack_display() {
        let s = format!("Prefix: {}", AckType::ClientIndividual);

        assert_eq!("Prefix: client-individual", s);
    }
}
