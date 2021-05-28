macro_rules! header_display {
    ( $type1:ty ) => {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
            write!(f, "{}:{}", self.header_name(), self.value)
        }
    };
}
macro_rules! header {

    ( $header:ident, $name:expr, $types:ty ) => {
        paste! {
            #[derive(Debug, Eq, PartialEq, Clone)]
            pub struct [<$header Value>] {
                value: $types
            }

             impl [<$header Value>] {
                const NAME: &'static str =  $name;
                pub fn new(value: $types) -> Self {
                    [<$header Value>] { value }
                }
            }

            impl HeaderValue<$types> for [<$header Value>] {
                fn header_type(&self) -> HeaderType {
                    HeaderType::$header
                }
                fn header_name(&self) -> &str {
                    [<$header Value>]::NAME
                }
                fn value(&self) -> &$types {
                    &self.value
                }
            }

            impl std::fmt::Display for [<$header Value>] {
                header_display!( $types );
            }

            impl FromStr for  [<$header Value>] {
                type Err = StompParseError;
                fn from_str(input: &str) -> Result<Self, Self::Err> {
                    // borrowing here means the input is copied for String-valued values
                    $types::from_str(input).map([<$header Value>]::new).map_err(|_|StompParseError::new("[<Error Parsing $header Value>]"))
                }
            }
        }
    };
}
macro_rules! headers {
        ( $( ($header:ident, $name:literal ,$types:ty ) ),*  ) => {

             #[derive(Debug, Eq, PartialEq, Clone)]
            pub struct CustomValue {
                name: String,
                value: String
            }

             impl CustomValue {
                pub fn new(name: String, value: String) -> Self {
                    CustomValue {
                        name,
                        value
                    }
                }
            }

            impl HeaderValue<String> for CustomValue {
                fn header_type(&self) -> HeaderType {
                    HeaderType::Custom(self.name.clone())
                }
                fn header_name(&self) -> &str {
                    &self.name
                }
                fn value(&self) -> &String {
                    &self.value
                }
            }

             impl std::fmt::Display for CustomValue {
                header_display!( String );
            }


        #[derive(Debug, Eq, PartialEq, Clone)]
        pub enum HeaderType {
            $(
            $header
            ),*
            ,Custom(String)
        }

        impl FromStr for HeaderType {
            type Err = StompParseError;
            fn from_str(input: &str) -> std::result::Result<HeaderType, StompParseError> {
                match(input) {
                        $(
                            $name => Ok(HeaderType::$header),
                        )*
                        name => Ok(HeaderType::Custom(name.to_owned()))
                    }
            }
        }

         impl std::fmt::Display for HeaderType {
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
                header!($header, $name ,$types );

                // fn [<parse_ $header _header>](input: String) -> Result<Header, StompParseError> {
                //     [<$header Value>]::from_str(&input).map(Header::$header)
                // }
            )*

                #[derive(Debug, Eq, PartialEq, Clone)]
                pub enum Header {
                    $(
                    $header([<$header Value>]),
                    )*
                    Custom(CustomValue)
                }

                #[doc(hidden)]
                pub mod parser {
                    #![allow(non_snake_case)]

                    use super::*;
                    pub type HeaderValueConverter = dyn Fn(String) -> Result<Header, StompParseError>;

                    pub fn find_header_parser(header_type: HeaderType) -> Box<HeaderValueConverter> {
                        match header_type {
                            $(
                                HeaderType::$header => Box::new([<parse_ $header _header>]),
                            )*
                            HeaderType::Custom(name) => {
                                let cloned = name.clone();
                                    Box::new(move |value| Ok(Header::Custom(CustomValue{
                                    name: (&cloned).clone(),
                                    value
                                })))
                            }
                        }
                    }

                    $(
                        pub fn [<parse_ $header _header>](input: String) -> Result<Header, StompParseError> {
                            [<$header Value>]::from_str(&input).map(Header::$header)
                        }
                    )*

                }
        }
    }
}
