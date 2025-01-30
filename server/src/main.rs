use anyhow::Result;
use chrono::{DateTime, Duration, TimeZone, Utc};
use clap::{Parser, Subcommand};
use std::{
    collections::{BinaryHeap, HashMap},
    path::Path,
};
use ustr::{ustr, Ustr};

mod export_station_locations;
mod find_optimal_paths;
mod gtfs_rkyv;
mod memory_mapped_rkyv;
mod pooled_chunked_vector;
mod prepare_direct_connections_rkyv;
mod prepare_gtfs_as_rkyv;

#[derive(Parser, Debug)]
#[command(name = "trip-atlas")]
struct CLI {
    #[command(subcommand)]
    command: CLICommand,
}

#[derive(Subcommand, Debug)]
enum CLICommand {
    TestAnalysis,
    PrepareGTFS {
        #[arg(long)]
        gtfs_path: String,
    },
    ExportStationLocations {
        #[arg(long)]
        gtfs_path: String,
        #[arg(long)]
        output_path: String,
    },
    FindOptimalPaths {
        #[arg(long)]
        gtfs_path: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct VehicleID(Ustr);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct StationID(Ustr);

#[derive(Debug, Clone)]
struct Station {
    id: StationID,
    max_change_time: Duration,
    vehicle_connections: Vec<VehicleConnection>,
}

#[derive(Debug, Clone)]
struct VehicleConnection {
    vehicle: VehicleID,
    next_station: StationID,
    departure: DateTime<Utc>,
    arrival: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct ArrivalOrigin<'a> {
    _vehicle: Option<VehicleID>,
    _previous_station: &'a Station,
}

#[derive(Debug, Clone)]
struct ArrivalInfo<'a> {
    time: DateTime<Utc>,
    _origin: Option<ArrivalOrigin<'a>>,
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

fn _find_next_departure_time(station: &Station, time: DateTime<Utc>) -> Option<DateTime<Utc>> {
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

fn _find_first_departure_index(station: &Station, time: DateTime<Utc>) -> Option<usize> {
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

fn _find_next_departures(station: &Station, time: DateTime<Utc>) -> &[VehicleConnection] {
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

fn _find_potentially_useful_vehicle_connections<'a>(
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
        _find_potentially_useful_vehicle_connections(station, info.time, &state.stations_by_id);
    for conn in connections {
        state.queue.push(ArrivalEvent {
            time: conn.arrival,
            station: state.stations_by_id.get(&conn.next_station).unwrap(),
            origin: Some(ArrivalOrigin {
                _vehicle: Some(conn.vehicle),
                _previous_station: station,
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
            _origin: None,
        },
        &mut state,
    );

    while let Some(event) = state.queue.pop() {
        arrive_at(
            event.station,
            ArrivalInfo {
                time: event.time,
                _origin: event.origin,
            },
            &mut state,
        );
    }
}

fn test_algorithm() {
    let hennigsdorf_id = StationID(ustr("Hennisdorf"));
    let heiligensee_id = StationID(ustr("Heiligensee"));

    let hennigsdorf = Box::new(Station {
        id: hennigsdorf_id,
        vehicle_connections: vec![],
        max_change_time: Duration::minutes(5),
    });
    let heiligensee = Box::new(Station {
        id: heiligensee_id,
        vehicle_connections: vec![],
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

async fn test_analysis() -> Result<()> {
    let database_url = "/home/jacques/Downloads/data_copy.db";
    let pool: sqlx::Pool<sqlx::Sqlite> = sqlx::sqlite::SqlitePoolOptions::new()
        .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(database_url))
        .await?;

    let station_rows = sqlx::query!("SELECT * FROM stations")
        .fetch_all(&pool)
        .await?;
    let trip_rows = sqlx::query!("SELECT * FROM trips LIMIT 100000")
        .fetch_all(&pool)
        .await?;
    let stop_rows = sqlx::query!("SELECT * FROM stops").fetch_all(&pool).await?;

    let mut station_rows_by_id = HashMap::new();
    for row in station_rows {
        station_rows_by_id.insert(StationID(Ustr::from(&row.id)), row);
    }

    let trip_id = &trip_rows
        .iter()
        .find(|row| row.line_name == Some("Bus X36".into()))
        .unwrap()
        .trip_id;
    println!("{:?}", trip_id);

    let stops = stop_rows.iter().filter(|row| row.trip_id == *trip_id);
    for stop in stops {
        println!(
            "{:?} {:?}",
            stop.arrival_time,
            station_rows_by_id
                .get(&StationID(Ustr::from(&stop.id)))
                .unwrap()
                .name
        );
    }

    test_algorithm();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    simple_logger::SimpleLogger::new().init()?;

    let cli = CLI::parse();
    match cli.command {
        CLICommand::TestAnalysis => test_analysis().await?,
        CLICommand::PrepareGTFS { gtfs_path } => {
            prepare_gtfs_as_rkyv::ensure_gtfs_folder_rkyv(Path::new(&gtfs_path)).await?;
        }
        CLICommand::ExportStationLocations {
            gtfs_path,
            output_path,
        } => {
            export_station_locations::export_station_locations(
                Path::new(&gtfs_path),
                Path::new(&output_path),
            )
            .await?;
        }
        CLICommand::FindOptimalPaths { gtfs_path } => {
            find_optimal_paths::find_optimal_paths(Path::new(&gtfs_path)).await?;
        }
    }
    Ok(())
}
