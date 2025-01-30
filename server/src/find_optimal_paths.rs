use anyhow::Result;
use indicatif::ProgressIterator;
use std::{collections::HashMap, io::Write, path::Path};

use crate::{prepare_direct_connections_rkyv, prepare_gtfs_as_rkyv};

pub async fn find_optimal_paths(gtfs_folder_path: &Path) -> Result<()> {
    let gtfs_rkyv = prepare_gtfs_as_rkyv::load_gtfs_folder_rkyv(gtfs_folder_path).await?;
    let all_connections_rkyv =
        prepare_direct_connections_rkyv::load_direct_connections_rkyv(gtfs_folder_path).await?;

    for (i, stop) in all_connections_rkyv.data.stops.iter().enumerate() {
        println!(
            "{:?}: {:?}",
            gtfs_rkyv.rkyv_data.stops[i].name,
            stop.connections.len()
        );
    }

    Ok(())
}
