use anyhow::Result;
use indicatif::ProgressIterator;
use std::{collections::HashMap, io::Write, path::Path};

use crate::prepare_gtfs_as_rkyv;

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
#[rkyv(derive(Debug))]
pub struct AllConnections {
    pub stations: Vec<ConnectionsFromStation>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, Clone)]
#[rkyv(derive(Debug))]
pub struct ConnectionsFromStation {
    pub connections: Vec<ConnectionToStation>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, Copy, Clone)]
#[rkyv(derive(Debug))]
pub struct ConnectionToStation {
    pub to_station_i: u32,
    pub duration: u32,
}

pub async fn prepare_direct_connections(gtfs_folder_path: &Path) -> Result<()> {
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
            if let (Some(from_station_i), Some(to_station_i)) = (
                index_by_stop_id.get(connection[0].stop_id.as_str()),
                index_by_stop_id.get(connection[1].stop_id.as_str()),
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

    let mut all_connections = AllConnections {
        stations: vec![
            ConnectionsFromStation {
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
        all_connections.stations[*from_station_i as usize]
            .connections
            .push(ConnectionToStation {
                to_station_i: *to_station_i,
                duration: *duration,
            });
    }

    {
        let rkyv_buffer = rkyv::to_bytes::<rkyv::rancor::Error>(&all_connections)?;
        let mut file = std::fs::File::create(gtfs_folder_path.join("all_connections.bin"))?;
        file.write_all(&rkyv_buffer)?;
    }

    Ok(())
}
