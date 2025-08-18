use std::error;

use blowup::{
    sub::{
        OutputFormat, OverlapFixMode, compare_two_srt_file, extract_sub_srt,
        list_all_subtitle_stream, update_srt_time,
    },
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
    #[command(
        name = "list",
        about = "List the number of subtitle streams in a video container"
    )]
    ListSubStream {
        file_name: String,
        #[arg(
            short = 'f',
            long = "format",
            help = "Output format: list/json/tab, default is list"
        )]
        format: Option<OutputFormat>,
    },
    #[command(
        name = "cmp",
        about = "Compare two SRT subtitle files, using -i to enable interactive display"
    )]
    CmpTwoSrt {
        srt_1: String,
        srt_2: String,
        #[arg(short, help = "enable interactive display")]
        interactive: bool,
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
            SubCommands::ListSubStream { file_name, format } => {
                list_all_subtitle_stream(file_name, format.unwrap_or(OutputFormat::List).clone())
                    .await
                    .expect("Failed to retrieve the information about subtitle stream")
            }
            SubCommands::CmpTwoSrt {
                srt_1,
                srt_2,
                interactive,
            } => compare_two_srt_file(srt_1.clone(), srt_2.clone(), *interactive)
                .await
                .expect("Failed to compare the two srt files"),
        },
    }
    Ok(())
}
