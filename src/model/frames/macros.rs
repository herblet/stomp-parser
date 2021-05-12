#[macro_export]
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

#[macro_export]
macro_rules! choose_from_presence {
    ($in:tt $present:tt, $absent:tt) => {
        $present
    };

    ($present:tt, $absent:tt) => {
        $absent
    };
}

#[macro_export]
macro_rules! frame {
    ( $name:ident, $command:ident, $origin:ident $(, $header_name:ident : $header_type:ident )* $(,( $(  $opt_header_name:ident : $opt_header_type:ident $(: $opt_header_default:tt)? ),* ))? $(,[custom: $has_custom:ident])? $(,[body: $has_body:ident])? ) => {
        paste::paste! {
            pub struct $name {
            $(
                pub $header_name: [<$header_type Value>],
            )*
            $($(
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
            pub fn new( $(
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

                /// _bytes may be unused, in which case it will be dropped
                pub fn set_raw(&mut self, _bytes: Vec<u8>) {
                    $(
                        blank!($has_body);
                        // $has_body
                    self.raw = Some(_bytes);
                    )?
                }
                $(
                     blank!($has_body);
                pub fn body(&self) -> Option<&[u8]> {
                    self.raw.as_ref().map(|vec| {
                        &vec[self.body_offset_length.0 as usize
                            ..(self.body_offset_length.0 as usize + self.body_offset_length.1)]
                    })
                }
            )?
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

#[macro_export]
macro_rules! frame_parser {
    ( $name:ident, $command:ident, $origin:ident $(, $header_name:ident : $header_type:ident )* $(,( $(  $opt_header_name:ident : $opt_header_type:ident ),* ))? $(,[custom: $has_custom:ident])? $(,[body: $has_body:ident])? ) => {
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

                                Ok([<$origin Frame>]::$name([<$name Frame>]::new(
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

#[macro_export]
macro_rules! frames {
    { $group_name:ident,
        $(
            ( $name:ident, $command:ident, $origin:ident $(, $header_name:ident : $header_type:ident )* $(,( $(  $opt_header_name:ident : $opt_header_type:ident $(: $opt_header_default:tt)?),* ))? $(,[custom: $has_custom:ident])? $(,[body: $has_body:ident])? )
        ),+
    } => {
        paste::paste! {
            $(
                frame! (
                    [<$name Frame>],
                    $command,
                    $group_name
                    $(, $header_name : $header_type )*
                    $(,( $(  $opt_header_name : $opt_header_type $(: $opt_header_default)? ),* ))?
                    $(,[custom: $has_custom])?
                    $(,[body: $has_body])?
                );
            )+

            pub enum [<$group_name Frame>] {
                $(
                    $name([<$name Frame>])
                ),+
            }

            impl [<$group_name Frame>] {
                pub fn set_raw(&mut self, bytes: Vec<u8>) {
                    match self {
                        $(
                            [<$group_name Frame>]::$name(frame) => {frame.set_raw(bytes);}
                        )+
                    }
                }
            }

            pub mod parsers {
                use super::*;
                use crate::headers::headers_parser;
                use crate::{null,remaining_without_null, Switch, command_line, always_fail};
                use crate::FullError;
                use crate::StompParseError;
                use nom::combinator::map_res;
                use nom::error::context;
                use nom::error::VerboseError;
                use nom::sequence::tuple;
                use nom::{IResult, Parser};
                 $(
                    frame_parser! (
                        $name,
                        $command,
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
