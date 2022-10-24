use std::fmt;
use std::fmt::Formatter;
use std::io::{BufRead, Error};

#[derive(Debug)]
pub enum ReadResult {
    Empty,
    InvalidFormat,
    InternalError
}

impl fmt::Display for ReadResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ReadResult::Empty => f.write_str("Empty"),
            ReadResult::InvalidFormat => f.write_str("Invalid format"),
            ReadResult::InternalError => f.write_str("Internal parsing error"),
        }
    }
}

pub struct FieldReader {
    data :std::io::Cursor<String>,
    sep :char,
}

impl FieldReader {
    pub fn new(data_str :String, separator: char) -> FieldReader {
        FieldReader {
            data: std::io::Cursor::new(data_str),
            sep: separator
        }
    }

    /// Skip next field
    pub fn skip(&mut self) -> bool {
        let mut bfr :Vec<u8> = vec![];
        return self.data.read_until(self.sep as u8, &mut bfr).is_ok() && bfr.len() != 0;
    }

    /// Read next value of the given type.
    /// This method allows for empty values and returns Option::None in such a case
    pub fn read_optional<T: std::str::FromStr>(&mut self) -> Result<Option<T>, ReadResult> {
        match self.read() {
            Ok(v) => Ok(Some(v)),
            Err(e) => {
                match e {
                    ReadResult::Empty => Ok(None),
                    _ => Err(e)
                }
            }
        }
    }

    /// Read next value of the given type.
    /// This method assumes that and empty value is not allowed hence an error
    /// ReadResult::Empty is returned in such a case
    pub fn read<T:  std::str::FromStr>(&mut self) -> Result<T, ReadResult> {
        let data_str = self.next()?;

        if let Ok(value) = data_str.parse::<T>() {
            Ok(value)
        } else {
            Err(ReadResult::InvalidFormat)
        }
    }

    /// Fetch the next string up to separator
    /// Also removes the separator if it exists
    fn next(&mut self) -> Result<String, ReadResult> {
        let mut bfr :Vec<u8> = vec![];
        return match self.data.read_until(self.sep as u8, &mut bfr) {
            Ok(v) => {
                if v == 0 {
                    Err(ReadResult::Empty)
                } else {
                    if let Ok(mut data_str) = String::from_utf8(bfr) {
                        if data_str.contains(self.sep) {
                            data_str.pop();
                        }

                        return Ok(data_str);
                    } else {
                        Err(ReadResult::InternalError)
                    }
                }
            }
            Err(_) => { Err(ReadResult::InternalError) }
        }
    }
}

mod tests {
    use super::*;

    #[test]
    /// Given an empty string guarantee that we receive a ReadResult::Empty
    pub fn empty_str() {
        let mut reader = FieldReader::new(String::from(""), ',');
        let ret = reader.read::<i32>();

        assert!(ret.is_err());
        assert!(matches!(ret.err().unwrap(), ReadResult::Empty));

    }

    #[test]
    /// Try to read a i32 value from something that can't be parsed as one
    /// Make sure we receive a ReadResult::InvalidFormat
    pub fn invalid_format() {
        let mut reader = FieldReader::new(String::from("#"), ',');
        let ret = reader.read::<i32>();

        assert!(ret.is_err());
        assert!(matches!(ret.err().unwrap(), ReadResult::InvalidFormat));
    }

    #[test]
    /// Parse a value from a string with no separator
    /// Expect the value to be properly parsed and a ReadResult::Empty to be
    /// returned when trying to read a new value
    pub fn single_argument() {
        let mut reader = FieldReader::new(String::from("10"), ',');
        let mut ret = reader.read::<i32>();

        assert!(ret.is_ok());
        assert_eq!(ret.unwrap(), 10);

        ret = reader.read::<i32>();
        assert!(ret.is_err());
        assert!(matches!(ret.err().unwrap(), ReadResult::Empty));
    }

    #[test]
    /// Parse multiple values until end of string
    /// Expect all values to be parsed correctly and ReadResult::Empty to be
    /// return when reaching EOS
    pub fn to_completion() {
        let mut reader = FieldReader::new(String::from("10,2,1001,50"), ',');
        let mut ret = reader.read::<i32>();

        assert!(ret.is_ok());
        assert_eq!(ret.unwrap(), 10);

        ret = reader.read::<i32>();

        assert!(ret.is_ok());
        assert_eq!(ret.unwrap(), 2);

        ret = reader.read::<i32>();

        assert!(ret.is_ok());
        assert_eq!(ret.unwrap(), 1001);

        ret = reader.read::<i32>();

        assert!(ret.is_ok());
        assert_eq!(ret.unwrap(), 50);
    }

    #[test]
    /// Parse multiple f64 value among other types
    pub fn with_floating_point_f64() {
        let mut reader = FieldReader::new(String::from("10.2,2,1001.999,50"), ',');
        let ret = reader.read::<f64>();

        assert!(ret.is_ok());
        assert_eq!(ret.unwrap(), 10.2);

        let ret2 = reader.read::<i32>();

        assert!(ret2.is_ok());
        assert_eq!(ret2.unwrap(), 2);

        let ret3 = reader.read::<f64>();

        assert!(ret3.is_ok());
        assert_eq!(ret3.unwrap(), 1001.999);

        let ret4 = reader.read::<i32>();

        assert!(ret4.is_ok());
        assert_eq!(ret4.unwrap(), 50);
    }

    #[test]
    /// Parser multiple f32 values among other types
    pub fn with_floating_point_f32() {
        let mut reader = FieldReader::new(String::from("10.2,2,1001.999,50"), ',');
        let ret = reader.read::<f32>();

        assert!(ret.is_ok());
        assert_eq!(ret.unwrap(), 10.2);

        let ret2 = reader.read::<i32>();

        assert!(ret2.is_ok());
        assert_eq!(ret2.unwrap(), 2);

        let ret3 = reader.read::<f32>();

        assert!(ret3.is_ok());
        assert_eq!(ret3.unwrap(), 1001.999);

        let ret4 = reader.read::<i32>();

        assert!(ret4.is_ok());
        assert_eq!(ret4.unwrap(), 50);
    }

    #[test]
    /// Parse multiple value types such as string, floating point and integer
    pub fn mixed() {
        let mut reader = FieldReader::new(String::from("10.2,2,ABCD,0.22,#aa"), ',');

        let ret = reader.read::<f64>();
        assert!(ret.is_ok());
        assert_eq!(ret.unwrap(), 10.2);

        let ret2 = reader.read::<i32>();
        assert!(ret2.is_ok());
        assert_eq!(ret2.unwrap(), 2);

        let ret3 = reader.read::<String>();
        assert!(ret3.is_ok());
        assert_eq!(ret3.unwrap(), "ABCD");

        let ret4 = reader.read::<f64>();
        assert!(ret4.is_ok());
        assert_eq!(ret4.unwrap(), 0.22);

        let ret5 = reader.read::<String>();
        assert!(ret5.is_ok());
        assert_eq!(ret5.unwrap(), "#aa");
    }

    #[test]
    /// Parse an integer into a floating point type
    pub fn integer_as_floating() {
        let mut reader = FieldReader::new(String::from("10"), ',');

        let ret = reader.read::<f64>();
        assert!(ret.is_ok());
        assert_eq!(ret.unwrap(), 10.0);
    }

    #[test]
    /// Parse a floating point type as integer and expect a failing test with
    /// return ReadResult::InvalidFormat
    pub fn floating_as_integer() {
        let mut reader = FieldReader::new(String::from("12.02"), ',');

        let ret = reader.read::<i32>();
        assert!(ret.is_err());
        assert!(matches!(ret.err().unwrap(), ReadResult::InvalidFormat));
    }

    #[test]
    /// Skip value
    /// Expect all values to be parsed correctly except skipped one
    pub fn skip() {
        let mut reader = FieldReader::new(String::from("10,2,1001,50"), ',');
        let mut ret = reader.read::<i32>();

        assert!(ret.is_ok());
        assert_eq!(ret.unwrap(), 10);

        assert!(reader.skip());

        ret = reader.read::<i32>();

        assert!(ret.is_ok());
        assert_eq!(ret.unwrap(), 1001);

        ret = reader.read::<i32>();

        assert!(ret.is_ok());
        assert_eq!(ret.unwrap(), 50);
    }

    #[test]
    /// Skip all fields
    /// Expect true on all calls until no more fields to skip
    pub fn skip_all() {
        let mut reader = FieldReader::new(String::from("10,2,1001,50"), ',');

        assert!(reader.skip());
        assert!(reader.skip());
        assert!(reader.skip());
        assert!(reader.skip());

        // nothing more to skip
        assert!(!reader.skip());

    }
}