use anyhow::Result;
use std::{collections::HashMap, path::Path};

use crate::prepare_gtfs_as_rkyv;

pub async fn prepare_direct_connections(gtfs_folder_path: &Path) -> Result<()> {
    let src_data = prepare_gtfs_as_rkyv::load_gtfs_folder_rkyv(gtfs_folder_path).await?;

    let mut stops_by_trip = HashMap::new();

    for stop_time in src_data.rkyv_data.stop_times.iter() {
        let stops_in_trip = stops_by_trip
            .entry(&stop_time.trip_id)
            .or_insert_with(|| vec![]);
        stops_in_trip.push(stop_time);
    }

    for item in stops_by_trip.iter_mut() {
        let trip_id = item.0;
        let stops_in_trip = item.1;
        stops_in_trip.sort_by_key(|stop_time| stop_time.stop_sequence);

        for connection in stops_in_trip.windows(2) {
            println!("{:?} -> {:?}", connection[0].stop_id, connection[1].stop_id);
        }

        println!("{:?}: {:?}", trip_id, stops_in_trip.len());
    }

    Ok(())
}
