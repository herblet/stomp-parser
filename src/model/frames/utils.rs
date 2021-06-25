use std::{fmt::Display, io::Write};

pub fn writeln<W: Write, D: Display>(writer: &mut W, item: D) -> Result<(), std::io::Error> {
    write!(writer, "{}\n", item)
}

pub fn select_slice<'a>(
    raw: &'a Option<Vec<u8>>,
    offset_length: &'a (isize, usize),
) -> Option<&'a [u8]> {
    raw.as_ref()
        .map(|vec| &vec[offset_length.0 as usize..(offset_length.0 as usize + offset_length.1)])
}

#[cfg(test)]
mod test {

    #[test]
    fn writeln_writes_line() {
        let mut buffer: Vec<u8> = Vec::new();
        super::writeln(&mut buffer, "abc").expect("Error writing");

        assert_eq!(b"abc\n", buffer.as_slice());
    }

    #[test]
    fn select_slice_selects() {
        let buffer = b"abc\n".to_vec();

        let option = Some(buffer);

        let selection = super::select_slice(&option, &(1, 2));

        assert_eq!(b"bc", selection.unwrap());
    }

    #[test]
    fn select_slice_with_none() {
        let selection = super::select_slice(&None, &(1, 2));

        assert!(selection.is_none());
    }
}
