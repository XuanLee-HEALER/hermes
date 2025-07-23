use clap::{Args, Parser, Subcommand};
use hermes::torrent::{self, download_newest_tracker};

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // hermes tracker update
    // torrent::download_newest_tracker().await.unwrap()
    let cli = Hermes::parse();

    match &cli.commands {
        Commands::Tracker(tracker_args) => match &tracker_args.commands {
            TrackerCommands::Update {} => download_newest_tracker().await?,
        },
    }
    Ok(())
}
