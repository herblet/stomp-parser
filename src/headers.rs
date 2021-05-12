use core::ops::FnMut;

use nom::branch::alt;
use nom::bytes::complete::{escaped_transform, is_not, tag};
use nom::character::complete::{char, line_ending};
use nom::combinator::{flat_map, map_res, value};
use nom::error::context;
use nom::multi::many0;
use nom::sequence::terminated;
use nom::IResult;
use nom::Parser;
use std::str::FromStr;

use crate::model::headers::parser::*;
use crate::model::headers::*;
use crate::{FullError, StompParseError};

#[allow(type_alias_bounds)]
type HeaderParser<'a, E: 'static + FullError<&'a [u8], StompParseError>> =
    dyn FnMut(&'a [u8]) -> IResult<&'a [u8], Header, E> + 'a;

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
) -> Box<dyn Fn(String) -> Box<HeaderParser<'a, E>> + 'a>
where
    E: 'a + FullError<&'a [u8], StompParseError>,
{
    // The part that deals with the specified required and optional headers
    let known_headers = init_known_header_parser(required, optional, allows_custom);

    // The part that deals with any other headers encountered
    //let custom_header_parser_provider = custom_header_parser_provider_factory(allows_custom);
    Box::new(move |name: String| {
        HeaderType::from_str(&name) // Determine the type
            .and_then(&*known_headers) // Then see if it is a known header, and return the appropriate parser
            .unwrap_or_else(|_| disallowed_header_parser(name))
    })
}

fn init_known_header_parser<'a, E>(
    required: Vec<HeaderType>,
    optional: Vec<HeaderType>,
    allows_custom: bool,
) -> Box<dyn Fn(HeaderType) -> Result<Box<HeaderParser<'a, E>>, StompParseError>>
where
    E: 'a + FullError<&'a [u8], StompParseError>,
{
    Box::new(move |header_type| {
        // Check if its one of the known accepted headers
        if required.contains(&header_type) || optional.contains(&header_type) {
            Ok(known_header_parser::<'a, E>(find_header_parser(
                header_type,
            )))
        } else if allows_custom {
            // Otherwise, if custom is allowed, save it as a custom header
            Ok(known_header_parser::<'a, E>(find_header_parser(
                // This converts 'known-but-not-accepted' headers to custom ones
                HeaderType::Custom(header_type.to_string()),
            )))
        } else {
            Err(StompParseError::new(""))
        }
    })
}

fn header_section<'a, E: FullError<&'a [u8], StompParseError>>(
    input: &'a [u8],
) -> IResult<&'a [u8], Vec<u8>, E> {
    escaped_transform(
        is_not("\\:\n\r"),
        '\\',
        alt((
            value(b"\n" as &[u8], tag("n")),
            value(b"\r" as &[u8], tag("r")),
            value(b":" as &[u8], tag("c")),
            value(b"\\" as &[u8], tag("\\")),
        )),
    )(input)
}

fn into_string(input: Vec<u8>) -> Result<String, StompParseError> {
    String::from_utf8(input).map_err(|_| StompParseError::new("bytes are not utf8"))
}

fn header_name<'a, E: FullError<&'a [u8], StompParseError>>(
    input: &'a [u8],
) -> IResult<&'a [u8], String, E> {
    context(
        "header name",
        map_res(terminated(header_section, char(':')), into_string),
    )(input)
}

fn header_value<'a, E: FullError<&'a [u8], StompParseError>>(
    input: &'a [u8],
) -> IResult<&'a [u8], String, E> {
    context(
        "header value",
        map_res(terminated(header_section, line_ending), into_string),
    )(input)
}

fn disallowed_header_parser<'a, E: 'a + FullError<&'a [u8], StompParseError>>(
    name: String,
) -> Box<HeaderParser<'a, E>> {
    Box::new(map_res(header_value, move |_| {
        Err(StompParseError::new(format!(
            "Unexpected header '{}' encountered",
            name
        )))
    }))
}

fn known_header_parser<'a, E: 'a + FullError<&'a [u8], StompParseError>>(
    parser: Box<dyn Fn(String) -> Result<Header, StompParseError>>,
) -> Box<HeaderParser<'a, E>> {
    Box::new(map_res(header_value, parser))
}

#[cfg(test)]
mod tests {
    use nom::error::VerboseError;

    use super::headers_parser;
    use crate::model::headers::*;
    use crate::{FullError, StompParseError};
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
        nom::dbg_dmp(
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
        nom::dbg_dmp(
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
    ) {
        let result = headers::<VerboseError<&'static [u8]>>(input.as_bytes())
            .unwrap()
            .1;

        if let Header::Custom(value) = &result[0] {
            assert_eq!(*expected_key, *value.header_name());
            assert_eq!(*expected_value, *value.value());
        } else {
            panic!("Expected custom header");
        }
    }

    #[test]
    fn header_line_terminated_by_rn() {
        assert_custom_header("abc:def\r\n\n", "abc", "def");
    }

    #[test]
    fn header_line_terminated_by_n() {
        assert_custom_header("abc:def\n\n", "abc", "def");
    }

    #[test]
    fn header_with_cr_fails() {
        let result = nom::dbg_dmp(header::<VerboseError<&[u8]>>, "header_line")(b"ab\rc:def\n");

        assert!(result.is_err());
    }

    #[test]
    fn header_with_nl_fails() {
        let result = nom::dbg_dmp(header::<VerboseError<&[u8]>>, "header_line")(b"ab\nc:def\n");

        assert!(result.is_err());
    }

    #[test]
    fn header_with_colon_fails() {
        let result = nom::dbg_dmp(header::<VerboseError<&[u8]>>, "header_line")(b"abc:d:ef\n");

        assert!(result.is_err());
    }

    #[test]
    fn header_accepts_escaped_cr() {
        assert_custom_header("a\\rbc:def\n\n", "a\rbc", "def");
    }

    #[test]
    fn header_line_accepts_escaped_nl() {
        assert_custom_header("abc:d\\nef\n\n", "abc", "d\nef");
        assert_custom_header("abc:d\\nef\n\n", "abc", "d\nef");
    }

    #[test]
    fn header_line_accepts_escaped_colon() {
        assert_custom_header("abc:d\\cef\n\n", "abc", "d:ef");
    }

    #[test]
    fn header_accepts_fwd_slash() {
        assert_custom_header("abc:d\\\\ef\n\n", "abc", "d\\ef");
    }

    #[test]
    fn header_rejects_escaped_tab() {
        let result = nom::dbg_dmp(header::<VerboseError<&[u8]>>, "header_line")(b"abc:d\\tef\n\n");

        assert!(result.is_err());
    }

    #[test]
    fn header_works_for_custom() {
        assert_custom_header("a\\rbc:d\\\\ef\n\n", "a\rbc", "d\\ef");
    }

    #[test]
    fn header_works_for_host() {
        let header = nom::dbg_dmp(header::<VerboseError<&[u8]>>, "header_line")(b"host:d\\nef\n\n")
            .unwrap()
            .1;

        if let Header::Host(value) = header {
            assert_eq!("d\nef", value.value());
        } else {
            panic!("Expected host header");
        }
    }

    #[test]
    fn header_works_for_heart_beat() {
        let header =
            nom::dbg_dmp(header::<VerboseError<&[u8]>>, "header_line")(b"heart-beat:10,20\n\n")
                .unwrap()
                .1;

        if let Header::HeartBeat(value) = header {
            assert_eq!(
                HeartBeatIntervalls {
                    expected: 10,
                    supplied: 20
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
        assert_custom_header("heArt-beat:10,20\n\n", "heArt-beat", "10,20");
    }

    #[test]
    fn headers_works_for_no_headers() {
        let headers = nom::dbg_dmp(headers::<VerboseError<&[u8]>>, "headers")(b"\n\n")
            .unwrap()
            .1;

        assert_eq!(0, headers.len());
    }

    #[test]
    fn headers_works_for_single_header() {
        let headers =
            nom::dbg_dmp(headers::<VerboseError<&[u8]>>, "headers")(b"heart-beat:10,20\n\n")
                .unwrap()
                .1;

        assert_eq!(1, headers.len());
        assert_eq!(
            Header::HeartBeat(HeartBeatValue::new(HeartBeatIntervalls {
                expected: 10,
                supplied: 20
            })),
            headers[0]
        );
    }

    #[test]
    fn headers_works_for_multiple_headers() {
        let headers = nom::dbg_dmp(headers::<VerboseError<&[u8]>>, "headers")(
            b"heart-beat:10,20\r\nabc:d\\nef\n\n",
        )
        .unwrap()
        .1;

        assert_eq!(2, headers.len());
        assert_eq!(
            Header::HeartBeat(HeartBeatValue::new(HeartBeatIntervalls {
                expected: 10,
                supplied: 20
            })),
            headers[0]
        );
        assert_eq!(
            Header::Custom(CustomValue::new("abc".to_string(), "d\nef".to_string())),
            headers[1]
        );
    }

    #[test]
    fn headers_rejects_custom_when_disallowed() {
        let result = nom::dbg_dmp(headers_no_custom::<VerboseError<&[u8]>>, "headers")(
            b"heart-beat:10,20\r\nabc:d\\nef\n\n",
        );

        assert_eq!(true, result.is_err());
    }

    #[test]
    fn headers_fails_when_no_empty_line() {
        let headers = nom::dbg_dmp(headers::<VerboseError<&[u8]>>, "headers")(
            b"heart-beat:10,20\r\nabc:d\\nef\n",
        );

        assert!(headers.is_err());
    }
}
