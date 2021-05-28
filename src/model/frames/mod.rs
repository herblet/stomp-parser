#[macro_use]
mod macros;

#[allow(non_snake_case)]
#[allow(unused_parens)]
#[allow(clippy::new_without_default)]
/// The `client` module defines the model for the frames that a STOMP client can send, as specified in the [STOMP Protocol Spezification,Version 1.2](https://stomp.github.io/stomp-specification-1.2.html).
pub mod client {

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
            accepted_versions: AcceptVersion,
            (heartbeat: HeartBeat: (||HeartBeatValue::new(HeartBeatIntervalls::new(0,0))),login: Login, passcode: Passcode)
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
                ack_type: Ack: (||AckValue::new(AckType::Auto)),
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
}

#[allow(non_snake_case)]
#[allow(unused_parens)]
#[allow(clippy::new_without_default)]
pub mod server {
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
            let raw = message.as_bytes().to_owned();
            let mut frame = ErrorFrame::new(Vec::<CustomValue>::new(), (0, raw.len()));
            frame.set_raw(raw);
            frame
        }
    }
}

#[cfg(test)]
#[macro_use]
mod test {
    use super::client::ClientFrame;
    use super::server::*;
    use crate::model::headers::*;
    use std::convert::TryFrom;

    #[test]
    fn parses_stomp_frame() {
        let result = ClientFrame::try_from(
            "STOMP\nhost:foo\naccept-version:1.1\nheart-beat:10,20\n\n\u{00}"
                .as_bytes()
                .to_owned(),
        );

        if let Ok(ClientFrame::Connect(frame)) = result {
            assert_eq!(StompVersion::V1_1, frame.accepted_versions.value().0[0])
        } else {
            panic!("Expected a connect frame")
        }
    }

    #[test]
    fn writes_connected_frame() {
        let frame = ConnectedFrame::new(
            VersionValue::new(StompVersion::V1_1),
            Some(HeartBeatValue::new(HeartBeatIntervalls {
                expected: 10,
                supplied: 20,
            })),
            None,
            None,
        );

        let displayed = frame.to_string();

        assert_eq!(
            "CONNECTED\nversion:1.1\nheart-beat:10,20\n\n\u{00}",
            displayed
        );
    }

    #[test]
    fn writes_message_frame() {
        let body = b"Lorem ipsum dolor sit amet,".to_vec();

        let mut frame = MessageFrame::new(
            MessageIdValue::new("msg-1".to_owned()),
            DestinationValue::new("path/to/hell".to_owned()),
            SubscriptionValue::new("annual".to_owned()),
            Some(ContentTypeValue::new("foo/bar".to_owned())),
            None,
            (0, body.len()),
        );
        frame.set_raw(body);

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
}
