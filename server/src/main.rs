use std::{cell::RefCell, cmp::min, collections::HashMap, iter::Map, ops::Deref, rc::Rc};

use chrono::{Date, DateTime, TimeZone, Utc};
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
struct ArrivalOrigin {
    vehicle: VehicleID,
    previous_station: StationID,
}

#[derive(Debug, Clone)]
struct ArrivalInfo {
    time: DateTime<Utc>,
    origin: Option<ArrivalOrigin>,
}

#[derive(Debug, Clone)]
struct StationState {
    earliest_arrival: DateTime<Utc>,
    arrivals: Vec<ArrivalInfo>,
}

#[derive(Debug, Clone)]
struct AnalysisState {
    station_states: HashMap<StationID, StationState>,
    queue: priority_queue::PriorityQueue<StationID, DateTime<Utc>>,
}

fn find_next_departure_time(station: &Station, time: DateTime<Utc>) -> Option<DateTime<Utc>> {
    // Find next connection from station:
    let next_vehicle_connection = station
        .vehicle_connections
        .binary_search_by(|conn| return conn.departure.cmp(&time));

    let index = match next_vehicle_connection {
        Ok(index) => index,
        Err(index) => index,
    };

    if index < station.vehicle_connections.len() {
        return Some(station.vehicle_connections[index].departure);
    }
    None
}

fn arrive_at(station: &Station, info: ArrivalInfo, state: &mut AnalysisState) {
    let station_state = state
        .station_states
        .entry(station.id)
        .or_insert_with(|| StationState {
            earliest_arrival: DateTime::<Utc>::MAX_UTC,
            arrivals: vec![],
        });

    if info.time < station_state.earliest_arrival {
        station_state.earliest_arrival = info.time;
        if let Some(next_departure_time) =
            find_next_departure_time(station, station_state.earliest_arrival)
        {
            state.queue.push(station.id, next_departure_time);
        }
    }
    station_state.arrivals.push(info);
}

fn analyse_single_start(
    start_station_id: StationID,
    start_time: DateTime<Utc>,
    stations_by_id: &HashMap<StationID, Box<Station>>,
) {
    let mut state = AnalysisState {
        station_states: HashMap::new(),
        queue: priority_queue::PriorityQueue::new(),
    };

    arrive_at(
        stations_by_id.get(&start_station_id).unwrap(),
        ArrivalInfo {
            time: start_time,
            origin: None,
        },
        &mut state,
    );

    println!("{:#?}", state);
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
            departure: Utc.with_ymd_and_hms(2025, 1, 12, 14, 28, 0).unwrap(),
            arrival: Utc.with_ymd_and_hms(2025, 1, 12, 14, 34, 0).unwrap(),
        });

    analyse_single_start(
        hennigsdorf_id,
        Utc.with_ymd_and_hms(2025, 1, 12, 14, 0, 0).unwrap(),
        &stations_by_id,
    );
}
