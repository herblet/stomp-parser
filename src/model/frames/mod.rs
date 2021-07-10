#[macro_use]
mod sender;
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
            (message: Message),
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
            [custom: cus],
            [body: body]
        )
    }

    impl ErrorFrame {
        pub fn from_message(message: &str) -> Self {
            ErrorFrameBuilder::new()
                .message(message.to_owned())
                .build()
                .unwrap()
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
    use std::convert::TryInto;
    use std::thread;

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
        let frame = ConnectedFrameBuilder::new()
            .version(StompVersion::V1_1)
            .heartbeat(HeartBeatIntervalls {
                supplied: 20,
                expected: 10,
            })
            .build()
            .unwrap();

        let displayed = frame.to_string();

        assert_eq!(
            "CONNECTED\nversion:1.1\nheart-beat:20,10\n\n\u{00}",
            displayed
        );
    }

    #[test]
    fn writes_message_frame() {
        let body = b"Lorem ipsum dolor sit amet,".to_vec();

        let frame = MessageFrameBuilder::new()
            .message_id("msg-1".to_owned())
            .destination("path/to/hell".to_owned())
            .subscription("annual".to_owned())
            .content_type("foo/bar".to_owned())
            .body(body)
            .build()
            .expect("Should be ok");

        assert_message_frame_roundtrip(
            frame,
            "msg-1",
            "path/to/hell",
            "annual",
            Some("foo/bar"),
            None,
            &vec![],
            Some(b"Lorem ipsum dolor sit amet,"),
        );
    }

    #[test]
    fn writes_custom_headers() {
        let body = b"Lorem ipsum dolor sit amet,".to_vec();

        let frame = MessageFrameBuilder::new()
            .message_id("msg-1".to_owned())
            .destination("path/to/hell".to_owned())
            .subscription("annual".to_owned())
            .content_type("foo/bar".to_owned())
            .add_custom_header("hello".to_owned(), "world".to_owned())
            .body(body)
            .build()
            .expect("Should be ok");

        assert_message_frame_roundtrip(
            frame,
            "msg-1",
            "path/to/hell",
            "annual",
            Some("foo/bar"),
            None,
            &vec![("hello", "world")],
            Some(b"Lorem ipsum dolor sit amet,"),
        );
    }

    fn assert_message_frame_roundtrip(
        frame: MessageFrame,
        expected_id: &str,
        expected_dest: &str,
        expected_sub: &str,
        expected_content_type: Option<&str>,
        expected_content_length: Option<u32>,
        expected_custom: &Vec<(&str, &str)>,
        expected_body: Option<&[u8]>,
    ) {
        assert_message_frame(
            &frame,
            expected_id,
            expected_dest,
            expected_sub,
            expected_content_type,
            expected_content_length,
            expected_custom,
            expected_body,
        );

        let bytes: Vec<u8> = frame.try_into().expect("Error writing bytes");

        if let Ok(ServerFrame::Message(frame)) = ServerFrame::try_from(bytes) {
            assert_message_frame(
                &frame,
                expected_id,
                expected_dest,
                expected_sub,
                expected_content_type,
                expected_content_length,
                expected_custom,
                expected_body,
            );
        } else {
            panic!("Should have received a Message frame")
        }
    }

    fn assert_message_frame(
        frame: &MessageFrame,
        expected_id: &str,
        expected_dest: &str,
        expected_sub: &str,
        expected_content_type: Option<&str>,
        expected_content_length: Option<u32>,
        expected_custom: &Vec<(&str, &str)>,
        expected_body: Option<&[u8]>,
    ) {
        assert_eq!(
            frame.message_id.value(),
            expected_id,
            "MessageId does not match"
        );
        assert_eq!(
            frame.destination.value(),
            expected_dest,
            "Destination does not match"
        );
        assert_eq!(
            frame.subscription.value(),
            expected_sub,
            "Subscription does not match"
        );
        assert_eq!(
            frame.content_type.as_ref().map(|value| value.value()),
            expected_content_type,
            "content-type does not match"
        );

        assert_eq!(
            frame.content_length.as_ref().map(|value| value.value()),
            expected_content_length.as_ref(),
            "content-length does not match"
        );
        expected_custom.iter().for_each(|(name, value)| {
            assert!(
                frame
                    .custom
                    .iter()
                    .any(|custom_value| custom_value.header_name() == *name
                        && custom_value.value() == value),
                "Missing custom value {}:{}",
                name,
                value
            );
        });

        assert_eq!(frame.body(), expected_body, "Body does not match");
    }

    #[test]
    fn writes_binary_message_frame() {
        let body = vec![0, 1, 1, 2, 3, 5, 8, 13];

        let frame = MessageFrameBuilder::new()
            .message_id("msg-1".to_owned())
            .destination("path/to/hell".to_owned())
            .subscription("annual".to_owned())
            .content_type("foo/bar".to_owned())
            .body(body)
            .build()
            .expect("Should be ok");

        assert_message_frame_roundtrip(
            frame,
            "msg-1",
            "path/to/hell",
            "annual",
            Some("foo/bar"),
            None,
            &vec![],
            Some(&[0, 1, 1, 2, 3, 5, 8, 13]),
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

    fn assert_in_range(ptr: *const u8, len: usize, actual: *const u8) {
        let offset = unsafe { actual.offset_from(ptr) };

        if offset < 0 || offset > (len as isize) {
            panic!("offset {} not in range of {}", offset, len);
        }
    }

    #[test]
    fn does_not_copy() {
        let message = b"SEND\n\
            destination:stairway/to/heaven\n\
            funky:doodle\n\
            \n\
            Lorem ipsum dolor sit amet,...\x00"
            .to_vec();

        let source_ptr = message.as_ptr();
        let source_len = message.len();

        if let Ok(ClientFrame::Send(frame)) = ClientFrame::try_from(message) {
            assert_in_range(source_ptr, source_len, frame.body().unwrap().as_ptr());
            assert_in_range(source_ptr, source_len, frame.destination.value().as_ptr());
            assert_in_range(source_ptr, source_len, frame.custom[0].value().as_ptr());
            assert_in_range(
                source_ptr,
                source_len,
                frame.custom[0].header_name().as_ptr(),
            );
        } else {
            panic!("Send Frame not parsed correctly");
        }
    }

    #[test]
    fn works_after_move() {
        let message = b"SEND\n\
            destination:stairway/to/heaven\n\
            \n\
            Lorem ipsum dolor sit amet,...\x00"
            .to_vec();

        let src_ptr = message.as_ptr() as u64;
        let len = message.len();
        let parsed = ClientFrame::try_from(message);

        let handle = thread::spawn(move || {
            if let Ok(ClientFrame::Send(frame)) = parsed {
                assert_eq!(
                    "Lorem ipsum dolor sit amet,...",
                    std::str::from_utf8(frame.body().unwrap()).unwrap()
                );

                assert_eq!("stairway/to/heaven", frame.destination.value());
                return frame.body().unwrap().as_ptr() as u64;
            } else {
                panic!("Send Frame not parsed correctly");
            }
        });

        if let Ok(address) = handle.join() {
            println!(
                "Source: {}, Len: {}, Offset: {} ",
                src_ptr,
                len,
                address - src_ptr,
            );
        } else {
            panic!("Error after move")
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
