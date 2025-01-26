use anyhow::Result;
use std::{collections::HashMap, path::Path};

use crate::prepare_gtfs_as_rkyv;

pub async fn prepare_direct_connections(gtfs_folder_path: &Path) -> Result<()> {
    let src_data = prepare_gtfs_as_rkyv::load_gtfs_folder_rkyv(gtfs_folder_path).await?;

    let mut index_by_stop_id = HashMap::new();
    for (i, stop) in src_data.rkyv_data.stops.iter().enumerate() {
        index_by_stop_id.insert(stop.id.as_str(), i);
    }

    let mut stops_by_trip = HashMap::new();

    for stop_time in src_data.rkyv_data.stop_times.iter() {
        let stops_in_trip = stops_by_trip
            .entry(&stop_time.trip_id)
            .or_insert_with(|| vec![]);
        stops_in_trip.push(stop_time);
    }

    let mut shortest_durations = HashMap::new();

    for item in stops_by_trip.iter_mut() {
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
                        .entry((from_station_i, to_station_i))
                        .or_insert(duration);
                    if *entry > duration {
                        *entry = duration;
                    }
                }
            }
        }
    }

    let max_item = shortest_durations.iter().max_by_key(|item| item.1).unwrap();
    println!(
        "{:?} -> {:?}: {:?}s",
        src_data.rkyv_data.stops[*max_item.0 .0].name,
        src_data.rkyv_data.stops[*max_item.0 .1].name,
        max_item.1
    );

    Ok(())
}
