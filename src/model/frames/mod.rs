#[macro_use]
mod macros;

#[allow(non_snake_case)]
#[allow(unused_parens)]
#[allow(clippy::new_without_default)]
pub mod client {

    use crate::model::headers::*;

    frames! {
        Client,
        (
            Abort,
            ABORT,
            Client,
            transaction: Transaction
        ),
        (
            Ack,
            ACK,
            Client,
            id: Id,
            transaction: Transaction,
            (receipt: Receipt)
        ),
        (
            Begin,
            BEGIN,
            Client,
            transaction: Transaction,
            (receipt: Receipt)
        ),
        (
            Commit,
            COMMIT,
            Client,
            transaction: Transaction,
            (receipt: Receipt)
        ),
        (
            Connect,
            CONNECT,
            Client,
            host: Host,
            accepted_versions: AcceptVersion,
            (heartbeat: HeartBeat: (||HeartBeatValue::new(HeartBeatIntervalls::new(0,0))),login: Login, passcode: Passcode)
        ),
        (
            Disconnect,
            DISCONNECT,
            Client,
            receipt: Receipt
        ),
        (
            Nack,
            NACK,
            Client,
            id: Id,
            transaction: Transaction,
            (receipt: Receipt)
        ),
        (
            Send,
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
            Stomp,
            STOMP,
            Client,
            host: Host,
            accepted_versions: AcceptVersion,
            (heartbeat: HeartBeat: (||HeartBeatValue::new(HeartBeatIntervalls::new(0,0))),login: Login, passcode: Passcode)
        ),
        (
            Subscribe,
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
    use super::server::*;
    use crate::model::headers::*;
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
