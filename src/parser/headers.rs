use core::ops::FnMut;

use nom::bytes::complete::{escaped, is_not};
use nom::character::complete::{char, line_ending, one_of};
use nom::combinator::{flat_map, map_res};
use nom::error::context;
use nom::multi::many0;
use nom::sequence::terminated;
use nom::IResult;
use nom::Parser;

use crate::error::{FullError, StompParseError};
use crate::model::headers::parser::*;
use crate::model::headers::*;

trait HeaderParser<'a, E: FullError<&'a [u8], StompParseError>>:
    FnMut(&'a [u8]) -> IResult<&'a [u8], Header, E> + 'a
{
}

impl<'a, E, T> HeaderParser<'a, E> for T
where
    E: FullError<&'a [u8], StompParseError>,
    T: FnMut(&'a [u8]) -> IResult<&'a [u8], Header, E> + 'a,
{
}

/// Creates an new HeadersParser accepting the specified required and optional Headers,
/// and optionally arbitrary other headers as "custom" headers.
pub fn headers_parser<'a, E>(
    required: Vec<HeaderType>,
    optional: Vec<HeaderType>,
    allows_custom: bool,
) -> Box<dyn Parser<&'a [u8], Vec<Header>, E> + 'a>
where
    E: 'a + FullError<&'a [u8], StompParseError>,
{
    let parser_selector = init_headers_parser(required, optional, allows_custom);

    Box::new(terminated(
        many0(flat_map(header_name, parser_selector)), // Accept many headers...
        context("header_terminator", line_ending),     //...terminated by a blank line
    ))
}

fn init_headers_parser<'a, E>(
    required: Vec<HeaderType>,
    optional: Vec<HeaderType>,
    allows_custom: bool,
) -> Box<dyn Fn(&'a str) -> Box<dyn HeaderParser<'a, E>> + 'a>
where
    E: 'a + FullError<&'a [u8], StompParseError>,
{
    // The part that deals with the specified required and optional headers
    let known_headers = init_known_header_parser(required, optional, allows_custom);

    // The part that deals with any other headers encountered
    //let custom_header_parser_provider = custom_header_parser_provider_factory(allows_custom);
    Box::new(move |name: &'a str| {
        // Determine the type
        known_headers(name) // Then see if it is a known header, and return the appropriate parser
            .unwrap_or_else(|_| disallowed_header_parser(name))
    })
}

fn find_header<'a, 'b, E>(
    name: &'a str,
    required: &'b Vec<HeaderType>,
    optional: &'b Vec<HeaderType>,
    allows_custom: bool,
) -> Result<Box<dyn HeaderParser<'a, E> + 'a>, StompParseError>
where
    'a: 'b,
    E: 'a + FullError<&'a [u8], StompParseError>,
{
    required
        .iter()
        .find(|header_type| header_type.matches(name))
        .or_else(|| {
            optional
                .iter()
                .find(|header_type| header_type.matches(name))
        })
        .map(|header_type| {
            Ok(known_header_parser::<'a, E>(find_header_parser(
                *header_type,
            )))
        })
        .unwrap_or_else(|| {
            if allows_custom {
                Ok(known_header_parser::<'a, E>(Box::new(
                    move |value: &str| {
                        Ok(Header::Custom(CustomValue::new(
                            unsafe { std::mem::transmute::<&'a str, &'static str>(name) },
                            unsafe { std::mem::transmute::<&'_ str, &'static str>(value) },
                        )))
                    },
                )))
            } else {
                Err(StompParseError::new(format!("Unknown header: {}", name)))
            }
        })
}

fn init_known_header_parser<'a, E>(
    required: Vec<HeaderType>,
    optional: Vec<HeaderType>,
    allows_custom: bool,
) -> impl Fn(&'a str) -> Result<Box<dyn HeaderParser<'a, E>>, StompParseError> + 'a
where
    E: 'a + FullError<&'a [u8], StompParseError>,
{
    move |name: &'a str| find_header(name, &required, &optional, allows_custom)
}

fn header_section<'a, E: FullError<&'a [u8], StompParseError>>(
    input: &'a [u8],
) -> IResult<&'a [u8], &'a [u8], E> {
    escaped(is_not("\\:\n\r"), '\\', one_of("rnc\\"))(input)
}

fn into_string<'a>(input: &'a [u8]) -> Result<&'a str, StompParseError> {
    std::str::from_utf8(input).map_err(|_| StompParseError::new("bytes are not utf8"))
}

fn header_name<'a, E: FullError<&'a [u8], StompParseError>>(
    input: &'a [u8],
) -> IResult<&'a [u8], &'a str, E> {
    context(
        "header name",
        map_res(terminated(header_section, char(':')), into_string),
    )(input)
}

fn header_value<'a, E: FullError<&'a [u8], StompParseError>>(
    input: &'a [u8],
) -> IResult<&'a [u8], &'a str, E> {
    context(
        "header value",
        map_res(terminated(header_section, line_ending), into_string),
    )(input)
}

fn disallowed_header_parser<'a, E: 'a + FullError<&'a [u8], StompParseError>>(
    name: &'a str,
) -> Box<dyn HeaderParser<'a, E>> {
    Box::new(map_res(header_value, move |_| {
        Err(StompParseError::new(format!(
            "Unexpected header '{}' encountered",
            name
        )))
    }))
}

fn known_header_parser<'a, E: 'a + FullError<&'a [u8], StompParseError>>(
    parser: Box<dyn Fn(&str) -> Result<Header, StompParseError> + 'a>,
) -> Box<dyn HeaderParser<'a, E>> {
    Box::new(map_res(header_value, parser))
}

#[cfg(test)]
mod tests {
    use either::Either;
    use nom::error::dbg_dmp;
    use nom::error::VerboseError;

    use super::headers_parser;
    use crate::error::{FullError, StompParseError};
    use crate::model::headers::*;
    use nom::IResult;
    use std::vec::Vec;

    fn header<E: 'static + FullError<&'static [u8], StompParseError> + std::fmt::Debug>(
        input: &'static [u8],
    ) -> IResult<&'static [u8], Header, E> {
        headers(input).map(|x| {
            let bytes = x.0;
            let mut vec = x.1;
            (bytes, vec.pop().unwrap())
        })
    }
    fn headers<E: 'static + FullError<&'static [u8], StompParseError> + std::fmt::Debug>(
        input: &'static [u8],
    ) -> IResult<&'static [u8], Vec<Header>, E> {
        dbg_dmp(
            |input| {
                headers_parser(
                    Vec::new(),
                    vec![
                        HeaderType::HeartBeat,
                        HeaderType::Destination,
                        HeaderType::Host,
                    ],
                    true,
                )
                .parse(input)
            },
            "header_line",
        )(input)
    }

    fn headers_no_custom<
        E: 'static + FullError<&'static [u8], StompParseError> + std::fmt::Debug,
    >(
        input: &'static [u8],
    ) -> IResult<&'static [u8], Vec<Header>, E> {
        dbg_dmp(
            |input| {
                headers_parser(
                    Vec::new(),
                    vec![
                        HeaderType::HeartBeat,
                        HeaderType::Destination,
                        HeaderType::Host,
                    ],
                    false,
                )
                .parse(input)
            },
            "header_line",
        )(input)
    }
    fn assert_custom_header(
        input: &'static str,
        expected_key: &'static str,
        expected_value: &'static str,
        expected_decoded_key: Option<&'static str>,
        expected_decoded_value: Option<&'static str>,
    ) {
        let result = headers::<VerboseError<&'static [u8]>>(input.as_bytes())
            .unwrap()
            .1;

        if let Header::Custom(value) = &result[0] {
            assert_eq!(expected_key, value.header_name());
            check_raw_and_decoded(
                expected_key,
                value.header_name(),
                expected_decoded_key,
                value.decoded_name(),
            );
            check_raw_and_decoded(
                expected_value,
                value.value(),
                expected_decoded_value,
                value.decoded_value(),
            );
        } else {
            panic!("Expected custom header");
        }
    }

    fn check_raw_and_decoded(
        expected_value: &str,
        actual_value: &str,
        expected_decoded_value: Option<&str>,
        actual_decoded: Result<Either<&str, String>, StompParseError>,
    ) {
        assert_eq!(expected_value, actual_value);
        if let Some(expected_decoded_value) = expected_decoded_value {
            match actual_decoded {
                Ok(Either::Left(val)) => {
                    assert_eq!(expected_decoded_value, val);
                }
                Ok(Either::Right(val)) => {
                    assert_eq!(expected_decoded_value, &val);
                }
                Err(_) => {
                    panic!("Decode failed!")
                }
            }
        }
    }

    #[test]
    fn header_line_terminated_by_rn() {
        assert_custom_header("abc:def\r\n\n", "abc", "def", Some("abc"), Some("def"));
    }

    #[test]
    fn header_line_terminated_by_n() {
        assert_custom_header("abc:def\n\n", "abc", "def", Some("abc"), None);
    }

    #[test]
    fn header_with_cr_fails() {
        let result = dbg_dmp(header::<VerboseError<&[u8]>>, "header_line")(b"ab\rc:def\n");

        assert!(result.is_err());
    }

    #[test]
    fn header_with_nl_fails() {
        let result = dbg_dmp(header::<VerboseError<&[u8]>>, "header_line")(b"ab\nc:def\n");

        assert!(result.is_err());
    }

    #[test]
    fn header_with_colon_fails() {
        let result = dbg_dmp(header::<VerboseError<&[u8]>>, "header_line")(b"abc:d:ef\n");

        assert!(result.is_err());
    }

    use nom::bytes::complete::{escaped, is_not};
    use nom::character::complete::one_of;

    fn esc(input: &[u8]) -> IResult<&[u8], &[u8]> {
        escaped(is_not("\\:\n\r"), '\\', one_of("rn:\\"))(input)
    }

    #[test]
    fn test_escaped() {
        let (_, matched) = esc(b"a\\rbc:def\n\n" as &[u8]).expect("Should be fine");

        assert_eq!(b"a\\rbc", matched)
    }

    #[test]
    fn header_accepts_escaped_cr() {
        assert_custom_header("a\\rbc:def\n\n", "a\\rbc", "def", Some("a\rbc"), None);
    }

    #[test]
    fn header_line_accepts_escaped_nl() {
        assert_custom_header(
            "abc:d\\nef\n\n",
            "abc",
            "d\\nef",
            Some("abc"),
            Some("d\nef"),
        );
    }

    #[test]
    fn header_line_accepts_escaped_colon() {
        assert_custom_header("abc:d\\cef\n\n", "abc", "d\\cef", None, Some("d:ef"));
    }

    #[test]
    fn header_accepts_fwd_slash() {
        assert_custom_header("abc:d\\\\ef\n\n", "abc", "d\\\\ef", None, Some("d\\ef"));
    }

    #[test]
    fn header_rejects_escaped_tab() {
        let result = dbg_dmp(header::<VerboseError<&[u8]>>, "header_line")(b"abc:d\\tef\n\n");

        assert!(result.is_err());
    }

    #[test]
    fn header_works_for_custom() {
        assert_custom_header(
            "a\\rbc:d\\\\ef\n\n",
            "a\\rbc",
            "d\\\\ef",
            Some("a\rbc"),
            Some("d\\ef"),
        );
    }

    #[test]
    fn header_works_for_host() {
        let header = dbg_dmp(header::<VerboseError<&[u8]>>, "header_line")(b"host:d\\nef\n\n")
            .unwrap()
            .1;

        if let Header::Host(value) = header {
            assert_eq!("d\\nef", value.value());
        } else {
            panic!("Expected host header");
        }
    }

    #[test]
    fn header_works_for_heart_beat() {
        let header = dbg_dmp(header::<VerboseError<&[u8]>>, "header_line")(b"heart-beat:10,20\n\n")
            .unwrap()
            .1;

        if let Header::HeartBeat(value) = header {
            assert_eq!(
                HeartBeatIntervalls {
                    supplied: 10,
                    expected: 20,
                },
                *value.value()
            );
        } else {
            panic!("Expected heart-beat header");
        }
    }

    #[test]
    fn header_is_case_sensitive() {
        //heart-beat not recognised
        assert_custom_header(
            "heArt-beat:10,20\n\n",
            "heArt-beat",
            "10,20",
            Some("heArt-beat"),
            None,
        );
    }

    #[test]
    fn headers_works_for_no_headers() {
        let headers = dbg_dmp(headers::<VerboseError<&[u8]>>, "headers")(b"\n\n")
            .unwrap()
            .1;

        assert_eq!(0, headers.len());
    }

    #[test]
    fn headers_works_for_single_header() {
        let headers = dbg_dmp(headers::<VerboseError<&[u8]>>, "headers")(b"heart-beat:10,20\n\n")
            .unwrap()
            .1;

        assert_eq!(1, headers.len());
        assert_eq!(
            Header::HeartBeat(HeartBeatValue::new(HeartBeatIntervalls {
                supplied: 10,
                expected: 20,
            })),
            headers[0]
        );
    }

    #[test]
    fn headers_works_for_multiple_headers() {
        let headers = dbg_dmp(headers::<VerboseError<&[u8]>>, "headers")(
            b"heart-beat:10,20\r\nabc:d\\nef\n\n",
        )
        .unwrap()
        .1;

        assert_eq!(2, headers.len());
        assert_eq!(
            Header::HeartBeat(HeartBeatValue::new(HeartBeatIntervalls {
                supplied: 10,
                expected: 20,
            })),
            headers[0]
        );
        assert_eq!(
            Header::Custom(CustomValue::new("abc", "d\\nef")),
            headers[1]
        );
    }

    #[test]
    fn headers_rejects_custom_when_disallowed() {
        let result = dbg_dmp(headers_no_custom::<VerboseError<&[u8]>>, "headers")(
            b"heart-beat:10,20\r\nabc:d\\nef\n\n",
        );

        assert_eq!(true, result.is_err());
    }

    #[test]
    fn headers_fails_when_no_empty_line() {
        let headers =
            dbg_dmp(headers::<VerboseError<&[u8]>>, "headers")(b"heart-beat:10,20\r\nabc:d\\nef\n");

        assert!(headers.is_err());
    }
}
