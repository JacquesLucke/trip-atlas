use std::path::Path;

use anyhow::Result;

use crate::{prepare_direct_connections_rkyv, prepare_gtfs_as_rkyv};

pub async fn find_optimal_paths(gtfs_folder_path: &Path) -> Result<()> {
    let gtfs_rkyv = prepare_gtfs_as_rkyv::load_gtfs_folder_rkyv(gtfs_folder_path).await?;
    let all_connections_rkyv =
        prepare_direct_connections_rkyv::load_direct_connections_rkyv(gtfs_folder_path).await?;

    for station in all_connections_rkyv.stations.iter() {
        println!(
            "{:?}: {:?}",
            gtfs_rkyv.stops[station.main_stop_i.to_native() as usize].name,
            station.connections.len()
        );
    }

    Ok(())
}
