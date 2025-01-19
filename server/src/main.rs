use std::collections::{BinaryHeap, HashMap};

use chrono::{Date, DateTime, Duration, TimeZone, Utc};
use ustr::{ustr, Ustr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct VehicleID(Ustr);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct StationID(Ustr);

#[derive(Debug, Clone)]
struct Station {
    id: StationID,
    max_change_time: Duration,
    vehicle_connections: Vec<VehicleConnection>,
    walk_connections: Vec<WalkConnection>,
}

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
struct ArrivalOrigin<'a> {
    vehicle: Option<VehicleID>,
    previous_station: &'a Station,
}

#[derive(Debug, Clone)]
struct ArrivalInfo<'a> {
    time: DateTime<Utc>,
    origin: Option<ArrivalOrigin<'a>>,
}

#[derive(Debug, Clone)]
struct StationState<'a> {
    arrivals: Vec<ArrivalInfo<'a>>,
    earliest_arrival: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
struct AnalysisState<'a> {
    stations_by_id: &'a HashMap<StationID, Box<Station>>,
    station_states: HashMap<StationID, StationState<'a>>,
    queue: BinaryHeap<ArrivalEvent<'a>>,
}

#[derive(Clone)]
struct ArrivalEvent<'a> {
    time: DateTime<Utc>,
    station: &'a Station,
    origin: Option<ArrivalOrigin<'a>>,
}

impl PartialEq for ArrivalEvent<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl PartialOrd for ArrivalEvent<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.time.cmp(&other.time))
    }
}

impl Eq for ArrivalEvent<'_> {}

impl Ord for ArrivalEvent<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}

impl std::fmt::Debug for ArrivalEvent<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Event")
            .field("station", &self.station.id)
            .field("time", &self.time)
            .field("origin", &self.origin)
            .finish()
    }
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

fn find_first_departure_index(station: &Station, time: DateTime<Utc>) -> Option<usize> {
    let next_vehicle_connection = station
        .vehicle_connections
        .binary_search_by(|conn| return conn.departure.cmp(&time));

    match next_vehicle_connection {
        Ok(mut index) => {
            while index > 0 && station.vehicle_connections[index - 1].departure == time {
                index -= 1;
            }
            Some(index)
        }
        Err(index) => {
            if index >= station.vehicle_connections.len() {
                None
            } else {
                Some(index)
            }
        }
    }
}

fn find_next_departures(station: &Station, time: DateTime<Utc>) -> &[VehicleConnection] {
    let next_vehicle_connection = station.vehicle_connections.binary_search_by(|conn| {
        return conn.departure.cmp(&time);
    });

    let found_index = match next_vehicle_connection {
        Ok(index) => index,
        Err(index) => index,
    };
    if found_index >= station.vehicle_connections.len() {
        return &[];
    }
    let mut first_index = found_index;
    while first_index > 0 && station.vehicle_connections[first_index - 1].departure == time {
        first_index -= 1;
    }
    let mut last_index = found_index;
    while last_index < station.vehicle_connections.len()
        && station.vehicle_connections[last_index].departure == time
    {
        last_index += 1;
    }
    &station.vehicle_connections[first_index..=last_index]
}

fn find_potentially_useful_vehicle_connections<'a>(
    station: &'a Station,
    arrival_time: DateTime<Utc>,
    stations_by_id: &HashMap<StationID, Box<Station>>,
) -> Vec<&'a VehicleConnection> {
    let mut next_stations: HashMap<StationID, Vec<&VehicleConnection>> = HashMap::new();
    let mut result = vec![];
    for conn in &station.vehicle_connections {
        if conn.departure < arrival_time {
            continue;
        }
        let next_station = stations_by_id.get(&conn.next_station).unwrap();
        let connections_to_same_station = next_stations.entry(conn.next_station).or_default();
        let has_better_alternative = connections_to_same_station
            .iter()
            .any(|other_conn| other_conn.arrival + next_station.max_change_time < conn.arrival);
        if !has_better_alternative {
            connections_to_same_station.push(conn);
            result.push(conn);
        }
    }
    result
}

fn arrive_at<'a>(station: &'a Station, info: ArrivalInfo<'a>, state: &mut AnalysisState<'a>) {
    println!("Arrive at {:?} at {:?}", station.id.0.as_str(), info.time);
    let station_state = state
        .station_states
        .entry(station.id)
        .or_insert_with(|| StationState {
            arrivals: vec![],
            earliest_arrival: None,
        });

    if station_state
        .earliest_arrival
        .unwrap_or(DateTime::<Utc>::MAX_UTC)
        < info.time
    {
        return;
    }
    station_state.earliest_arrival = Some(info.time);

    let connections =
        find_potentially_useful_vehicle_connections(station, info.time, &state.stations_by_id);
    for conn in connections {
        state.queue.push(ArrivalEvent {
            time: conn.arrival,
            station: state.stations_by_id.get(&conn.next_station).unwrap(),
            origin: Some(ArrivalOrigin {
                vehicle: Some(conn.vehicle),
                previous_station: station,
            }),
        });
    }

    station_state.arrivals.push(info);
}

fn analyse_single_start(
    start_station_id: StationID,
    start_time: DateTime<Utc>,
    stations_by_id: &HashMap<StationID, Box<Station>>,
) {
    let mut state = AnalysisState {
        stations_by_id,
        station_states: HashMap::new(),
        queue: BinaryHeap::new(),
    };

    let start_station = stations_by_id.get(&start_station_id).unwrap();
    arrive_at(
        start_station,
        ArrivalInfo {
            time: start_time,
            origin: None,
        },
        &mut state,
    );

    while let Some(event) = state.queue.pop() {
        arrive_at(
            event.station,
            ArrivalInfo {
                time: event.time,
                origin: event.origin,
            },
            &mut state,
        );
    }
}

fn main() {
    let hennigsdorf_id = StationID(ustr("Hennisdorf"));
    let heiligensee_id = StationID(ustr("Heiligensee"));

    let hennigsdorf = Box::new(Station {
        id: hennigsdorf_id,
        vehicle_connections: vec![],
        walk_connections: vec![],
        max_change_time: Duration::minutes(5),
    });
    let heiligensee = Box::new(Station {
        id: heiligensee_id,
        vehicle_connections: vec![],
        walk_connections: vec![],
        max_change_time: Duration::minutes(2),
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
