use clap::{Args, Parser, Subcommand};
use hermes::{sub::update_srt_time, torrent::download_newest_tracker};

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
    #[command(about = "modify SRT time entries: add or subtract a millisecond offset")]
    Incr { file_name: String, ms: i32 },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // hermes tracker update
    // torrent::download_newest_tracker().await.unwrap()
    let cli = Hermes::parse();

    match &cli.commands {
        Commands::Tracker(tracker_args) => match &tracker_args.commands {
            TrackerCommands::Update {} => download_newest_tracker().await?,
        },
        Commands::Sub(sub_args) => match &sub_args.commands {
            SubCommands::Incr { file_name, ms } => update_srt_time(file_name, *ms),
        },
    }
    Ok(())
}
