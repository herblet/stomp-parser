//! Implements the model for headers, as specified in the
//! [STOMP Protocol Spezification,Version 1.2](https://stomp.github.io/stomp-specification-1.2.html).
#![allow(non_snake_case)]
#[macro_use]
mod macros;
use crate::error::StompParseError;
use paste::paste;
use std::str::FromStr;

/// A Header that reveals it's type and it's value, and can be displayed
pub trait HeaderValue<T>: std::fmt::Display {
    fn header_type(&self) -> HeaderType;
    fn header_name(&self) -> &str;
    fn value(&self) -> &T;
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
        write!(f, "{},{}", &self.expected, &self.supplied)
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

headers!(
    (Ack, "ack", AckType),
    (AcceptVersion, "accept-version", StompVersions),
    (ContentLength, "content-length", u32),
    (ContentType, "content-type", String),
    (Destination, "destination", String),
    (HeartBeat, "heart-beat", HeartBeatIntervalls),
    (Host, "host", String),
    (Id, "id", String),
    (Login, "login", String),
    (Message, "message", String),
    (MessageId, "message-id", String),
    (Passcode, "passcode", String),
    (Receipt, "receipt", String),
    (ReceiptId, "receipt-id", String),
    (Server, "server", String),
    (Session, "session", String),
    (Subscription, "subscription", String),
    (Transaction, "transaction", String),
    (Version, "version", StompVersion)
);

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::headers::HeartBeatIntervalls;

    use super::ContentLengthValue;

    #[test]
    fn header_value_display() {
        let x = ContentLengthValue::new(10);

        assert_eq!("content-length:10", x.to_string())
    }

    #[test]
    fn heartbeat_is_supplied_then_expected() {
        let hb = HeartBeatIntervalls::from_str("100,200").expect("Heartbeat parse failed");

        assert_eq!(100, hb.supplied);
        assert_eq!(200, hb.expected);
    }
}
