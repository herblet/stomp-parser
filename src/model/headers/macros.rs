macro_rules! header_display {
    ( ) => {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
            write!(f, "{}:{}", self.header_name(), self.value)
        }
    };
}
macro_rules! header {
    ( $header:ident, $name:expr $(,$types:ty $(, $default:expr )?)? ) => {
        paste! {

                #[derive(Eq, PartialEq, Clone)]
                pub struct [<$header Value>]<'a> {
                    value: or_else_type!($($types)?,&'a str),
                    phantom: std::marker::PhantomData<&'a or_else_type!($($types)?, str)>,
                }

                impl <'a> Default for [<$header Value>]<'a> {
                    fn default() -> Self {
                        [<$header Value>] {
                            value: or_else!($($($default)?)?,EMPTY),
                            phantom: std::marker::PhantomData
                        }
                    }
                }

                 impl <'a> [<$header Value>]<'a> {

                    pub const NAME: &'static str =  $name;

                    pub(crate) fn new(value: or_else_type!($($types)?,&'a str)) -> Self {
                        [<$header Value>] {
                            value,
                            phantom: std::marker::PhantomData
                        }
                    }

                    pub(crate) fn from_owned(_value: or_else_type!($($types)?,String)) -> Self {
                        choose_from_presence!($($types)? {
                            Self::new(_value)
                        }, {
                            panic!("Macro error, should never be called");
                        })
                    }

                    pub(crate) fn from_str<'b>(input: &'b str) -> Result<[<$header Value>]<'b>, StompParseError> {
                        choose_from_presence!($($types)? ($($types)?::from_str(input).map([<$header Value>]::new)
                            .map_err(|_| StompParseError::new("[<Error Parsing $header Value>]"))), (Ok([<$header Value>]::new(
                                input
                            ))))
                    }

                    pub fn value(&self) -> & or_else_type!($($types)?,str) {
                        choose_from_presence!($($types)? {&self.value}, {&self.value})
                    }
                }

                if_not_present!($($types)? (impl <'a> DecodableValue for [<$header Value>]<'a> {
                        fn decoded_value(&self) -> Result<Either<&str, String>, StompParseError> {
                            decode_str(self.value())
                        }
                    }
                ));

                impl <'a> HeaderValue  for [<$header Value>]<'a> {
                    type OwnedValue = or_else_type!($($types)?,String);
                    type Value=or_else_type!($($types)?,&'static str);
                    const OWNED: bool = choose_from_presence!($($types)? true, false);

                    fn header_name(&self) -> &str {
                        [<$header Value>]::NAME
                    }
                }

                impl  <'a> Into<or_else_type!($($types)?,&'a str)> for [<$header Value>]<'a> {
                    fn into(self) -> or_else_type!($($types)?,&'a str) {
                        self.value
                    }
                }

                impl <'a> std::fmt::Display for [<$header Value>]<'a> {
                    header_display!( );
                }

                impl <'a> std::fmt::Debug for [<$header Value>]<'a> {
                    header_display!( );
                }

        }
    };
}
macro_rules! headers {
        ( $( ($header:ident, $name:literal $(,$types:ty $(, $default:expr )?)? ) ),*  ) => {

             #[derive(Debug, Eq, PartialEq, Clone)]
            pub struct CustomValue {
                name: &'static str,
                value: &'static str
            }

             impl  CustomValue {
                pub fn new(name: &'static  str, value: &'static  str) -> Self {
                    CustomValue {
                        name,
                        value
                    }
                }

                pub fn value(&self) -> &&'static str {
                    &self.value
                }

                pub fn decoded_name(&self) -> Result<Either<&str, String>, StompParseError> {
                    decode_str(self.name)
                }
            }

            impl DecodableValue for CustomValue {
                fn decoded_value(&self) -> Result<Either<&str, String>, StompParseError> {
                    decode_str(self.value())
                }
            }

            impl  HeaderValue for CustomValue {
                type OwnedValue = String;
                type Value = &'static str;
                const OWNED: bool = false;

                fn header_name(&self) -> &str {
                    &self.name
                }
            }

             impl  std::fmt::Display for CustomValue {
                header_display!( );
            }


        #[derive(Debug, Eq, PartialEq, Copy, Clone)]
        pub enum HeaderType {
            $(
            $header
            ),*
//            ,Custom(&'static str)
        }

        impl HeaderType {
            pub fn matches(&self, name: &str) -> bool {
                match self {
                        $(
                            HeaderType::$header => name == $name,
                        )*
//                        HeaderType::Custom(header_name) => &name == header_name
                    }
            }
        }

        impl TryFrom<&'static str> for HeaderType {
            type Error = StompParseError;
            fn try_from(input: &'static str) -> std::result::Result<HeaderType, StompParseError> {
                match(input) {
                        $(
                            $name => Ok(HeaderType::$header),
                        )*
                        _ => panic!("Not a known Header")
//                        name => Ok(HeaderType::Custom(name))
                    }
            }
        }

         impl  std::fmt::Display for HeaderType {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                match(self) {
                    $(HeaderType::$header => {
                        formatter.write_str($name)
                    })*
                }
            }
        }


        paste! {
            $(
                header!($header, $name $(,$types $(, $default )?)? );
            )*

                #[derive(Debug, Eq, PartialEq, Clone)]
                pub enum Header<'a> {
                    $(
                    $header([<$header Value>]<'a>),
                    )*
                    Custom(CustomValue)
                }

                #[doc(hidden)]
                pub mod parser {
                    #![allow(non_snake_case)]

                    use super::*;
                    pub type HeaderValueConverter = dyn Fn(&str) -> Result<Header, StompParseError>;

                    pub fn find_header_parser(header_type: HeaderType) -> Box<HeaderValueConverter> {
                        match header_type {
                            $(
                                HeaderType::$header => Box::new([<parse_ $header _header>]),
                            )*
                        }
                    }

                    $(
                        pub fn [<parse_ $header _header>]<'a>(input: &'a str) -> Result<Header, StompParseError> {
                            [<$header Value>]::from_str(input).map(Header::$header)
                        }
                    )*

                }
        }
    }
}
