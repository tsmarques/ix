use crate::drivers::gps::nmea::Sentence;
use crate::drivers::gps::sentences::DataGGA;
use crate::drivers::gps::Task;
use crate::{drivers, task};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Barrier};

#[test]
fn handle_latitude() {
    let task_barrier = Arc::new(Barrier::new(1));
    let mut task_flag = Arc::new(AtomicBool::new(true));

    let mut gps_task = Task::new(task::Context {
        running: Arc::clone(&task_flag),
        barrier: Arc::clone(&task_barrier),
    });

    gps_task.handle_latitude(Some(12.02), Some(String::from("N")));
    assert_eq!(gps_task.fix._lat, 12.02);

    gps_task.handle_latitude(Some(12.02), Some(String::from("S")));
    assert_eq!(gps_task.fix._lat, -12.02);
}

#[test]
fn handle_longitude() {
    let task_barrier = Arc::new(Barrier::new(1));
    let mut task_flag = Arc::new(AtomicBool::new(true));

    let mut gps_task = Task::new(task::Context {
        running: Arc::clone(&task_flag),
        barrier: Arc::clone(&task_barrier),
    });

    gps_task.handle_longitude(Some(12.02), Some(String::from("E")));
    assert_eq!(gps_task.fix._lon, 12.02);

    gps_task.handle_longitude(Some(12.02), Some(String::from("W")));
    assert_eq!(gps_task.fix._lon, -12.02);
}

#[test]
fn handle_gga() {
    let task_barrier = Arc::new(Barrier::new(1));
    let mut task_flag = Arc::new(AtomicBool::new(true));

    let mut task = Task::new(task::Context {
        running: Arc::clone(&task_flag),
        barrier: Arc::clone(&task_barrier),
    });

    let gga = Sentence::GGA(DataGGA {
        utc_time: Some(165035.0),
        lat: Some(28.608389),
        ns: Some(String::from("N")),
        lon: Some(80.604333),
        ew: Some(String::from("W")),
        validity: 1,
        sat: Some(14),
        hdop: Some(0.7),
        alt: Some(0.0),
        units: None,
        gsep: None,
        gsep_units: None,
        dgps_age: None,
        dgps_id: None,
    });

    task.handle_sentence(gga);
    assert_eq!(
        task.fix._type,
        (imc::GpsFix::TypeEnum::GFT_STANDALONE as u8)
    );

    // valid pos
    assert_eq!(
        task.fix._validity & (imc::GpsFix::ValidityBits::GFV_VALID_POS as u16),
        (imc::GpsFix::ValidityBits::GFV_VALID_POS as u16)
    );

    // valid hdop
    assert_eq!(
        task.fix._validity & (imc::GpsFix::ValidityBits::GFV_VALID_HDOP as u16),
        (imc::GpsFix::ValidityBits::GFV_VALID_HDOP as u16)
    );
}
