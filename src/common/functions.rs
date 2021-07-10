use either::Either;

use crate::error::StompParseError;

use super::constants::{HEADER_PARTS_SEPARATOR, LINE_SEPARATOR, TERMINATOR};

pub fn extend_from_vec(bytes: &mut Vec<u8>, extension: &mut Vec<u8>) -> (usize, usize) {
    let begin = bytes.len();
    bytes.append(extension);
    (begin, bytes.len())
}

pub fn extend_from_slice(bytes: &mut Vec<u8>, extension: &[u8]) -> (usize, usize) {
    let begin = bytes.len();
    bytes.extend_from_slice(extension);
    (begin, bytes.len())
}

pub fn extend_name_value(
    bytes: &mut Vec<u8>,
    name: &str,
    value: &mut Vec<u8>,
) -> ((usize, usize), (usize, usize)) {
    let name_range = extend_from_slice(bytes, name.as_bytes());
    extend_from_slice(bytes, HEADER_PARTS_SEPARATOR);
    let value_range = extend_from_vec(bytes, value);

    (name_range, value_range)
}

pub fn extend_name_value_line(
    bytes: &mut Vec<u8>,
    name: &str,
    value: &mut Vec<u8>,
) -> ((usize, usize), (usize, usize)) {
    let result = extend_name_value(bytes, name, value);
    extend_from_slice(bytes, LINE_SEPARATOR);
    result
}

pub fn write_command(bytes: &mut Vec<u8>, name: &str) {
    bytes.extend_from_slice(name.as_bytes());
    bytes.extend_from_slice(LINE_SEPARATOR);
}

pub fn write_header(
    bytes: &mut Vec<u8>,
    header_name: &str,
    value_bytes: &mut Vec<u8>,
) -> ((usize, usize), (usize, usize)) {
    extend_name_value_line(bytes, header_name, value_bytes)
}

pub fn write_headers_end(bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(LINE_SEPARATOR);
}

pub fn write_body(bytes: &mut Vec<u8>, body: &mut Vec<u8>) -> (usize, usize) {
    extend_from_vec(bytes, body)
}

pub fn write_frame_end(bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(TERMINATOR);
}

pub fn decode_escape_sequence(slice: &str) -> Result<char, StompParseError> {
    match slice {
        "\\\\" => Ok('\\'),
        "\\r" => Ok('\r'),
        "\\n" => Ok('\n'),
        "\\c" => Ok(':'),
        _ => Err(StompParseError::new(format!(
            "Unknown escape sequence: '{}'",
            slice
        ))),
    }
}

pub fn decode_str(raw: &str) -> Result<Either<&str, String>, StompParseError> {
    match raw.find('\\') {
        None => Ok(Either::Left(raw)),
        Some(index) => {
            let mut buffer = String::with_capacity(raw.len());
            decode_at_and_continue(&mut buffer, raw, index)?;
            Ok(Either::Right(buffer))
        }
    }
}

fn decode_at_and_continue(
    buffer: &mut String,
    slice: &str,
    index: usize,
) -> Result<(), StompParseError> {
    buffer.push_str(&slice[..index]);
    if index < slice.len() - 1 {
        buffer.push(decode_escape_sequence(&slice[index..index + 2])?);
        decode_slice(buffer, &slice[index + 2..])
    } else {
        Err(StompParseError::new("input ends with control character \\"))
    }
}

fn decode_slice(buffer: &mut String, slice: &str) -> Result<(), StompParseError> {
    match slice.find('\\') {
        None => {
            buffer.push_str(slice);
            Ok(())
        }
        Some(index) => decode_at_and_continue(buffer, slice, index),
    }
}

#[cfg(test)]
mod test {
    use crate::common::constants::*;
    use crate::common::functions::*;

    #[test]
    pub fn write_command_appends_and_separates() {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.append(&mut vec![1u8, 2, 3]);
        write_command(&mut bytes, "FUNK");

        assert_eq!(bytes.as_slice(), b"\x01\x02\x03FUNK\n");
    }

    #[test]
    pub fn write_header_appends_and_separates() {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.append(&mut vec![1u8, 2, 3]);
        write_header(&mut bytes, "FUNK", &mut b"hello".to_vec());

        assert_eq!(bytes.as_slice(), b"\x01\x02\x03FUNK:hello\n");
    }

    #[test]
    pub fn write_body_appends() {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.append(&mut vec![1u8, 2, 3]);
        write_body(&mut bytes, &mut b"Frankly, my dear,...".to_vec());

        assert_eq!(bytes.as_slice(), b"\x01\x02\x03Frankly, my dear,...");
    }

    #[test]
    pub fn write_frame_end_terminates() {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.append(&mut vec![1u8, 2, 3]);
        write_frame_end(&mut bytes);

        assert_eq!(bytes.as_slice(), b"\x01\x02\x03\x00");
    }

    #[test]
    pub fn extend_from_vec_consumes() {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.append(&mut vec![1, 2, 3]);

        let mut extension = vec![1, 2, 3, 4];

        let range = extend_from_vec(&mut bytes, &mut extension);

        assert_eq!(bytes, vec![1, 2, 3, 1, 2, 3, 4]);
        assert_eq!(extension, vec![]);
        assert_eq!((3, 7), range);
    }

    #[test]
    pub fn extend_from_slice_extends() {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.append(&mut vec![1, 2, 3]);

        let extension = vec![1, 2, 3, 4];

        let range = extend_from_slice(&mut bytes, &extension);

        assert_eq!(bytes, vec![1, 2, 3, 1, 2, 3, 4]);
        assert_eq!(extension, vec![1, 2, 3, 4]);
        assert_eq!((3, 7), range);
    }

    #[test]
    pub fn extend_name_value_extends() {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.append(&mut vec![1, 2, 3]);

        let mut extension = vec![1, 2, 3, 4];

        let name = "foobar";

        let (name_range, value_range) = extend_name_value(&mut bytes, name, &mut extension);

        let name_length = name.as_bytes().len();

        let expected_value_begin = 3 + name_length + HEADER_PARTS_SEPARATOR.len();
        let expected_value_end = expected_value_begin + 4;

        assert_eq!(extension, vec![]);
        assert_eq!((3, 3 + name_length), name_range);
        assert_eq!((expected_value_begin, expected_value_end), value_range);
        assert_eq!(expected_value_end, bytes.len());
    }

    #[test]
    pub fn extend_name_value_line_extends() {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.append(&mut vec![1, 2, 3]);

        let mut extension = vec![1, 2, 3, 4];

        let name = "foobar";

        let (name_range, value_range) = extend_name_value_line(&mut bytes, name, &mut extension);

        let name_length = name.as_bytes().len();

        let expected_value_begin = 3 + name_length + HEADER_PARTS_SEPARATOR.len();
        let expected_value_end = expected_value_begin + 4;

        assert_eq!(extension, vec![]);
        assert_eq!((3, 3 + name_length), name_range);
        assert_eq!((expected_value_begin, expected_value_end), value_range);
        assert_eq!(&bytes[expected_value_end..], LINE_SEPARATOR);
        assert_eq!(expected_value_end + LINE_SEPARATOR.len(), bytes.len());
    }
}
