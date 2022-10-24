use crate::drivers::gps::field_reader::FieldReader;
use crate::drivers::gps::nmea::Sentence;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct DataGGA {
    pub utc_time: Option<f64>,
    pub lat: Option<f64>,
    pub ns: Option<String>,
    pub lon: Option<f64>,
    pub ew: Option<String>,
    pub validity: u8,
    pub sat: Option<u8>,
    pub hdop: Option<f32>,
    /// @todo alt or height?
    pub alt: Option<f32>,
    pub units: Option<String>,
    pub gsep: Option<f32>,
    pub gsep_units: Option<String>,
    pub dgps_age: Option<f32>,
    pub dgps_id: Option<u8>,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct DataVTG {
    pub cog_true:Option<f32>,
    pub cog_magnetic:Option<f32>,
    pub sog_knots :Option<f32>,
    pub sog_kph :Option<f32>,
    pub mode :Option<String>
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct DataRMC {}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct DataZDA {
    /// UTC time status hhmmss.ss
    pub utc: Option<f32>,
    /// Day from 01 to 31
    pub day: Option<u8>,
    /// Month from 01 to 12
    pub month: Option<u8>,
    /// Year
    pub year: Option<u16>,
}

pub fn parse_fields(s: &mut Sentence, fields_str: String) -> bool {
    let mut fin = FieldReader::new(fields_str, ',');
    match s {
        Sentence::GGA(m) => {
            return
            optional_field(&mut fin, &mut m.utc_time, "GGA: failed parsing utc_time") &&
            optional_field(&mut fin, &mut m.lat, "GGA: failed parsing latitude") &&
            optional_field(&mut fin, &mut m.ns, "GGA: failed parsing N/S") &&
            optional_field(&mut fin, &mut m.lon, "GGA: failed parsing longitude") &&
            optional_field(&mut fin, &mut m.lat, "GGA: failed parsing E/W") &&
            field(&mut fin, &mut m.validity, "GGA: failed parsing validity")  &&
            optional_field(&mut fin, &mut m.sat, "GGA: failed parsing number of satellites") &&
            optional_field(&mut fin, &mut m.hdop, "GGA: failed parsing HDOP") &&
            optional_field(&mut fin, &mut m.alt, "GGA: failed parsing altitude") &&
            optional_field(&mut fin, &mut m.units, "GGA: failed parsing units");
        }
        Sentence::VTG(m) => {
            // True COG and fixed field 'T'
            if !optional_field(&mut fin, &mut m.cog_true, "VTG: failed parsing true COG") {
                return false;
            }

            // Magnetic COG and fixed field 'M'
            if !optional_field(&mut fin, &mut m.cog_magnetic, "VTG: failed parsing magnetic COG") ||
               !fin.skip() {
                return false;
            }

            // Speed Over Ground knots and fixed field 'N'
            if !optional_field(&mut fin, &mut m.sog_knots, "VTG: failed to parse SOG knots") ||
               !fin.skip(){
                return false;
            }

            // Speed Over Ground kph and fixed field 'K'
            if !optional_field(&mut fin, &mut m.sog_kph, "VTG: failed to parse SOG kph") ||
               !fin.skip(){
                return false;
            }

            // Mode indicator
            if !optional_field(&mut fin, &mut m.mode, "VTG: failed to parse mode indicator") {
                return false;
            }

            return true;
        }
        Sentence::RMC(_) => {}
        Sentence::ZDA(_) => {}
        _ => return false
    }

    true
}

// Utils

/// Parse an optional field from the given field reader and store in the
/// given reference variable.
/// If it fails print the given message and return false
fn optional_field<T: std::str::FromStr>(fin: &mut FieldReader,
                                        out: &mut Option<T>,
                                        error_msg: &str) -> bool {
    match fin.read_optional::<T>() {
        Ok(v) => {
            *out = v;
        }
        Err(e) => {
            println!("{} \"{}\"", error_msg, e);
            return false;
        }
    }

    true
}

/// Parse a field from the given field reader and store in the
/// given reference variable.
/// If it fails print the given message and return false
fn field<T: std::str::FromStr>(fin: &mut FieldReader,
                               out: &mut T,
                               error_msg: &str) -> bool {
    match fin.read::<T>() {
        Ok(v) => {
            *out = v;
        }
        Err(e) => {
            println!("{} \"{}\"", error_msg, e);
            return false;
        }
    }

    true
}