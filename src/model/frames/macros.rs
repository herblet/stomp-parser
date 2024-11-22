macro_rules! frame {
    ( $name:ident,  $($comment:literal,)? $command:ident, $origin:ident $(, $header_name:ident : $header_type:ident )* $(,( $(  $opt_header_name:ident : $opt_header_type:ident $(: $opt_header_default:tt $(: $opt_header_default_comment:literal)?)?  ),* ))? $(,[custom: $has_custom:ident])? $(,[body: $has_body:ident])?  $(,$long_comment:literal)*) => {
        paste::paste! {

            sender_frame!($name,  $($comment,)? $command, $origin $(, $header_name : $header_type )* $(,( $(  $opt_header_name : $opt_header_type $(: $opt_header_default $(: $opt_header_default_comment )?)?  ),* ))? $(,[custom: $has_custom])? $(,[body: $has_body])?  $(,$long_comment)*);

            $(#[doc = ""$comment]
            #[doc = ""])?
            #[doc = "This frame has required headers "$("`"$header_name"`")","* $(" and optional headers " $("`"$opt_header_name"`")","* )?"."]
            $(#[doc = ""]
            #[doc = ""$long_comment])?
            pub struct $name<'a> {
                raw: Vec<u8>,
            $(
                #[doc = "The value of the `"$header_name"` header."]
                $header_name: [<$header_type Value>]<'a>,
            )*
            $($(
                #[doc = "The value of the `"$opt_header_name"` header."]
                $($(#[doc = "Defaults to `"$opt_header_default_comment"` if not supplied."])?)?
                $opt_header_name: choose_from_presence!($($opt_header_default)? ([<$opt_header_type Value>]<'a>),(Option<[<$opt_header_type Value>]<'a>>)),
            )*)?
            $(
                #[allow(unused)]
                $has_custom: (),
                pub custom: Vec<CustomValue>,
            )?
            $(
                #[allow(unused)]
                $has_body: &'a [u8],
            )?
        }

        impl <'a> $name<'a> {
            pub const NAME: &'static str = stringify!($command);
        }

        impl <'a> $name<'a> {

            fn init(raw: Vec<u8>) -> Self {
                $name {
                    raw,
                     $(
                $header_name: [<$header_type Value>]::default(),
            )*
                    $($(
                $opt_header_name: choose_from_presence!($(($opt_header_default))? ([<$opt_header_type Value>]::default()),None),
            )*)? $(
                 #[allow(unused)]
                $has_custom: (),
                custom: vec![],
            )? $(
                $has_body: &EMPTY,
            )?
            }
        }
                $(
                pub fn body(&self) -> Option<&'a [u8]> {
                    Some(self.$has_body)
                }
            )?

            $(
                #[doc = "The value of the `"$header_name"` header."]
                pub fn $header_name(&'a self) -> &'a [<$header_type Value>]<'a> {
                    &self.$header_name
                }
            )*
            $($(
                #[doc = "The value of the `"$opt_header_name"` header."]
                $($(#[doc = "Defaults to `"$opt_header_default_comment"` if not supplied."])?)?
                pub fn $opt_header_name(&'a self) -> choose_from_presence!($($opt_header_default)? (&'a [<$opt_header_type Value>]<'a>),(Option<&'a [<$opt_header_type Value>]<'a>>)) {
                    choose_from_presence!($($opt_header_default)? (&self.$opt_header_name),(self.$opt_header_name.as_ref()))
                }
            )*)?
        }

        #[doc = "This implementation serialises [`"$name"`] into a byte array."]
        impl <'a> Into<Vec<u8>> for $name<'a> {
            fn into(self) -> Vec<u8> {
                self.raw
            }
        }

        impl <'a> std::fmt::Debug for $name<'a> {
             fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                write!(f, "{}{{", Self::NAME)?;
                $(
                    write!(f, " {}: '{}', ", stringify!($header_name), self.$header_name.value())?;
                )*

                $($(
                    write!(f, " {}: '{}', ", stringify!($opt_header_name),
                    choose_from_presence!($( $opt_header_default )?
                    {self.$opt_header_name.value().to_string() },
                    { self.$opt_header_name.as_ref().map(|header|header.value().to_string()).unwrap_or("None".to_owned()) }))?;
                )*)?
                $(
                    self.$has_body;
                    write!(f, "body-length: {}", self.body().map(|body|body.len()).unwrap_or(0))?;
                )?

                f.write_str("}}\n")
            }
        }

        }
    }
}

macro_rules! frame_parser {
    ( $name:ident, $origin:ident $(, $header_name:ident : $header_type:ident )* $(,( $(  $opt_header_name:ident : $opt_header_type:ident $(: $opt_header_default:tt)?),* ))? $(,[custom: $has_custom:ident])? $(,[body: $has_body:ident])? ) => {
        paste::paste! {
            #[allow(unused)]
            pub fn [<$name:lower _frame>]<E: 'static + FullError<&'static [u8], StompParseError>>(
                mut frame: [<$name Frame>]<'static>
            ) -> Result<[<$origin Frame>], StompParseError>{

                let bytes : *const [u8] = frame.raw.as_slice();

                let input = unsafe { bytes.as_ref().unwrap() };

                let (input,_) = command_line::<VerboseError<&[u8]>, StompParseError>(input).map_err(|_|StompParseError::new("Error parsing frame"))?;

                        let headers_parser = headers_parser::<'static, E>(
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
                            remaining_without_null::<'static>
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

                                for header in headers {
                                    match header {
                                        $(
                                        Header::$header_type(val) => { $header_name = Some(val); }
                                        )*
                                        $($(
                                        Header::$opt_header_type(val) => { frame.$opt_header_name = choose_from_presence!( $($opt_header_default)? val, (Some(val))); }
                                        )*)?
                                        $(
                                        Header::Custom(val)=> {
                                            blank!($has_custom);
                                            frame.custom.push(val);
                                        }
                                        )?
                                        _ => {Err(StompParseError::new(format!("Unexpected header: {:?}",header)))?;}
                                    }
                                }


                                $(
                                    frame.$header_name = $header_name.ok_or_else(|| StompParseError::new(format!("Missing required header of type: {:?}",HeaderType::$header_type)))?;
                                )*

                                //         $has_custom,
                                //     )?
                                $(
                                frame.$has_body =  x.1;
                                )?

                                Ok(())
                            }
                        ));

                        fnmut(input).map_err(|_|StompParseError::new("Error parsing frame"))?;
                        drop(fnmut);
                        Ok([<$origin Frame>]::$name(frame))
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
        use crate::common::constants::*;
        use crate::common::functions::*;

        use crate::error::StompParseError;

        use std::convert::TryFrom;

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
                    $name([<$name Frame>]<'static>)
                ),+
            }

            #[doc = "This implementation serialises [`"$group_name Frame"`] into a byte array."]
            impl Into<Vec<u8>> for [<$group_name Frame>] {
                fn into(self) -> Vec<u8> {
                    match self {
                        $(
                            [<$group_name Frame>]::$name(frame) => frame.into(),
                        )+
                    }
                }
            }

            impl std::fmt::Debug for [<$group_name Frame>] {
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
                use crate::parser::{null,remaining_without_null, command_line};
                use crate::error::FullError;
                use crate::error::StompParseError;
                use nom::combinator::map_res;
                use nom::error::context;
                use nom::error::VerboseError;
                use nom::sequence::tuple;
                 $(
                    frame_parser! (
                        $name,
                        $group_name
                        $(, $header_name : $header_type )*
                        $(,( $(  $opt_header_name : $opt_header_type $(: $opt_header_default )? ),* ))?
                        $(,[custom: $has_custom])?
                        $(,[body: $has_body])?
                    );
                )+

                pub fn [<$group_name:lower _frame>](input: Vec<u8>) -> Result<[<$group_name Frame>], StompParseError>
                {
                    let slice = input.as_slice();

                    let (_,command_string) = command_line::<VerboseError<&[u8]>, StompParseError>(slice).map_err(|_|StompParseError::new("Error parsing frame"))?;

                    let initialiser: Box<dyn FnOnce(Vec<u8>)-> [<$group_name Frame>]> = std::str::from_utf8(command_string)
                        .map_err(|_|StompParseError::new("badly formed command string, not utf8"))
                        .and_then(move |command_string| match command_string {
                            $(

                                stringify!($command) => Ok(Box::new(|input|[<$group_name Frame>]::$name([<$name Frame>]::init(input))) as Box<dyn FnOnce(Vec<u8>)-> [<$group_name Frame>]>),
                                $(
                                    stringify!($alias) => Ok(Box::new(|input|[<$group_name Frame>]::$name([<$name Frame>]::init(input)))),
                                )*
                            )+
                            _ => Err(StompParseError::new(format!("Unknown command {}", command_string)))
                        })?;

                    let frame = initialiser(input);

                    match frame {
                        $(
                        [<$group_name Frame>]::$name(inner) =>  {
                            [<$name:lower _frame>]::<VerboseError<&[u8]>>(inner)
                        }
                        )+
                    }
                }

            }

        }
    }
}
