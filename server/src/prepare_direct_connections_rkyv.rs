use crate::memory_mapped_rkyv::{self, MemoryMappedRkyv};
use anyhow::Result;
use indicatif::ProgressIterator;
use std::{
    collections::HashMap,
    io::Write,
    path::{Path, PathBuf},
};

use crate::prepare_gtfs_as_rkyv;

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
#[rkyv(derive(Debug))]
pub struct AllConnections {
    pub stations: Vec<ConnectionsFromStation>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, Clone)]
#[rkyv(derive(Debug))]
pub struct ConnectionsFromStation {
    pub main_stop_i: u32,
    pub connections: Vec<ConnectionToStation>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, Copy, Clone)]
#[rkyv(derive(Debug))]
pub struct ConnectionToStation {
    pub to_station_i: u32,
    pub duration: u32,
}

const DIRECT_CONNECTIONS_FILE_NAME: &str = "all_connections.bin";

pub async fn load_direct_connections_rkyv(
    gtfs_folder_path: &Path,
) -> Result<MemoryMappedRkyv<'_, ArchivedAllConnections>> {
    let rkyv_path = ensure_direct_connections_rkyv(gtfs_folder_path).await?;
    unsafe {
        memory_mapped_rkyv::load_memory_mapped_rkyv::<ArchivedAllConnections>(&rkyv_path).await
    }
}

pub async fn ensure_direct_connections_rkyv(gtfs_folder_path: &Path) -> Result<PathBuf> {
    let output_path = gtfs_folder_path.join(DIRECT_CONNECTIONS_FILE_NAME);
    if !output_path.exists() {
        let rkyv_buffer = get_direct_connections_rkyv_buffer(gtfs_folder_path).await?;
        log::info!("Writing data to {:?}", output_path);
        let mut file = std::fs::File::create(&output_path)?;
        file.write_all(&rkyv_buffer)?;
    }
    Ok(output_path)
}

pub async fn get_direct_connections_rkyv_buffer(
    gtfs_folder_path: &Path,
) -> Result<rkyv::util::AlignedVec> {
    let style = indicatif::ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {human_pos:>7}/{human_len:7} {msg}",
    )
    .unwrap();

    let src_data = prepare_gtfs_as_rkyv::load_gtfs_folder_rkyv(gtfs_folder_path).await?;

    let mut connections_by_stations = vec![];

    let mut station_index_by_stop_id = HashMap::new();
    for (stop_i, stop) in src_data
        .stops
        .iter()
        .into_iter()
        .enumerate()
        .progress_with_style(style.clone())
        .with_message("Order unique stations.")
        .with_finish(indicatif::ProgressFinish::AndLeave)
    {
        if stop.parent_station_id.is_none() {
            let station_i = station_index_by_stop_id.len() as u32;
            connections_by_stations.push(ConnectionsFromStation {
                main_stop_i: stop_i as u32,
                connections: vec![],
            });
            station_index_by_stop_id.insert(stop.id.as_str(), station_i);
        }
    }

    for stop in src_data
        .stops
        .iter()
        .into_iter()
        .progress_with_style(style.clone())
        .with_message("Map stops to stations.")
        .with_finish(indicatif::ProgressFinish::AndLeave)
    {
        if let Some(parent_station_id) = stop.parent_station_id.as_ref() {
            if let Some(station_i) = station_index_by_stop_id.get(parent_station_id.as_str()) {
                station_index_by_stop_id.insert(&stop.id.as_str(), *station_i);
            }
        }
    }

    let mut stops_by_trip = HashMap::new();

    for stop_time in src_data
        .stop_times
        .iter()
        .progress_with_style(style.clone())
        .with_message("Find stop times for each trip.")
        .with_finish(indicatif::ProgressFinish::AndLeave)
    {
        let stops_in_trip = stops_by_trip
            .entry(&stop_time.trip_id)
            .or_insert_with(|| vec![]);
        stops_in_trip.push(stop_time);
    }

    let mut shortest_durations = HashMap::new();

    for item in stops_by_trip
        .iter_mut()
        .progress_with_style(style.clone())
        .with_message("Find shortest durations.")
        .with_finish(indicatif::ProgressFinish::AndLeave)
    {
        let stops_in_trip = item.1;
        stops_in_trip.sort_by_key(|stop_time| stop_time.stop_sequence);

        for connection in stops_in_trip.windows(2) {
            if let (Some(from_station_i), Some(to_station_i)) = (
                station_index_by_stop_id.get(connection[0].stop_id.as_str()),
                station_index_by_stop_id.get(connection[1].stop_id.as_str()),
            ) {
                if let (Some(deparature_time), Some(arrival_time)) = (
                    connection[0].departure_time.as_ref(),
                    connection[1].arrival_time.as_ref(),
                ) {
                    let duration = arrival_time - deparature_time;
                    let entry = shortest_durations
                        .entry((*from_station_i, *to_station_i))
                        .or_insert(duration);
                    if *entry > duration {
                        *entry = duration;
                    }
                }
            }
        }
    }

    for ((from_station_i, to_station_i), duration) in shortest_durations
        .iter()
        .progress_with_style(style.clone())
        .with_message("Create connections.")
        .with_finish(indicatif::ProgressFinish::AndLeave)
    {
        connections_by_stations[*from_station_i as usize]
            .connections
            .push(ConnectionToStation {
                to_station_i: *to_station_i,
                duration: *duration,
            });
    }

    return Ok(rkyv::to_bytes::<rkyv::rancor::Error>(&AllConnections {
        stations: connections_by_stations,
    })?);
}
