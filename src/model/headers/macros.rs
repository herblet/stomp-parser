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

                #[derive(Debug, Eq, PartialEq, Clone)]
                pub struct [<$header Value>]<'a> {
                    value: or_else_type!($($types)?,&'a str),
                    _lt: PhantomData<&'a str>
                }

                impl <'a> Default for [<$header Value>]<'a> {
                    fn default() -> Self {
                        [<$header Value>] {
                            value: or_else!($($($default)?)?,EMPTY),
                            _lt: PhantomData
                        }
                    }
                }

                 impl <'a> [<$header Value>]<'a> {

                    pub const NAME: &'static str =  $name;

                    pub fn new(value: or_else_type!($($types)?,&'a str)) -> Self {
                        [<$header Value>] { value, _lt: PhantomData }
                    }

                    fn from_str(input: &'a str) -> Result<[<$header Value>]<'a>, StompParseError> {
                        choose_from_presence!($($types)? ($($types)?::from_str(input).map([<$header Value>]::<'a>::new)
                            .map_err(|_| StompParseError::new("[<Error Parsing $header Value>]"))), (Ok([<$header Value>]::new(input))))

                    }

                    pub fn from_either(_value: Option<or_else_type!($($types)?,&'static str)>, _bytes: &'static [u8] ) -> Self {
                        choose_from_presence!($($types)? {
                            Self::new(_value.unwrap())
                        }, {
                            Self::new(unsafe { std::str::from_utf8_unchecked( _bytes ) })
                        })
                    }

                }

                impl <'a> HeaderValue<'a>  for [<$header Value>]<'a> {
                    type OwnedValue = or_else_type!($($types)?,&'a str);
                    type Value=&'a or_else_type!($($types)?,str);
                    const OWNED: bool = choose_from_presence!($($types)? true, false);

                    fn header_type(&self) -> HeaderType {
                        HeaderType::$header
                    }
                    fn header_name(&self) -> &str {
                        [<$header Value>]::NAME
                    }
                    fn value(&'a self) -> &'a or_else_type!($($types)?,str) {
                        choose_from_presence!($($types)? {&self.value}, {self.value})
                    }
                }

                impl <'a> Into<or_else_type!($($types)?,&'a str)> for [<$header Value>]<'a> {
                    fn into(self) -> or_else_type!($($types)?,&'a str) {
                        self.value
                    }
                }

                impl<'a> std::fmt::Display for [<$header Value>]<'a> {
                    header_display!( );
                }

        }
    };
}
macro_rules! headers {
        ( $( ($header:ident, $name:literal $(,$types:ty $(, $default:expr )?)? ) ),*  ) => {

             #[derive(Debug, Eq, PartialEq, Clone)]
            pub struct CustomValue<'a> {
                name: &'a str,
                value: &'a str
            }

             impl <'a> CustomValue<'a> {
                pub fn new(name: &'a str, value: &'a str) -> Self {
                    CustomValue {
                        name,
                        value
                    }
                }

                pub fn decoded_name(&self) -> Result<Either<&str, String>, StompParseError> {
                    decode_str(self.name)
                }
            }

            impl <'a> HeaderValue<'a> for CustomValue<'a> {
                type OwnedValue = &'a str;
                type Value = &'a str;
                const OWNED: bool = false;

                fn header_type(&self) -> HeaderType {
                    HeaderType::Custom(self.name)
                }
                fn header_name(&self) -> &str {
                    &self.name
                }
                fn value(&self) -> &'a str {
                    &self.value
                }
            }

             impl <'a> std::fmt::Display for CustomValue<'a> {
                header_display!( );
            }


        #[derive(Debug, Eq, PartialEq, Clone)]
        pub enum HeaderType<'a> {
            $(
            $header
            ),*
            ,Custom(&'a str)
        }

        impl <'a> HeaderType<'a> {
            pub fn matches(&self, name: &str) -> bool {
                match self {
                        $(
                            HeaderType::$header => name == $name,
                        )*
                        HeaderType::Custom(header_name) => &name == header_name
                    }
            }
        }

        impl <'a> TryFrom<&'a str> for HeaderType<'a> {
            type Error = StompParseError;
            fn try_from(input: &'a str) -> std::result::Result<HeaderType<'a>, StompParseError> {
                match(input) {
                        $(
                            $name => Ok(HeaderType::$header),
                        )*
                        name => Ok(HeaderType::Custom(name))
                    }
            }
        }

         impl <'a> std::fmt::Display for HeaderType<'a> {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                match(self) {
                    $(HeaderType::$header => {
                        formatter.write_str($name)
                    })*
                    HeaderType::Custom(name) => {
                        formatter.write_str(name)
                    }
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
                    Custom(CustomValue<'a>)
                }

                #[doc(hidden)]
                pub mod parser {
                    #![allow(non_snake_case)]

                    use super::*;
                    pub type HeaderValueConverter<'a> = dyn Fn(&'a str) -> Result<Header<'a>, StompParseError> + 'a;

                    pub fn find_header_parser<'a>(header_type: &HeaderType<'a>) -> Box<HeaderValueConverter<'a>> {
                        match header_type {
                            $(
                                HeaderType::$header => Box::new([<parse_ $header _header>]),
                            )*
                            HeaderType::Custom(name) => {
                                let cloned = name.clone();
                                    Box::new(move |value| Ok(Header::<'a>::Custom(CustomValue::<'a>{
                                    name: (&cloned).clone(),
                                    value
                                })))
                            }
                        }
                    }

                    $(
                        pub fn [<parse_ $header _header>](input: &str) -> Result<Header, StompParseError> {
                            [<$header Value>]::from_str(input).map(Header::$header)
                        }
                    )*

                }
        }
    }
}
