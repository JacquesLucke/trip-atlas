use std::{cmp::Reverse, collections::BinaryHeap, io::Write, path::Path};

use crate::pooled_chunked_vector::{ChunkedVector, ChunkedVectorPool};

use anyhow::Result;

use crate::{prepare_direct_connections_rkyv, prepare_gtfs_as_rkyv};

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

    let mut start_station_indices = vec![];

    let start_station_names = vec!["S Hennigsdorf Bhf", "AB-Schweinheim, Feldchenstr."];

    for (i, station) in all_connections_rkyv.stations.iter().enumerate() {
        let stop = &gtfs_rkyv.stops[station.main_stop_i.to_native() as usize];
        if let Some(name) = stop.name.as_ref() {
            if start_station_names.contains(&name.as_str()) {
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

    let start_instant = std::time::Instant::now();

    let mut chunk_pool: ChunkedVectorPool<u32> = ChunkedVectorPool::new();

    let iterations_num = 1;

    for iteration_i in 0..iterations_num {
        // _find_optimal_paths_with_binary_heap(
        //     &all_connections_rkyv,
        //     &start_station_indices,
        //     &mut station_states,
        // );
        find_optimal_paths_with_time_buckets(
            &all_connections_rkyv,
            &start_station_indices,
            &mut station_states,
            &mut chunk_pool,
        );

        if iteration_i < iterations_num - 1 {
            // Reset for benchmarking reasons.
            for item in &mut station_states {
                *item = StationState {
                    earliest_arrival: None,
                };
            }
        }
    }

    println!("Took {:?}", start_instant.elapsed());

    let mut result = OutputStationsWithTime { stations: vec![] };

    for station_i in 0..all_connections_rkyv.stations.len() {
        let station = &all_connections_rkyv.stations[station_i as usize];
        let station_state = &station_states[station_i as usize];
        let stop = &gtfs_rkyv.stops[station.main_stop_i.to_native() as usize];

        if stop.latitude.unwrap() > 53.12
            || stop.latitude.unwrap() < 52.19
            || stop.longitude.unwrap() < 12.46
            || stop.longitude.unwrap() > 14.0
        {
            continue;
        }

        if let Some(name) = stop.name.as_ref() {
            if !name.contains("Hennigsdorf") {
                continue;
            }
        }

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

fn _find_optimal_paths_with_binary_heap(
    all_connections_rkyv: &prepare_direct_connections_rkyv::ArchivedAllConnections,
    start_station_indices: &[u32],
    station_states: &mut [StationState],
) {
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

    let mut queue = BinaryHeap::new();

    for start_station_i in start_station_indices {
        queue.push(Reverse(TimeWithStation {
            time: 0,
            station_i: *start_station_i,
        }));
        station_states[*start_station_i as usize].earliest_arrival = Some(0);
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
}

fn find_optimal_paths_with_time_buckets(
    all_connections_rkyv: &prepare_direct_connections_rkyv::ArchivedAllConnections,
    start_station_indices: &[u32],
    station_states: &mut [StationState],
    mut chunk_pool: &mut ChunkedVectorPool<u32>,
) {
    struct Bucket {
        station_indices: ChunkedVector<u32>,
    }

    let max_seconds = 3000 * 60;
    let seconds_per_bucket = 30;
    let buckets_num = max_seconds / seconds_per_bucket + 1;

    let mut buckets = vec![];
    buckets.reserve_exact(buckets_num);
    for _ in 0..buckets_num {
        buckets.push(Bucket {
            station_indices: ChunkedVector::new(),
        });
    }

    let first_bucket = &mut buckets[0];
    for station_i in start_station_indices {
        first_bucket
            .station_indices
            .push(*station_i, &mut chunk_pool);
    }

    for station_i in start_station_indices {
        let station_state = &mut station_states[*station_i as usize];
        station_state.earliest_arrival = Some(0);
    }

    for bucket_i in 0..buckets_num {
        let current_time = bucket_i * seconds_per_bucket;
        let (before_buckets, after_buckets) = buckets.split_at_mut(bucket_i + 1);

        let bucket = &mut before_buckets[bucket_i];
        while !bucket.station_indices.is_empty() {
            let mut new_station_indices = ChunkedVector::new();
            let mut chunk_opt = bucket.station_indices.first_chunk();
            while let Some(chunk) = chunk_opt {
                for station_i in chunk.get_slice() {
                    let station = &all_connections_rkyv.stations[*station_i as usize];
                    for connection in station.connections.iter() {
                        let connection_duration = connection.duration.to_native() as usize;
                        let next_station_i = connection.to_station_i.to_native();
                        let next_station_state = &mut station_states[next_station_i as usize];
                        let next_station_time = current_time + connection_duration;
                        if let Some(next_station_earliest_arrival) =
                            next_station_state.earliest_arrival
                        {
                            if next_station_time >= next_station_earliest_arrival as usize {
                                // Connection arrives at a later point than already found.
                                continue;
                            }
                        }
                        next_station_state.earliest_arrival = Some(next_station_time as u32);
                        let next_bucket_i = next_station_time / seconds_per_bucket;
                        if next_bucket_i == bucket_i {
                            new_station_indices.push(next_station_i, &mut chunk_pool);
                        } else {
                            let next_bucket = &mut after_buckets[next_bucket_i - bucket_i - 1];
                            next_bucket
                                .station_indices
                                .push(next_station_i, &mut chunk_pool);
                        }
                    }
                }
                chunk_opt = chunk.next_chunk();
            }

            bucket.station_indices.clear(&mut chunk_pool);
            bucket.station_indices = new_station_indices;
        }
    }
}
