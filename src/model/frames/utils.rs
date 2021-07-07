// use std::{fmt::Display, io::Write};

// // pub fn writeln<W: Write, D: Display>(writer: &mut W, item: D) -> Result<(), std::io::Error> {
// //     write!(writer, "{}\n", item)
// // }

// #[cfg(test)]
// mod test {

//     #[test]
//     fn writeln_writes_line() {
//         let mut buffer: Vec<u8> = Vec::new();
//         super::writeln(&mut buffer, "abc").expect("Error writing");

//         assert_eq!(b"abc\n", buffer.as_slice());
//     }
// }
