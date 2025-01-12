use std::{cell::RefCell, collections::HashMap, iter::Map, ops::Deref, rc::Rc};

use chrono::{DateTime, TimeZone, Utc};
use ustr::{ustr, Ustr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct VehicleID(Ustr);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct StationID(Ustr);

#[derive(Debug, Clone)]
struct VehicleConnection {
    vehicle: VehicleID,
    next_station: StationID,
    departure: DateTime<Utc>,
    arrival: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct WalkConnection {
    next_station: StationID,
    duration: chrono::Duration,
}

#[derive(Debug, Clone)]
struct Station {
    id: StationID,
    vehicle_connections: Vec<VehicleConnection>,
    walk_connections: Vec<WalkConnection>,
}

#[derive(Debug, Clone)]
struct ArrivalInfo {
    time: DateTime<Utc>,
    vehicle: Option<VehicleID>,
    previous_station: StationID,
}

#[derive(Debug, Clone)]
struct AnalysisResult {
    arrival_by_station: HashMap<StationID, Vec<ArrivalInfo>>,
}

fn analyse_single_start(
    start: StationID,
    stations_by_id: &HashMap<StationID, Box<Station>>,
) -> AnalysisResult {
    let arrival_by_station = HashMap::new();
    AnalysisResult { arrival_by_station }
}

fn main() {
    let hennigsdorf_id = StationID(ustr("Hennisdorf"));
    let heiligensee_id = StationID(ustr("Heiligensee"));

    let hennigsdorf = Box::new(Station {
        id: hennigsdorf_id,
        vehicle_connections: vec![],
        walk_connections: vec![],
    });
    let heiligensee = Box::new(Station {
        id: heiligensee_id,
        vehicle_connections: vec![],
        walk_connections: vec![],
    });

    let mut stations_by_id = HashMap::new();
    stations_by_id.insert(hennigsdorf_id, hennigsdorf);
    stations_by_id.insert(heiligensee_id, heiligensee);

    stations_by_id
        .get_mut(&hennigsdorf_id)
        .unwrap()
        .vehicle_connections
        .push(VehicleConnection {
            vehicle: VehicleID(ustr("S25")),
            next_station: heiligensee_id,
            departure: Utc.with_ymd_and_hms(2023, 1, 12, 14, 28, 0).unwrap(),
            arrival: Utc.with_ymd_and_hms(2025, 1, 12, 14, 34, 0).unwrap(),
        });

    let result = analyse_single_start(hennigsdorf_id, &stations_by_id);

    println!("{:#?}", result);
}
