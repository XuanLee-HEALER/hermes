use std::error;

use blowup::{
    sub::{OverlapFixMode, extract_sub_srt, update_srt_time},
    torrent::download_newest_tracker,
};
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(propagate_version = true, version = "0.1", about = "all about movie", long_about = None)]
struct Hermes {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "handle all things about tracker list")]
    Tracker(TrackerArgs),
    #[command(about = "subtitle file processing tools")]
    Sub(SubArgs),
}

#[derive(Args)]
struct TrackerArgs {
    #[command(subcommand)]
    commands: TrackerCommands,
}

#[derive(Subcommand)]
enum TrackerCommands {
    #[command(about = "update the newest tracker list")]
    Update {},
}

#[derive(Args)]
struct SubArgs {
    #[command(subcommand)]
    commands: SubCommands,
}

#[derive(Subcommand)]
enum SubCommands {
    #[command(about = "Modify SRT time entries: add or subtract a millisecond offset")]
    Incr {
        #[arg(help = "Video file path")]
        file_name: String,
        #[arg(help = "Modification duration in milliseconds")]
        ms: i64,
        #[arg(
            help = "Mode for handling overlaps: 1 to keep the first entry's time, 2 to keep the second."
        )]
        overlap_mode: OverlapFixMode,
    },
    #[command(
        name = "export",
        about = "Extract subtitle streams from the specified video container to a designated location"
    )]
    ExportSub {
        file_name: String,
        output_path: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    // hermes tracker update
    // torrent::download_newest_tracker().await.unwrap()
    let cli = Hermes::parse();

    match &cli.commands {
        Commands::Tracker(tracker_args) => match &tracker_args.commands {
            TrackerCommands::Update {} => download_newest_tracker().await?,
        },
        Commands::Sub(sub_args) => match &sub_args.commands {
            SubCommands::Incr {
                file_name,
                ms,
                overlap_mode,
            } => update_srt_time(file_name, *ms, overlap_mode.clone()),
            SubCommands::ExportSub {
                file_name,
                output_path,
            } => extract_sub_srt(file_name, output_path)
                .await
                .expect("Failed to extract the subtitle stream from media file"),
        },
    }
    Ok(())
}
