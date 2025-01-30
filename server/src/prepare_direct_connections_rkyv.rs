use crate::gtfs_rkyv::{self, *};
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
    pub stops: Vec<ConnectionsFromStop>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, Clone)]
#[rkyv(derive(Debug))]
pub struct ConnectionsFromStop {
    pub connections: Vec<ConnectionToStop>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, Copy, Clone)]
#[rkyv(derive(Debug))]
pub struct ConnectionToStop {
    pub to_station_i: u32,
    pub duration: u32,
}

pub struct AllConnectionsRkyv<'a> {
    pub _mmap: memmap2::Mmap,
    // This references data owned by the mmap.
    pub data: &'a ArchivedAllConnections,
}

const DIRECT_CONNECTIONS_FILE_NAME: &str = "all_connections.bin";

pub async fn load_direct_connections_rkyv(gtfs_folder_path: &Path) -> Result<AllConnectionsRkyv> {
    let rkyv_path = ensure_direct_connections_rkyv(gtfs_folder_path).await?;
    let file = std::fs::File::open(&rkyv_path)?;
    // Safety: This is safe for as long as the underlying file is not modified.
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    let buffer: &[u8] = unsafe { std::slice::from_raw_parts(mmap.as_ptr(), mmap.len()) };
    let rkyv_data = unsafe { rkyv::access_unchecked::<ArchivedAllConnections>(buffer) };
    Ok(AllConnectionsRkyv {
        _mmap: mmap,
        data: rkyv_data,
    })
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

    let mut index_by_stop_id = HashMap::new();
    for (i, stop) in src_data
        .rkyv_data
        .stops
        .iter()
        .enumerate()
        .into_iter()
        .progress_with_style(style.clone())
        .with_message("Remember index by stop id.")
        .with_finish(indicatif::ProgressFinish::AndLeave)
    {
        index_by_stop_id.insert(stop.id.as_str(), i as u32);
    }

    let mut stops_by_trip = HashMap::new();

    for stop_time in src_data
        .rkyv_data
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
            if let (Some(from_stop_i), Some(to_stop_i)) = (
                index_by_stop_id.get(connection[0].stop_id.as_str()),
                index_by_stop_id.get(connection[1].stop_id.as_str()),
            ) {
                if let (Some(deparature_time), Some(arrival_time)) = (
                    connection[0].departure_time.as_ref(),
                    connection[1].arrival_time.as_ref(),
                ) {
                    let duration = arrival_time - deparature_time;
                    let entry = shortest_durations
                        .entry((*from_stop_i, *to_stop_i))
                        .or_insert(duration);
                    if *entry > duration {
                        *entry = duration;
                    }
                }
            }
        }
    }

    let mut all_connections = AllConnections {
        stops: vec![
            ConnectionsFromStop {
                connections: vec![],
            };
            src_data.rkyv_data.stops.len()
        ],
    };

    for ((from_station_i, to_station_i), duration) in shortest_durations
        .iter()
        .progress_with_style(style.clone())
        .with_message("Create connections.")
        .with_finish(indicatif::ProgressFinish::AndLeave)
    {
        all_connections.stops[*from_station_i as usize]
            .connections
            .push(ConnectionToStop {
                to_station_i: *to_station_i,
                duration: *duration,
            });
    }

    return Ok(rkyv::to_bytes::<rkyv::rancor::Error>(&all_connections)?);
}
