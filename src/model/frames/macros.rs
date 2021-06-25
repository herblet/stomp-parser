#![doc(hidden)]
/// This macro is useful for forcing repeat expression - particularly optional
/// items - without actually outputting anything depending on the input.
macro_rules! blank {
    ($in:ident) => {};
}

macro_rules! true_if_present {
    ($in:ident) => {
        true
    };

    () => {
        false
    };
}

macro_rules! choose_from_presence {
    ($in:tt $present:tt, $absent:tt) => {
        $present
    };

    ($present:tt, $absent:tt) => {
        $absent
    };
}

macro_rules! frame {
    ( $name:ident,  $($comment:literal,)? $command:ident, $origin:ident $(, $header_name:ident : $header_type:ident )* $(,( $(  $opt_header_name:ident : $opt_header_type:ident $(: $opt_header_default:tt $(: $opt_header_default_comment:literal)?)?  ),* ))? $(,[custom: $has_custom:ident])? $(,[body: $has_body:ident])?  $(,$long_comment:literal)*) => {
        paste::paste! {
            $(#[doc = ""$comment]
            #[doc = ""])?
            #[doc = "This frame has required headers "$("`"$header_name"`")","* $(" and optional headers " $("`"$opt_header_name"`")","* )?"."]
            $(#[doc = ""]
            #[doc = ""$long_comment])?
            pub struct $name {
            $(
                #[doc = "The value of the `"$header_name"` header."]
                pub $header_name: [<$header_type Value>],
            )*
            $($(
                #[doc = "The value of the `"$opt_header_name"` header."]
                $($(#[doc = "Defaults to `"$opt_header_default_comment"` if not supplied."])?)?
                pub $opt_header_name: choose_from_presence!($($opt_header_default)? [<$opt_header_type Value>],(Option<[<$opt_header_type Value>]>)),
            )*)?
            $(
                #[allow(unused)]
                $has_custom: (),
                pub custom: Vec<CustomValue>,
            )?
            $(
                #[allow(unused)]
                $has_body: (),
                body_offset_length: (isize, usize),
                raw: Option<Vec<u8>>,
            )?

            // prevents construction by external code
            #[allow(unused)]
            dummy_private: (),
        }

        impl $name {
            const NAME: &'static str = stringify!($command);
            #[doc = "Creates a new" $name"."]
            fn from_parsed( $(
                $header_name: [<$header_type Value>],
            )* $($(
                $opt_header_name: Option<[<$opt_header_type Value>]>,
            )*)? $(
                $has_custom: Vec<CustomValue>,
            )? $(
                $has_body: (isize, usize),
            )?
         )  -> Self {
                $name {
                    $(
                        $header_name,
                    )*
                    $($(
                        $opt_header_name: choose_from_presence!($(($opt_header_default))? ($opt_header_name.unwrap_or_else($($opt_header_default)?)),($opt_header_name)),
                    )*)?
                    $(
                        $has_custom: (),
                        custom: $has_custom,
                    )?
                    $(
                        $has_body: (),
                        body_offset_length: $has_body,
                        raw: None,
                    )?

                    dummy_private: ()
                }

            }

            pub fn new( $(
                $header_name: [<$header_type Value>],
            )* $($(
                $opt_header_name: Option<[<$opt_header_type Value>]>,
            )*)? $(
                $has_custom: Vec<CustomValue>,
            )? $(
                $has_body: Vec<u8>,
            )?
         )  -> Self {
                $name {
                    $(
                        $header_name,
                    )*
                    $($(
                        $opt_header_name: choose_from_presence!($(($opt_header_default))? ($opt_header_name.unwrap_or_else($($opt_header_default)?)),($opt_header_name)),
                    )*)?
                    $(
                        $has_custom: (),
                        custom: $has_custom,
                    )?
                    $(
                        $has_body: (),
                        body_offset_length: (0,$has_body.len()),
                        raw: Some($has_body),
                    )?

                    dummy_private: ()
                }

            }
                $(
                     blank!($has_body);
                pub fn body(&self) -> Option<&[u8]> {
                    select_slice(&self.raw, &self.body_offset_length)
                }
            )?
        }

        $(blank!($has_body);
        impl crate::parser::HasBody for $name {
                /// Sets the vector containing the bytes of the body
                fn set_raw(&mut self, bytes: Vec<u8>) {
                    self.raw = Some(bytes);
                }
            }
        )?

        impl TryInto<Vec<u8>> for $name {
            type Error = StompParseError;

            fn try_into(self) -> Result<Vec<u8>, Self::Error> {
                {
                    let mut result = Vec::new();

                    // STOMP Command
                    writeln(&mut result, Self::NAME)?;

                    // Required Headers
                    $( writeln(&mut result, self.$header_name)?; )*

                    // Optional Headers
                    $($(
                        choose_from_presence!($($opt_header_default)? { writeln(&mut result, self.$opt_header_name)?; },{self.$opt_header_name.as_ref().map_or(Ok(()),|value| writeln(&mut result, value))?;});
                    )*)?

                    // End of Headers
                    result.write(b"\n")?;


                    $(
                        blank!($has_body);
                        select_slice(&self.raw,&self.body_offset_length)
                            .map_or(Ok(0),|body|result.write(body))?;
                    )?

                    // end of frame
                    result.push(0u8);

                    Ok::<Vec<u8>, std::io::Error>(result)
                }.map_err(StompParseError::from)
            }
        }

        impl std::fmt::Display for $name {
             fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                writeln!(f, "{}", Self::NAME)?;
                $( writeln!(f, "{}",  self.$header_name)?; )*

                $($(
                    choose_from_presence!($($opt_header_default)? { writeln!(f, "{}",  self.$opt_header_name)?; },{self.$opt_header_name.as_ref().map_or(Ok(()),|value| writeln!(f, "{}",  value))?;});
                )*)?
                f.write_str("\n")?; // End of headers
                $(
                    self.$has_body;
                    self.body().map_or(Ok(()),|value| f.write_str(  unsafe {
                        std::str::from_utf8_unchecked(value)
                    }))?;
                )?
                f.write_str("\u{00}") // End of frame
            }
        }

        }
    }
}

macro_rules! frame_parser {
    ( $name:ident, $origin:ident $(, $header_name:ident : $header_type:ident )* $(,( $(  $opt_header_name:ident : $opt_header_type:ident ),* ))? $(,[custom: $has_custom:ident])? $(,[body: $has_body:ident])? ) => {
        paste::paste! {

            #[allow(unused)]
            pub fn [<$name:lower _frame>]<'a, E: 'a + FullError<&'a [u8], StompParseError>>(
                base_offset: *const u8,
            ) -> Box<dyn Parser<&'a [u8], [<$origin Frame>], E>> {
                    Box::new({
                        move |input: &'a [u8]| {
                        let headers_parser = headers_parser(
                                vec![$(
                            HeaderType::$header_type,
                        )*],
                        vec![$($(
                            HeaderType::$opt_header_type,
                        )*)?],
                        true_if_present!(
                        $(
                            $has_custom
                        )?)
                            );

                        let body_section = if true_if_present!($($has_body)?) {
                            remaining_without_null
                        } else {
                            null
                        };
                        let mut fnmut = context(
                            stringify!([<$name _frame>]),
                            map_res(tuple((headers_parser, body_section)), |x| {
                                let headers = x.0;
                                $(
                                    let mut $header_name: Option<[<$header_type Value>]> = None;
                                )*
                                $($(
                                    let mut $opt_header_name: Option<[<$opt_header_type Value>]> = None;
                                )*)?
                                $(
                                    let mut $has_custom = Vec::new();
                                )?

                                for header in headers {
                                    match header {
                                        $(
                                        Header::$header_type(val) => { $header_name = Some(val); }
                                        )*
                                        $($(
                                        Header::$opt_header_type(val) => { $opt_header_name = Some(val); }
                                        )*)?
                                        $(
                                        Header::Custom(val)=> { $has_custom.push(val); }
                                        )?
                                        _ => {return Err(StompParseError::new(format!("Unexpected header: {:?}",header)));}
                                    }
                                }

                                $(
                                let $has_body =
                                    unsafe { (x.1.as_ptr().offset_from(base_offset), x.1.len()) };
                                    )?

                                Ok([<$origin Frame>]::$name([<$name Frame>]::from_parsed(
                                    $(
                                        $header_name.ok_or_else(|| StompParseError::new(format!("Missing required header of type: {:?}",HeaderType::$header_type)))?,
                                    )*
                                    $($(
                                        $opt_header_name,
                                    )*)?
                                    $(
                                        $has_custom,
                                    )?
                                    $(
                                        $has_body,
                                    )?
                                )))
                            }),
                        );

                        let res = fnmut(input);
                        drop(fnmut);
                        res
                    }
                })
            }
        }
    };
}

macro_rules! frames {
    { $group_name:ident,
        $(
            ( $name:ident, $($comment:literal,)? $command:ident$(|$alias:ident)*, $origin:ident $(, $header_name:ident : $header_type:ident )* $(,( $(  $opt_header_name:ident : $opt_header_type:ident $(: $opt_header_default:tt$(: $opt_header_default_comment:literal)?)?),* ))? $(,[custom: $has_custom:ident])? $(,[body: $has_body:ident])? $(,$long_comment:literal)* )
        ),+
    } => {
        use crate::error::StompParseError;
        use crate::model::frames::utils::*;

        use std::convert::{TryFrom, TryInto};
        use std::io::Write;

        paste::paste! {
            $(
                frame! (
                    [<$name Frame>],
                    $($comment,)?
                    $command,
                    $group_name
                    $(, $header_name : $header_type )*
                    $(,( $(  $opt_header_name : $opt_header_type $(: $opt_header_default $(: $opt_header_default_comment)?)? ),* ))?
                    $(,[custom: $has_custom])?
                    $(,[body: $has_body])?
                    $(,$long_comment)?
                );
            )+

            #[doc = "The `" $group_name "Frame` enum contains a variant for each frame that the "$group_name:lower" can send."]
            #[doc = ""]
            #[doc = "The `try_from(bytes: Vec<u8>)` method, provided via an implementaton of `TryFrom<Vec<u8>>`, is the recommended way to obtain a Frame from a received message."]
            pub enum [<$group_name Frame>] {
                $(
                    $(#[doc=$comment])?
                    $name([<$name Frame>])
                ),+
            }

            impl crate::parser::HasBody for [<$group_name Frame>] {
                fn set_raw(&mut self, bytes: Vec<u8>) {
                    match self {
                        $(
                            $([<$group_name Frame>]::$name(frame) => { blank!($has_body); frame.set_raw(bytes);})?
                        )+
                        _ => { /* Frames with no body do nothing */ }
                    }
                }
            }

            impl TryInto<Vec<u8>> for [<$group_name Frame>] {
                type Error = StompParseError;

                fn try_into(self) -> Result<Vec<u8>, <Self as TryInto<Vec<u8>>>::Error> {
                    match self {
                        $(
                            [<$group_name Frame>]::$name(frame) => frame.try_into(),
                        )+
                    }
                }
            }

            impl std::fmt::Display for [<$group_name Frame>] {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                    match self {
                        $(
                            [<$group_name Frame>]::$name(frame) => frame.fmt(f),
                        )+
                    }
                }
            }

            #[doc = "Parses a `" $group_name "Frame`  from the data contained in the provided vector of bytes."]
            impl TryFrom<Vec<u8>> for [<$group_name Frame>]{
                        type Error = StompParseError;
                        fn try_from(bytes: Vec<u8>) -> Result<Self, StompParseError> {
                            self::parsers::[<$group_name:lower _frame>](bytes)
                         }
            }

            mod parsers {
                use super::*;
                use crate::parser::headers::headers_parser;
                use crate::parser::{null,remaining_without_null, Switch, command_line, always_fail, HasBody};
                use crate::error::FullError;
                use crate::error::StompParseError;
                use nom::combinator::map_res;
                use nom::error::context;
                use nom::error::VerboseError;
                use nom::sequence::tuple;
                use nom::{IResult, Parser};
                 $(
                    frame_parser! (
                        $name,
                        $group_name
                        $(, $header_name : $header_type )*
                        $(,( $(  $opt_header_name : $opt_header_type ),* ))?
                        $(,[custom: $has_custom])?
                        $(,[body: $has_body])?
                    );
                )+

                fn command<'a, E: 'a + FullError<&'a [u8], StompParseError>>(
                    input: &'a [u8],
                ) -> IResult<&'a [u8], [<$group_name Frame>], E> {
                    Switch::<'a, fn(&'a [u8]) -> IResult<&[u8], &[u8], E>, [<$group_name Frame>], E, StompParseError>::new(
                        command_line,
                        vec![
                            $(
                                (stringify!($command), [<$name:lower _frame>](input.as_ptr())),
                                $((stringify!($alias), [<$name:lower _frame>](input.as_ptr())),)*
                            )+
                        ],
                        Box::new(always_fail),
                    )
                    .parse(input)
                }

                /// The entry point to this package, which parses a frame in this group
                pub fn [<$group_name:lower _frame>]<'a, 'b>(input: Vec<u8>) -> Result<[<$group_name Frame>], StompParseError>
                where
                    'b: 'a,
                {
                    let parser = |input: &'b [u8]| command::<VerboseError<&'b [u8]>>(input);

                    let result = nom::dbg_dmp(parser, "frame")(input.as_slice());

                    match result {
                        Err(_) => Err(StompParseError::new("Error parsing frame")),
                        Ok((_, mut client_frame)) => {
                            client_frame.set_raw(input);
                            Ok(client_frame)
                        }
                    }
                }

            }

        }
    }
}
