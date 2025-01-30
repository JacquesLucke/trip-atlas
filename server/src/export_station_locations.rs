use anyhow::Result;
use std::{io::Write, path::Path};

use crate::prepare_gtfs_as_rkyv;

#[derive(Debug, Clone, serde::Serialize)]
struct StationLocations {
    stations: Vec<StationInfo>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct StationInfo {
    name: String,
    latitude: f64,
    longitude: f64,
}

pub async fn export_station_locations(gtfs_folder_path: &Path, output_path: &Path) -> Result<()> {
    let gtfs_rkyv = prepare_gtfs_as_rkyv::load_gtfs_folder_rkyv(gtfs_folder_path).await?;

    let mut result = StationLocations { stations: vec![] };

    for stop in gtfs_rkyv.stops.iter() {
        if let Some(name) = stop.name.as_ref() {
            if name.contains("Hennigsdorf") && stop.parent_station_id.is_none() {
                result.stations.push(StationInfo {
                    name: name.as_str().to_string(),
                    latitude: stop.latitude.unwrap().to_native(),
                    longitude: stop.longitude.unwrap().to_native(),
                });
            }
        }
    }
    let mut file = std::fs::File::create(output_path)?;
    file.write_all(serde_json::to_string_pretty(&result)?.as_bytes())?;
    Ok(())
}
