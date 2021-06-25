#[macro_use]
mod macros;
mod utils;
#[allow(non_snake_case)]
#[allow(unused_parens)]
#[allow(clippy::new_without_default)]
pub mod client {
    //! Implements the model for the frames that a STOMP client can send, as specified in
    //! the [STOMP Protocol Spezification,Version 1.2](https://stomp.github.io/stomp-specification-1.2.html).

    use crate::model::headers::*;

    frames! {
        Client,
        (
            Abort,
            "Aborts a transaction that has begun but not yet been committed.",
            ABORT,
            Client,
            transaction: Transaction
        ),
        (
            Ack,
            "Acknowledges a received message.",
            ACK,
            Client,
            id: Id,
            transaction: Transaction,
            (receipt: Receipt)
        ),
        (
            Begin,
            "Begins a transaction.",
            BEGIN,
            Client,
            transaction: Transaction,
            (receipt: Receipt)
        ),
        (
            Commit,
            "Commits a transaction.",
            COMMIT,
            Client,
            transaction: Transaction,
            (receipt: Receipt)
        ),
        (
            Connect,
            "Initiates a STOMP session.",
            CONNECT|STOMP,
            Client,
            host: Host,
            accept_version: AcceptVersion,
            (heartbeat: HeartBeat: (||HeartBeatValue::new(HeartBeatIntervalls::new(0,0))):"(0,0)",login: Login, passcode: Passcode),
            "See [CONNECT Frame](https://stomp.github.io/stomp-specification-1.2.html#CONNECT_or_STOMP_Frame)."
        ),
        (
            Disconnect,
            "Ends a STOMP session.",
            DISCONNECT,
            Client,
            receipt: Receipt
        ),
        (
            Nack,
            "Indicates that the client did not, or could not, process a message.",
            NACK,
            Client,
            id: Id,
            transaction: Transaction,
            (receipt: Receipt)
        ),
        (
            Send,
            "Sends a message to a specific destination.",
            SEND,
            Client,
            destination: Destination,
            (
                content_type: ContentType,
                content_length: ContentLength,
                transaction: Transaction,
                receipt: Receipt
            ),
            [custom: cus],
            [body: body]
        ),
        (
            Subscribe,
            "Subscribes to a specific destination.",
            SUBSCRIBE,
            Client,
            destination: Destination,
            id: Id,
            (
                ack_type: Ack: (||AckValue::new(AckType::Auto)):"Auto",
                receipt: Receipt
            ),
            [custom: cus]
        ),
        (
            Unsubscribe,
            "Cancels a specific subscription.",
            UNSUBSCRIBE,
            Client,
            id: Id,
            (receipt: Receipt)
        )
    }

    impl SendFrame {}

    #[cfg(test)]
    mod test {
        use crate::{headers::*, parser::HasBody};

        #[test]
        fn body_extracts_correct_slice() {
            let mut frame = super::SendFrame::from_parsed(
                DestinationValue::new("foo".to_owned()),
                None,
                None,
                None,
                None,
                Vec::default(),
                (2, 3),
            );
            frame.set_raw(vec![0, 1, 2, 3, 4, 5, 6, 7]);

            assert_eq!(b"\x02\x03\x04", frame.body().unwrap());
        }
    }
}

#[allow(non_snake_case)]
#[allow(unused_parens)]
#[allow(clippy::new_without_default)]
pub mod server {
    //! Implements the model for the frames that a STOMP server can send, as specified in the
    //! [STOMP Protocol Spezification,Version 1.2](https://stomp.github.io/stomp-specification-1.2.html).
    use crate::model::headers::*;
    frames! {
        Server,
        (
            Connected,
            CONNECTED,
            Server,
            version: Version,
            (
                heartbeat: HeartBeat,
                session: Session, server: Server
            )
        ),
        (
            Receipt,
            RECEIPT,
            Server,
            receipt_id: ReceiptId
        ),
        (
            Error,
            ERROR,
            Server,
            [custom: cus],
            [body: body]),
        (
            Message,
            MESSAGE,
            Server,
            message_id: MessageId,
            destination: Destination,
            subscription: Subscription,
            (
                content_type: ContentType,
                content_length: ContentLength
            ),
            [body: body]
        )
    }

    impl ErrorFrame {
        pub fn from_message(message: &str) -> Self {
            ErrorFrame::new(Vec::<CustomValue>::new(), message.as_bytes().to_owned())
        }
    }
}

#[cfg(test)]
#[macro_use]
mod test {
    use super::client::ClientFrame;
    use super::client::SendFrame;
    use super::server::*;
    use crate::model::headers::*;
    use std::convert::TryFrom;
    use std::convert::TryInto;

    #[test]
    fn parses_stomp_frame() {
        let result = ClientFrame::try_from(
            "STOMP\nhost:foo\naccept-version:1.1\nheart-beat:10,20\n\n\u{00}"
                .as_bytes()
                .to_owned(),
        );

        if let Ok(ClientFrame::Connect(frame)) = result {
            assert_eq!(StompVersion::V1_1, frame.accept_version.value().0[0])
        } else {
            panic!("Expected a connect frame")
        }
    }

    #[test]
    fn writes_connected_frame() {
        let frame = ConnectedFrame::new(
            VersionValue::new(StompVersion::V1_1),
            Some(HeartBeatValue::new(HeartBeatIntervalls {
                supplied: 20,
                expected: 10,
            })),
            None,
            None,
        );

        let displayed = frame.to_string();

        assert_eq!(
            "CONNECTED\nversion:1.1\nheart-beat:20,10\n\n\u{00}",
            displayed
        );
    }

    #[test]
    fn writes_message_frame() {
        let body = b"Lorem ipsum dolor sit amet,".to_vec();

        let frame = MessageFrame::new(
            MessageIdValue::new("msg-1".to_owned()),
            DestinationValue::new("path/to/hell".to_owned()),
            SubscriptionValue::new("annual".to_owned()),
            Some(ContentTypeValue::new("foo/bar".to_owned())),
            None,
            body,
        );

        let displayed = frame.to_string();

        assert_eq!(
            "MESSAGE\n\
            message-id:msg-1\n\
            destination:path/to/hell\n\
            subscription:annual\n\
            content-type:foo/bar\n\
            \n\
            Lorem ipsum dolor sit amet,\u{00}",
            displayed
        );
    }

    #[test]
    fn writes_message_frame_bytes() {
        let body = b"Lorem ipsum dolor sit amet,".to_vec();

        let frame = MessageFrame::new(
            MessageIdValue::new("msg-1".to_owned()),
            DestinationValue::new("path/to/hell".to_owned()),
            SubscriptionValue::new("annual".to_owned()),
            Some(ContentTypeValue::new("foo/bar".to_owned())),
            None,
            body,
        );

        let bytes: Vec<u8> = frame.try_into().expect("Error writing bytes");

        assert_eq!(
            b"MESSAGE\n\
            message-id:msg-1\n\
            destination:path/to/hell\n\
            subscription:annual\n\
            content-type:foo/bar\n\
            \n\
            Lorem ipsum dolor sit amet,\x00",
            bytes.as_slice()
        );
    }

    #[test]
    fn writes_binary_message_frame() {
        let body = vec![0, 1, 1, 2, 3, 5, 8, 13];

        let frame = MessageFrame::new(
            MessageIdValue::new("msg-1".to_owned()),
            DestinationValue::new("path/to/hell".to_owned()),
            SubscriptionValue::new("annual".to_owned()),
            Some(ContentTypeValue::new("foo/bar".to_owned())),
            None,
            body,
        );

        let bytes: Vec<u8> = frame.try_into().expect("Error writing bytes");

        assert_eq!(
            b"MESSAGE\n\
            message-id:msg-1\n\
            destination:path/to/hell\n\
            subscription:annual\n\
            content-type:foo/bar\n\
            \n\
            \x00\x01\x01\x02\x03\x05\x08\x0d\
            \x00",
            bytes.as_slice()
        );
    }

    #[test]
    fn parses_send_frame() {
        let message = b"SEND\n\
            destination:stairway/to/heaven\n\
            \n\
            Lorem ipsum dolor sit amet,...\x00"
            .to_vec();

        if let Ok(ClientFrame::Send(frame)) = ClientFrame::try_from(message) {
            assert_eq!(
                "Lorem ipsum dolor sit amet,...",
                std::str::from_utf8(frame.body().unwrap()).unwrap()
            );
        } else {
            panic!("Send Frame not parsed correctly");
        }
    }

    #[test]
    fn parses_binary_send_frame() {
        let message = b"SEND\n\
            destination:stairway/to/heaven\n\
            \n\
            \x00\x01\x01\x02\x03\x05\x08\x0d\
            \x00"
            .to_vec();

        if let Ok(ClientFrame::Send(frame)) = ClientFrame::try_from(message) {
            assert_eq!(&[0u8, 1, 1, 2, 3, 5, 8, 13], frame.body().unwrap());
        } else {
            panic!("Send Frame not parsed correctly");
        }
    }
}
