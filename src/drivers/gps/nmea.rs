use crate::drivers::gps::nmea::State::InvalidFields;
use crate::drivers::gps::sentences::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Sentence {
    Invalid,
    /// Global Positioning System Fix Data
    GGA(DataGGA),
    /// Course over Ground and Ground Speed
    VTG(DataVTG),
    /// GNSS DOP and Active Satellites
    RMC(DataRMC),
    /// Time and Date
    ZDA(DataZDA),
}

impl Sentence {
    pub fn from(s: &String) -> Sentence {
        if s.ends_with("GGA") {
            return Sentence::GGA(Default::default());
        } else if s.ends_with("VTG") {
            return Sentence::VTG(Default::default());
        } else if s.ends_with("RMC") {
            return Sentence::RMC(Default::default());
        } else if s.ends_with("VTG") {
            return Sentence::VTG(Default::default());
        }

        Sentence::Invalid
    }
}

#[derive(Debug, PartialEq)]
pub enum State {
    /// Parsing on going
    OnGoing,
    /// Invalid sync char (should be '$')
    InvalidSync(char),
    /// Unknown sentence ID
    InvalidId(String),
    /// Incorrect sentence fields
    InvalidFields,
    /// Checksum mismatch between parser and sentence
    ChecksumMismatch { expected: i32, received: i32 },
}

#[derive(Debug, PartialEq)]
enum Field {
    Sync,
    Id,
    Data,
    Checksum,
}

pub struct Parser {
    /// Current NMEA field being parsed
    field: Field,
    /// Which sentence we're parsing
    sntc: Sentence,
    /// Work buffer
    bfr: String,
    /// Current checksum value
    checksum: i32,
    /// Checksum read from sentence
    read_checksum: i32,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            field: Field::Sync,
            sntc: Sentence::Invalid,
            bfr: String::from(""),
            checksum: 0,
            read_checksum: 0,
        }
    }

    pub fn reset(&mut self) {
        self.field = Field::Sync;
        self.sntc = Sentence::Invalid;
        self.bfr = String::from("");
        self.checksum = 0;
        self.read_checksum = 0;
    }

    pub fn fail_with(&mut self, e: State) -> Result<Sentence, State> {
        self.reset();
        Err(e)
    }

    pub fn push(&mut self, c: char) -> Result<Sentence, State> {
        match self.field {
            Field::Sync => {
                if c == '$' {
                    self.reset();

                    self.field = Field::Id
                } else {
                    return self.fail_with(State::InvalidSync(c));
                }
            }
            Field::Id => {
                self.checksum = utils::xor(self.checksum, c);
                if c == ',' {
                    self.sntc = Sentence::from(&self.bfr);
                    if self.sntc == Sentence::Invalid {
                        return self.fail_with(State::InvalidId(self.bfr.clone()));
                    } else {
                        self.field = Field::Data;
                        self.bfr.clear();
                    }
                } else {
                    self.bfr.push(c);
                }
            }
            Field::Data => {
                if c == '*' {
                    if !parse_fields(&mut self.sntc, self.bfr.clone()) {
                        return self.fail_with(InvalidFields);
                    }

                    self.field = Field::Checksum;
                    self.bfr.clear();
                } else {
                    self.bfr.push(c);
                    self.checksum = utils::xor(self.checksum, c);
                }
            }
            Field::Checksum => {
                self.bfr.push(c);

                if self.bfr.len() == 2 {
                    self.read_checksum = utils::parse_checksum(self.bfr.as_str()).unwrap();
                    return if self.checksum == self.read_checksum {
                        self.field = Field::Sync;
                        Ok(self.sntc.clone())
                    } else {
                        self.fail_with(State::ChecksumMismatch {
                            expected: self.checksum,
                            received: self.read_checksum,
                        })
                    }
                }
            }
        }

        Err(State::OnGoing)
    }
}

mod utils {
    use std::num::ParseIntError;

    pub fn xor(current :i32, c :char) -> i32 {
        let ascii = c as i32;
        current ^ ascii
    }

    pub fn parse_checksum(hexstr :&str) -> Result<i32, ParseIntError> {
        i32::from_str_radix(hexstr, 16)
    }
}

mod tests {
    use crate::drivers::gps::nmea::State::OnGoing;

    use super::*;

    #[test]
    fn initial_state() {
        let parser = Parser::new();

        assert_eq!(parser.field, Field::Sync);
        assert_eq!(parser.sntc, Sentence::Invalid);
        assert!(parser.bfr.is_empty());
        assert_eq!(parser.read_checksum, 0);
        assert_eq!(parser.checksum, 0);
    }

    #[test]
    fn reset_state() {
        let mut parser = Parser {
            field: Field::Checksum,
            sntc: Sentence::from(&String::from("GGA")),
            bfr: "42".to_string(),
            checksum: 2,
            read_checksum: 5
        };

        assert_ne!(parser.field, Field::Sync);
        assert_ne!(parser.sntc, Sentence::Invalid);
        assert!(!parser.bfr.is_empty());
        assert_ne!(parser.read_checksum, 0);
        assert_ne!(parser.checksum, 0);

        parser.reset();

        assert_eq!(parser.field, Field::Sync);
        assert_eq!(parser.sntc, Sentence::Invalid);
        assert!(parser.bfr.is_empty());
        assert_eq!(parser.read_checksum, 0);
        assert_eq!(parser.checksum, 0);
    }

    #[test]
    fn invalid_sync() {
        let sentence = "#GGA,dasda,dasdsad,*321";
        let mut parser = Parser::new();

        let ret = parser.push(sentence.chars().nth(0).unwrap());
        assert!(ret.is_err());
        assert!(matches!(ret.err().unwrap(), State::InvalidSync(_)));
    }

    #[test]
    fn invalid_sentence_id() {
        let sentence = "$X,dasda,dasdsad*321";
        let mut parser = Parser::new();

        let mut ret = parser.push(sentence.chars().nth(0).unwrap());
        assert!(ret.is_err());
        assert!(matches!(ret.err().unwrap(), State::OnGoing));

        ret = parser.push(sentence.chars().nth(1).unwrap());
        assert!(ret.is_err());
        assert!(matches!(ret.err().unwrap(), State::OnGoing));

        ret = parser.push(sentence.chars().nth(2).unwrap());
        assert!(ret.is_err());
        assert!(matches!(ret.err().unwrap(), State::InvalidId(_)));
    }

    #[test]
    fn valid_sentence_id() {
        let sentence = "$GPGGA,dasda,dasdsad*321";
        let mut parser = Parser::new();

        let mut ret = parser.push(sentence.chars().nth(0).unwrap());
        assert!(ret.is_err());
        assert!(matches!(ret.err().unwrap(), State::OnGoing));

        for i in 1..7 {
            ret = parser.push(sentence.chars().nth(i).unwrap());
            assert!(ret.is_err());
            assert!(matches!(ret.err().unwrap(), State::OnGoing));
        }

        assert!(matches!(parser.sntc, Sentence::GGA(_)));
    }

    #[test]
    fn full_valid_parse() {
        let sentence = "$GPGGA,202530.00,5109.0262,N,11401.8407,W,5,40,0.5,1097.36,M,-17.00,M,18,TSTR*61";
        let mut parser = Parser::new();

        // parse full sentence
        let mut ret :Result<Sentence, State> = Err(OnGoing);
        for c in sentence.chars() {
            ret = parser.push(c);
        }

        // check parser reset state properly
        assert!(matches!(parser.field, Field::Sync));
        assert!(ret.is_ok());
        // proper checksum
        assert_eq!(parser.read_checksum, 97);
    }
}