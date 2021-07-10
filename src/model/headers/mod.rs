//! Implements the model for headers, as specified in the
//! [STOMP Protocol Spezification,Version 1.2](https://stomp.github.io/stomp-specification-1.2.html).
#[macro_use]
mod macros;
use crate::common::functions::decode_str;
use crate::error::StompParseError;
use either::Either;
use paste::paste;
use std::convert::TryFrom;
use std::str::FromStr;

/// A Header that reveals it's type and it's value, and can be displayed
pub trait HeaderValue: std::fmt::Display {
    type OwnedValue;
    type Value;
    const OWNED: bool;

    fn header_name(&self) -> &str;
}

pub trait DecodableValue {
    fn decoded_value(&self) -> Result<Either<&str, String>, StompParseError>;
}
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct NameValue {
    pub name: String,
    pub value: String,
}

impl std::fmt::Display for NameValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}:{}", &self.name, &self.value)
    }
}

fn split_once(input: &str, delim: char) -> Option<(&str, &str)> {
    input
        .find(delim)
        .map(|idx| (&input[0..idx], &input[(idx + 1)..input.len()]))
}

impl FromStr for NameValue {
    type Err = StompParseError;
    fn from_str(input: &str) -> Result<NameValue, StompParseError> {
        split_once(input, ':')
            .map(|(name, value)| NameValue {
                name: name.to_owned(),
                value: value.to_owned(),
            })
            .ok_or_else(|| StompParseError::new(format!("Poorly formatted header: {}", input)))
    }
}

/// A pair of numbers which specify at what intervall the originator of
/// the containing message will supply a heartbeat and expect a heartbeat.
#[derive(Eq, PartialEq, Debug, Clone, Default)]
pub struct HeartBeatIntervalls {
    pub supplied: u32,
    pub expected: u32,
}

impl HeartBeatIntervalls {
    pub fn new(supplied: u32, expected: u32) -> HeartBeatIntervalls {
        HeartBeatIntervalls { expected, supplied }
    }
}

impl std::fmt::Display for HeartBeatIntervalls {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{},{}", &self.supplied, &self.expected)
    }
}

impl FromStr for HeartBeatIntervalls {
    type Err = StompParseError;
    /// Parses the string message as two ints representing "supplied, expected" heartbeat intervalls
    fn from_str(input: &str) -> Result<HeartBeatIntervalls, StompParseError> {
        split_once(input, ',')
            .ok_or_else(|| StompParseError::new(format!("Poorly formatted heartbeats: {}", input)))
            .and_then(|(supplied, expected)| {
                u32::from_str(expected)
                    .and_then(|expected| {
                        u32::from_str(supplied)
                            .map(|supplied| HeartBeatIntervalls { expected, supplied })
                    })
                    .map_err(|_| {
                        StompParseError::new(format!("Poorly formatted heartbeats: {}", input))
                    })
            })
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct StompVersions(pub Vec<StompVersion>);

impl std::fmt::Display for StompVersions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|version| version.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

impl FromStr for StompVersions {
    type Err = StompParseError;
    fn from_str(input: &str) -> Result<StompVersions, StompParseError> {
        input
            .split(',')
            .map(|section| StompVersion::from_str(section))
            .try_fold(Vec::new(), |mut vec, result| {
                result
                    .map(|version| {
                        vec.push(version);
                        vec
                    })
                    .map_err(|_| {
                        StompParseError::new(format!("Poorly formatted accept-versions: {}", input))
                    })
            })
            .map(StompVersions)
    }
}

impl std::ops::Deref for StompVersions {
    type Target = Vec<StompVersion>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
/// The Ack approach to be used for the subscription
pub enum AckType {
    /// The client need not send Acks. Messages are assumed received as soon as sent.
    Auto,
    /// Client must send Ack frames. Ack frames are cummulative, acknowledging also all previous messages.
    Client,
    /// Client must send Ack frames. Ack frames are individual, acknowledging only the specified message.
    ClientIndividual,
}

impl Default for AckType {
    fn default() -> Self {
        AckType::Auto
    }
}

impl std::fmt::Display for AckType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            AckType::Auto => "auto",
            AckType::Client => "client",
            AckType::ClientIndividual => "client-individual",
        })
    }
}

impl FromStr for AckType {
    type Err = StompParseError;
    fn from_str(input: &str) -> Result<AckType, StompParseError> {
        match input {
            "auto" => Ok(AckType::Auto),
            "client" => Ok(AckType::Client),
            "client-individual" => Ok(AckType::ClientIndividual),
            _ => Err(StompParseError::new(format!("Unknown ack-type: {}", input))),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq, Clone)]
/// Stomp Versions that client and server can negotiate to use
pub enum StompVersion {
    V1_0,
    V1_1,
    V1_2,
    Unknown(String),
}

impl std::fmt::Display for StompVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let text = match self {
            StompVersion::V1_0 => "1.0",
            StompVersion::V1_1 => "1.1",
            StompVersion::V1_2 => "1.2",
            _ => return Err(std::fmt::Error {}),
        };
        f.write_str(text)
    }
}

impl FromStr for StompVersion {
    type Err = StompParseError;
    fn from_str(input: &str) -> Result<StompVersion, StompParseError> {
        match input {
            "1.0" => Ok(StompVersion::V1_0),
            "1.1" => Ok(StompVersion::V1_1),
            "1.2" => Ok(StompVersion::V1_2),
            _ => Ok(StompVersion::Unknown(input.to_owned())),
        }
    }
}

const EMPTY: &str = "";

headers!(
    (Ack, "ack", AckType, (AckType::Auto)),
    (
        AcceptVersion,
        "accept-version",
        StompVersions,
        (StompVersions(Vec::new()))
    ),
    (ContentLength, "content-length", u32, 0),
    (ContentType, "content-type"),
    (Destination, "destination"),
    (
        HeartBeat,
        "heart-beat",
        HeartBeatIntervalls,
        (HeartBeatIntervalls::new(0, 0))
    ),
    (Host, "host"),
    (Id, "id"),
    (Login, "login"),
    (Message, "message"),
    (MessageId, "message-id"),
    (Passcode, "passcode"),
    (Receipt, "receipt"),
    (ReceiptId, "receipt-id"),
    (Server, "server"),
    (Session, "session"),
    (Subscription, "subscription"),
    (Transaction, "transaction"),
    (Version, "version", StompVersion, (StompVersion::V1_2))
);

#[cfg(test)]
mod test {
    use crate::common::functions::decode_str;
    use crate::error::StompParseError;
    use crate::headers::{HeartBeatIntervalls, HeartBeatValue};
    use either::Either;

    use std::{fmt::Display, str::FromStr};

    use super::{ContentLengthValue, DecodableValue, DestinationValue, HeaderValue};

    fn do_something(value: &str) {
        println!("Value: {}", value);
    }

    #[test]
    fn header_value() {
        let d = DestinationValue::new("Foo");

        let value: &str = d.value();

        do_something(value);

        drop(d);

        //        println!("Value: {}", value);
    }

    #[test]
    fn header_value_display() {
        let x = ContentLengthValue::new(10);

        assert_eq!("content-length:10", x.to_string())
    }

    #[test]
    fn heartbeat_reads_supplied_then_expected() {
        let hb = HeartBeatIntervalls::from_str("100,200").expect("Heartbeat parse failed");

        assert_eq!(100, hb.supplied);
        assert_eq!(200, hb.expected);
    }

    #[test]
    fn heartbeat_writes_supplied_then_expected() {
        let hb = HeartBeatIntervalls::new(500, 300);

        assert_eq!("500,300", hb.to_string());
    }

    #[test]
    fn heartbeat_into_intervalls() {
        let hb = HeartBeatValue::new(HeartBeatIntervalls::new(123, 987));

        let intervalls: HeartBeatIntervalls = hb.into();

        assert_eq!(123, intervalls.supplied);
        assert_eq!(987, intervalls.expected);
    }

    struct TestValue {
        value: &'static str,
    }

    impl TestValue {
        fn value(&self) -> &str {
            self.value
        }
    }

    impl DecodableValue for TestValue {
        fn decoded_value(&self) -> Result<Either<&str, String>, StompParseError> {
            decode_str(self.value())
        }
    }

    impl Display for TestValue {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_fmt(format_args!("test:{}", self.value))
        }
    }

    impl HeaderValue for TestValue {
        type OwnedValue = String;
        type Value = &'static str;
        const OWNED: bool = false;

        fn header_name(&self) -> &str {
            todo!()
        }
    }

    #[test]
    fn returns_value_if_no_escape() {
        let value = "Hello";
        let instance = TestValue { value };

        let result = instance.decoded_value();

        if let Ok(Either::Left(result)) = result {
            assert_eq!(value.as_ptr(), result.as_ptr());
        } else {
            panic!("Unexpected return");
        }
    }

    #[test]
    fn transforms_escaped_slash() {
        let value = "Hel\\\\lo";
        let instance = TestValue { value };

        let result = instance.decoded_value();

        if let Ok(Either::Right(result)) = result {
            assert_eq!("Hel\\lo", &result);
        } else {
            panic!("Unexpected return");
        }
    }

    #[test]
    fn transforms_escaped_n() {
        let value = "Hell\\nno";
        let instance = TestValue { value };

        let result = instance.decoded_value();

        if let Ok(Either::Right(result)) = result {
            assert_eq!("Hell\nno", &result);
        } else {
            panic!("Unexpected return");
        }
    }

    #[test]
    fn transforms_escaped_r() {
        let value = "Hell\\rno";
        let instance = TestValue { value };

        let result = instance.decoded_value();

        if let Ok(Either::Right(result)) = result {
            assert_eq!("Hell\rno", &result);
        } else {
            panic!("Unexpected return");
        }
    }

    #[test]
    fn transforms_escaped_c() {
        let value = "Hell\\cno";
        let instance = TestValue { value };

        let result = instance.decoded_value();

        if let Ok(Either::Right(result)) = result {
            assert_eq!("Hell:no", &result);
        } else {
            panic!("Unexpected return");
        }
    }

    #[test]
    fn rejects_escaped_t() {
        let value = "Hell\\tno";
        let instance = TestValue { value };

        let result = instance.decoded_value();

        if let Ok(_) = result {
            panic!("Unexpected return");
        }
    }

    #[test]
    fn rejects_slash_at_end() {
        let value = "Hell\\";
        let instance = TestValue { value };

        let result = instance.decoded_value();

        if let Ok(_) = result {
            panic!("Unexpected return");
        }
    }
}
