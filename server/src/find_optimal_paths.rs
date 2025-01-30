use std::{cmp::Reverse, collections::BinaryHeap, io::Write, path::Path};

use anyhow::Result;

use crate::{prepare_direct_connections_rkyv, prepare_gtfs_as_rkyv};

#[derive(Debug, Clone, Copy)]
struct TimeWithStation {
    time: u32,
    station_i: u32,
}

impl PartialEq for TimeWithStation {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl PartialOrd for TimeWithStation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.time.cmp(&other.time))
    }
}

impl Eq for TimeWithStation {}

impl Ord for TimeWithStation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}

#[derive(Debug, Clone)]
struct StationState {
    earliest_arrival: Option<u32>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct OutputStationsWithTime {
    stations: Vec<OutputStationWithTime>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct OutputStationWithTime {
    name: String,
    time: u32,
    latitude: f64,
    longitude: f64,
}

pub async fn find_optimal_paths(gtfs_folder_path: &Path) -> Result<()> {
    let gtfs_rkyv = prepare_gtfs_as_rkyv::load_gtfs_folder_rkyv(gtfs_folder_path).await?;
    let all_connections_rkyv =
        prepare_direct_connections_rkyv::load_direct_connections_rkyv(gtfs_folder_path).await?;

    let start_instant = std::time::Instant::now();

    let mut start_station_indices = vec![];

    for (i, station) in all_connections_rkyv.stations.iter().enumerate() {
        let stop = &gtfs_rkyv.stops[station.main_stop_i.to_native() as usize];
        if let Some(name) = stop.name.as_ref() {
            if name == "S Hennigsdorf Bhf" {
                start_station_indices.push(i as u32);
            }
        }
    }

    let mut station_states = vec![
        StationState {
            earliest_arrival: None
        };
        all_connections_rkyv.stations.len()
    ];
    let mut queue = BinaryHeap::new();

    for start_station_i in start_station_indices {
        queue.push(Reverse(TimeWithStation {
            time: 0,
            station_i: start_station_i,
        }));
        station_states[start_station_i as usize].earliest_arrival = Some(0);
    }

    while let Some(event) = queue.pop() {
        let station_i = event.0.station_i;
        let station = &all_connections_rkyv.stations[station_i as usize];
        for connection in station.connections.iter() {
            let next_station_i = connection.to_station_i.to_native();
            let next_station_time = event.0.time + connection.duration;
            let next_station_state = &mut station_states[next_station_i as usize];
            if let Some(next_station_earliest_arrival) = next_station_state.earliest_arrival {
                if next_station_time >= next_station_earliest_arrival {
                    // Connection arrives at a later point than already found.
                    continue;
                }
            }
            next_station_state.earliest_arrival = Some(next_station_time);
            queue.push(Reverse(TimeWithStation {
                time: next_station_time,
                station_i: next_station_i,
            }));
        }
    }
    println!("Took {:?}", start_instant.elapsed());

    let mut result = OutputStationsWithTime { stations: vec![] };

    for station_i in 0..all_connections_rkyv.stations.len() {
        let station = &all_connections_rkyv.stations[station_i as usize];
        let station_state = &station_states[station_i as usize];
        let stop = &gtfs_rkyv.stops[station.main_stop_i.to_native() as usize];

        // if !stop.name.as_ref().unwrap().contains("Hennigsdorf")
        //     && !stop.name.as_ref().unwrap().contains("Berlin")
        // {
        //     continue;
        // }

        if let Some(earliest_arrival) = station_state.earliest_arrival {
            result.stations.push(OutputStationWithTime {
                name: stop.name.as_ref().unwrap().to_string(),
                time: earliest_arrival,
                latitude: stop.latitude.unwrap().to_native(),
                longitude: stop.longitude.unwrap().to_native(),
            });
        }
    }

    let mut file = std::fs::File::create(
        "/home/jacques/Documents/trip-atlas/frontend/src/stations_test_data.json",
    )?;
    file.write_all(serde_json::to_string_pretty(&result)?.as_bytes())?;

    // println!(
    //     "Station states: {:#?}",
    //     station_states.iter().take(10000).collect::<Vec<_>>()
    // );

    Ok(())
}
