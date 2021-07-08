macro_rules! sender_frame {
    ( $name:ident,  $($comment:literal,)? $command:ident, $origin:ident $(, $header_name:ident : $header_type:ident )* $(,( $(  $opt_header_name:ident : $opt_header_type:ident $(: $opt_header_default:tt $(: $opt_header_default_comment:literal)?)?  ),* ))? $(,[custom: $has_custom:ident])? $(,[body: $has_body:ident])?  $(,$long_comment:literal)*) => {

        paste::paste! {
            $(#[doc = ""$comment]
            #[doc = ""])?
            #[doc = "This frame has required headers "$("`"$header_name"`")","* $(" and optional headers " $("`"$opt_header_name"`")","* )?"."]
            $(#[doc = ""]
            #[doc = ""$long_comment])?
            pub struct [<$name Builder>] {
                $(
                    $header_name: Option<<[<$header_type Value>]<'static> as HeaderValue<'static>>::OwnedValue>,
                )*
                $($(
                    $opt_header_name: Option<<[<$opt_header_type Value>]<'static> as HeaderValue<'static>>::OwnedValue>,
                )*)?
                $(
                    #[doc(hidden)]
                    #[doc = "Useseless doc: `"$has_custom"`."]
                    custom: Vec<(String, String)>,
                )?
                $(
                    #[doc(hidden)]
                    #[doc = "Useseless doc: `"$has_body"`."]
                    body: Option<Vec<u8>>,
                )?
            }

            impl [<$name Builder>] {
                $(
                    #[doc = "The value of the `"$header_name"` header."]
                    pub fn $header_name<'a,'b>(&'b mut self, new_val: <[<$header_type Value>]<'a> as HeaderValue<'a>>::OwnedValue) -> &'b mut [<$name Builder>] {

                        self.$header_name = Some(new_val);

                        self
                    }
                )*
                $($(
                    #[doc = "The value of the `"$opt_header_name"` header."]
                    $($(#[doc = "Defaults to `"$opt_header_default_comment"` if not supplied."])?)?
                    pub fn $opt_header_name<'a>(&'a mut self, new_val: <[<$opt_header_type Value>]<'a> as HeaderValue<'a>>::OwnedValue) -> &'a mut [<$name Builder>] {
                        self.$opt_header_name = Some(new_val);

                        self
                    }
                )*)?
                $(
                    #[doc = "Useseless doc: `"$has_custom"`."]
                    pub fn add_custom_header<'a>(&'a mut self, name: String, value: String) -> &'a mut [<$name Builder>] {
                        self.custom.push((name, value));
                        self
                    }
                )?
                $(
                    #[doc = "Useseless doc: `"$has_body"`."]
                    pub fn body<'a>(&'a mut self, new_value: Vec<u8>) -> &'a mut [<$name Builder>] {
                        self.body = Some(new_value);
                        self
                    }
                )?

                pub fn new() -> [<$name Builder>] {
                    [<$name Builder>] {
                        $(
                            $header_name: Option::<<[<$header_type Value>]<'static> as HeaderValue<'static>>::OwnedValue>::None,
                        )*
                        $($(
                            $opt_header_name: choose_from_presence!($($opt_header_default)? {Some($($opt_header_default)?().into())},{None}),
                        )*)?
                        $(
                            custom: choose_from_presence!($has_custom {Vec::new()}, {Vec::new()}),
                        )?
                        $(
                            body: choose_from_presence!($has_body None, None),
                        )?
                    }
                }

                pub fn build(mut self) -> Result<$name<'static>, StompParseError> {
                    // First, build the byte array
                    let mut bytes : Vec<u8> = Vec::with_capacity(1000);
                    let bytes_ref = &mut bytes;

                    let mut frame = $name::init(Vec::new());

                    write_command(bytes_ref, $name::NAME);


                    $(
                        // Write the required header, returning an error if the value was not set
                        let (_,[<$header_name _range>]) = if [<$header_type Value>]::OWNED {
                            // Owned values are already in the right form for the frame, but also need to be written to the
                            // output buffer
                            self.[<$header_name>].take().map(|value| {
                                let mut bytes = value.to_string().into_bytes();
                                frame.$header_name = [<$header_type Value>]::from_owned(value);
                                write_header(bytes_ref, [<$header_type Value>]::NAME, &mut bytes)
                            }).ok_or(StompParseError::new(format!("Required header {} not set.", ([<$header_type Value>]::NAME))))?
                        } else {
                            // Non-owned values strings; the value for the header on the frame needs to be in the byte buffer
                            self.[<$header_name>].take().map(|value| {
                                let mut bytes = value.to_string().into_bytes();
                                write_header(bytes_ref, [<$header_type Value>]::NAME, &mut bytes)
                            }).ok_or(StompParseError::new(format!("Required header {} not set.", ([<$header_type Value>]::NAME))))?
                        };
                    )*

                    $($(
                        // Write the required header, returning an error if the value was not set
                        let [<$opt_header_name _range>] = if [<$opt_header_type Value>]::OWNED {
                            // Owned values are already in the right form for the frame, but also need to be written to the
                            // output buffer
                            self.[<$opt_header_name>].take().map(|value| {
                                let mut bytes = value.to_string().into_bytes();
                                 choose_from_presence!($($opt_header_default)? {
                                    frame.$opt_header_name = [<$opt_header_type Value>]::from_owned(value);
                                }, {
                                    frame.$opt_header_name = Some([<$opt_header_type Value>]::from_owned(value));
                                });
                                write_header(bytes_ref, [<$opt_header_type Value>]::NAME, &mut bytes)
                            })
                        } else {
                            // Non-owned values strings; the value for the header on the frame needs to be in the byte buffer
                            self.[<$opt_header_name>].take().map(|value| {
                                let mut bytes = value.to_string().into_bytes();
                                write_header(bytes_ref, [<$opt_header_type Value>]::NAME, &mut bytes)
                            })
                        };
                    )*)?

                    $(
                        let $has_custom : Vec<((usize, usize),(usize,usize))> = self.custom.iter().map(|(name, value)| {
                             let mut bytes = value.to_string().into_bytes();
                             write_header(bytes_ref, &name, &mut bytes)
                        }).collect();
                    )?

                    // End the headers
                    write_headers_end(bytes_ref);

                    $(
                    let mut [<_ $has_body>] = ();

                    if let None = self.body {
                        return Err(StompParseError::new("Body not set."));
                    }

                    let body_range = self.body.take().as_mut().map(|body| write_body(bytes_ref, body));
                    )?

                    // End the frame
                    write_frame_end(bytes_ref);

                    let ptr : *const [u8] = bytes.as_slice();
                    let slice = unsafe { ptr.as_ref().unwrap() };

                    frame.raw = bytes;

                    $(
                        if ![<$header_type Value>]::OWNED {
                            let value = unsafe { std::str::from_utf8_unchecked(&slice[[<$header_name _range>].0..[<$header_name _range>].1]) };
                            frame.$header_name = [<$header_type Value>]::from_str(value).expect("Should never fail because string valued");
                        }
                    )*

                    $($(
                        if let Some((_,[<$opt_header_name _range>])) = [<$opt_header_name _range>] {
                            if ![<$opt_header_type Value>]::OWNED {
                                let value = unsafe { std::str::from_utf8_unchecked(&slice[[<$opt_header_name _range>].0..[<$opt_header_name _range>].1]) };
                                choose_from_presence!($($opt_header_default)? {
                                    frame.$opt_header_name = [<$opt_header_type Value>]::from_str(value).expect("Should never fail because string valued");
                                }, {
                                    frame.$opt_header_name = Some([<$opt_header_type Value>]::from_str(value).expect("Should never fail because string valued"));
                                });
                            }
                        };
                    )*)?

                    $(
                        frame.custom = $has_custom.iter().map(|ranges| {
                            let name = unsafe { std::str::from_utf8_unchecked(&slice[ranges.0.0..ranges.0.1]) };
                            let value = unsafe { std::str::from_utf8_unchecked(&slice[ranges.1.0..ranges.1.1]) };

                            CustomValue::new(name, value)
                        }).collect();
                    )?


                    $(
                        [<_ $has_body>] = ();
                        body_range.iter().for_each(|body_range|{
                            frame.body = &slice[body_range.0..body_range.1]
                        });

                    )?

                    Ok(frame)
                }
            }
        }
    }
}
