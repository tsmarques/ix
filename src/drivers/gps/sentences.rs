use crate::drivers::gps::nmea::Sentence;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct DataGGA {
    uct_time :f32,
    flat :f32,
    ns :String,
    lon :f32,
    ew :String,
    validity: u8,
    sat :u32,
    hdp :f32,
    alt :f32,
    units :String,
    gsep :f32,
    gsep_units :String,
    dgps_age :f32,
    dgps_id :u8,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct DataVTG {

}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct DataGSA {

}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct DataGSV {

}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct DataRMC {

}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct DataZDA {
    /// UTC time status hhmmss.ss
    utc :f32,
    /// Day from 01 to 31
    day :u8,
    /// Month from 01 to 12
    month :u8,
    /// Year
    year :u16
}

pub fn parse_fields(s: &mut Sentence, fields_str :&str) -> bool {
    match s {
        Sentence::GGA(_) => {}
        Sentence::VTG(_) => {}
        Sentence::GSA(_) => {}
        Sentence::GSV(_) => {}
        Sentence::RMC(_) => {}
        Sentence::ZDA(_) => {}
        _ => return false
    }

    true
}