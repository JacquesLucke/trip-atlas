use anyhow::Result;
use std::path::Path;

use crate::prepare_gtfs_as_rkyv;

pub async fn prepare_direct_connections(gtfs_folder_path: &Path) -> Result<()> {
    let start = std::time::Instant::now();
    let src_data = prepare_gtfs_as_rkyv::load_gtfs_folder_rkyv(gtfs_folder_path).await?;
    println!("Loading GTFS data took {:?}", start.elapsed());

    for stop in src_data.rkyv_data.stops.iter() {
        if let Some(name) = stop.name.as_ref() {
            if name.contains("Hennigsdorf") {
                println!("{:?}", name);
            }
        }
    }

    Ok(())
}
