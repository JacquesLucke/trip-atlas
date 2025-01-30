use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::Path;

mod export_station_locations;
mod find_optimal_paths;
mod gtfs_rkyv;
mod memory_mapped_rkyv;
mod pooled_chunked_vector;
mod prepare_direct_connections_rkyv;
mod prepare_gtfs_as_rkyv;

#[derive(Parser, Debug)]
#[command(name = "trip-atlas")]
struct CLI {
    #[command(subcommand)]
    command: CLICommand,
}

#[derive(Subcommand, Debug)]
enum CLICommand {
    PrepareGTFS {
        #[arg(long)]
        gtfs_path: String,
    },
    ExportStationLocations {
        #[arg(long)]
        gtfs_path: String,
        #[arg(long)]
        output_path: String,
    },
    FindOptimalPaths {
        #[arg(long)]
        gtfs_path: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    simple_logger::SimpleLogger::new().init()?;

    let cli = CLI::parse();
    match cli.command {
        CLICommand::PrepareGTFS { gtfs_path } => {
            prepare_gtfs_as_rkyv::ensure_gtfs_folder_rkyv(Path::new(&gtfs_path)).await?;
        }
        CLICommand::ExportStationLocations {
            gtfs_path,
            output_path,
        } => {
            export_station_locations::export_station_locations(
                Path::new(&gtfs_path),
                Path::new(&output_path),
            )
            .await?;
        }
        CLICommand::FindOptimalPaths { gtfs_path } => {
            find_optimal_paths::find_optimal_paths(Path::new(&gtfs_path)).await?;
        }
    }
    Ok(())
}
