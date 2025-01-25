use anyhow::Result;

pub async fn prepare_gtfs(gtfs_path: &str) -> Result<()> {
    println!("Loading GTFS data from {:?}, other file", gtfs_path);
    Ok(())
}
