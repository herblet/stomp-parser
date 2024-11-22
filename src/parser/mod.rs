pub mod headers;

use nom::bytes::complete::is_not;
use nom::character::complete::{char, line_ending};
use nom::combinator::eof;
use nom::error::context;
use nom::sequence::terminated;
use nom::{IResult, Needed};

use crate::error::FullError;
use crate::error::StompParseError;

pub fn null<'a, E: 'a + FullError<&'a [u8], StompParseError>>(
    input: &'a [u8],
) -> IResult<&[u8], &'a [u8], E> {
    terminated(
        context("Null Octet", char('\x00')),
        context("Data after null", eof),
    )(input)
    .map(|(rem, _)| (rem, &input[input.len() - 1..]))
}

pub fn command_line<'a, E: FullError<&'a [u8], E2>, E2>(
    input: &'a [u8],
) -> IResult<&[u8], &'a [u8], E> {
    terminated(is_not("\r\n"), line_ending)(input)
}

pub fn remaining_without_null<'a, E: FullError<&'a [u8], StompParseError>>(
    input: &'a [u8],
) -> IResult<&'a [u8], &'a [u8], E> {
    match input.split_last() {
        Some((&0u8, bytes)) => Ok((&bytes[0..0], bytes)),
        _ => Err(nom::Err::Incomplete(Needed::Unknown)),
    }
}

#[cfg(test)]
mod tests {
    use crate::client::ClientFrame;
    use crate::headers::{AckType, HeartBeatIntervalls, StompVersion, StompVersions};
    use std::convert::TryFrom;

    #[test]
    fn it_recognises_connect_frames() {
        let frame = ClientFrame::try_from(
            "CONNECT\naccept-version:1.1,1.2,funk\nhost:b\r\n\n\u{00}"
                .as_bytes()
                .to_owned(),
        );

        match frame.unwrap() {
            ClientFrame::Connect(frame) => {
                assert_eq!(
                    StompVersions(vec![
                        StompVersion::V1_1,
                        StompVersion::V1_2,
                        StompVersion::Unknown("funk".to_string())
                    ]),
                    *frame.accept_version().value()
                );
                assert_eq!("b", frame.host().value());
                assert_eq!(None, frame.login());
                assert_eq!(None, frame.passcode());
            }
            _ => panic!("Not a Connect Frame!"),
        }
    }

    #[test]
    fn it_accepts_optional_connect_frame_headers() {
        let frame = ClientFrame::try_from(
            "CONNECT\naccept-version:1.1,1.2,\
            funk\nhost:b\r\nlogin:slarti\npasscode:bartfast\n\n\u{00}"
                .as_bytes()
                .to_owned(),
        );
        match frame.unwrap() {
            ClientFrame::Connect(frame) => {
                assert_eq!("slarti", frame.login().unwrap().value());
                assert_eq!("bartfast", frame.passcode().unwrap().value());
            }
            _ => panic!("Not a Connect Frame!"),
        }
    }

    #[test]
    fn it_fails_if_host_missing() {
        let frame = ClientFrame::try_from(
            "CONNECT\naccept-version:a\nlogin:foo\r\nheart-beat:10,20\r\n\
            \nasakldkf\u{00}"
                .as_bytes()
                .to_owned(),
        );

        assert!(matches!(frame, Err(_)));
    }

    #[test]
    fn it_accepts_heartbeat() {
        let frame = ClientFrame::try_from(
            "CONNECT\naccept-version:a\nhost:b\r\nlogin:foo\r\nheart-beat:10,20\r\n\
            \n\u{00}"
                .as_bytes()
                .to_owned(),
        )
        .unwrap();
        if let ClientFrame::Connect(frame) = frame {
            assert_eq!(
                HeartBeatIntervalls {
                    supplied: 10,
                    expected: 20,
                },
                *frame.heartbeat().value()
            );
        } else {
            panic!("Not a connect frame!")
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn CONNECT_frame_rejects_body() {
        let frame =
            ClientFrame::try_from(b"CONNECT\naccept-version:a\nhost:b\r\n\nfoobar\x00".to_vec());
        assert!(frame.is_err());
    }

    #[test]
    #[allow(non_snake_case)]
    fn CONNECT_frame_rejects_data_after_null() {
        let frame =
            ClientFrame::try_from(b"CONNECT\naccept-version:a\nhost:b\r\n\n\x00foobar".to_vec());
        assert!(frame.is_err());
    }

    #[test]
    fn it_is_case_sensitive() {
        let frame = ClientFrame::try_from(
            b"coNNect\naccept-version:a\nhost:b\r\n\n
        \x00"
                .to_vec(),
        );
        assert!(frame.is_err());
    }

    #[test]
    fn it_doesnt_match_rubbish() {
        let frame =
            ClientFrame::try_from(b"coNct\naccept-version:a\nhost:b\r\n\nasakldkf\x00".to_vec());
        assert!(frame.is_err());
    }

    #[test]
    #[allow(non_snake_case)]
    fn STOMP_frame_works() {
        let frame =
            ClientFrame::try_from(b"STOMP\naccept-version:a\nhost:b\r\n\n\x00".to_vec()).unwrap();
        assert!(matches!(frame, ClientFrame::Connect(_)));
    }

    #[test]
    #[allow(non_snake_case)]
    fn SUBSCRIBE_frame_accepted() {
        let frame = ClientFrame::try_from(
            b"SUBSCRIBE\r\ndestination:y/b\nid:1\nack:client\n\n\x00".to_vec(),
        )
        .unwrap();

        if let ClientFrame::Subscribe(frame) = frame {
            assert_eq!("y/b", frame.destination().value());
            assert_eq!("1", frame.id().value());
            assert_eq!(AckType::Client, *frame.ack_type().value())
        } else {
            panic!("Not a SUBSCRIBE");
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn SUBSCRIBE_frame_requires_id() {
        let frame =
            ClientFrame::try_from(b"SUBSCRIBE\r\ndestination:y/b\nack:client\n\n\x00".to_vec());
        assert!(frame.is_err())
    }

    #[test]
    #[allow(non_snake_case)]
    fn SUBSCRIBE_frame_requires_destination() {
        let frame = ClientFrame::try_from(b"SUBSCRIBE\r\nid:1\nack:client\n\n\x00".to_vec());
        assert!(frame.is_err())
    }

    #[test]
    #[allow(non_snake_case)]
    fn SUBSCRIBE_frame_accepts_ack() {
        let frame = ClientFrame::try_from(
            b"SUBSCRIBE\r\ndestination:y/b\nid:1\nack:client-individual\n\n\x00".to_vec(),
        )
        .unwrap();

        if let ClientFrame::Subscribe(frame) = frame {
            assert_eq!(AckType::ClientIndividual, *frame.ack_type().value())
        } else {
            panic!("Not a SUBSCRIBE");
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn SUBSCRIBE_frame_disallows_body() {
        let frame = ClientFrame::try_from(
            b"SUBSCRIBE\r\ndestination:y/b\nid:1\nack:client-individual\n\nsdsd\x00".to_vec(),
        );

        assert!(frame.is_err())
    }

    #[test]
    #[allow(non_snake_case)]
    fn it_accepts_SEND_frame() {
        let frame =
            ClientFrame::try_from(b"SEND\ndestination:foo\r\n\nhello,world\x00".to_vec()).unwrap();

        if let ClientFrame::Send(frame) = frame {
            assert_eq!(
                "hello,world",
                std::str::from_utf8(frame.body().unwrap()).unwrap()
            );
        } else {
            panic!("Not a send");
        }
    }
}
