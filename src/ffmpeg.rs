use std::{
    path::{Path, PathBuf},
    result,
};

use thiserror::Error;
use tokio::{
    io::{self},
    process::Command,
};
use walkdir::WalkDir;
use which::which;

#[cfg(target_family = "unix")]
const FFMPEG_CLI: &str = "ffmpeg";
#[cfg(target_family = "windows")]
const FFMPEG_CLI: &str = "ffmpeg.exe";

#[derive(Debug, Error)]
enum FfmpegError {
    #[error("ffmpeg cli is not found")]
    FfmpegNotFound,
    #[error("ffmpeg command execution error")]
    ExecuteError(io::Error),
}

impl From<io::Error> for FfmpegError {
    fn from(value: io::Error) -> Self {
        Self::ExecuteError(value)
    }
}

type Result<T> = result::Result<T, FfmpegError>;

fn ffmpeg_cmd<P: AsRef<Path>>(path: Option<P>) -> Option<PathBuf> {
    match path {
        Some(path) => {
            if path.as_ref().is_dir() {
                for entry in WalkDir::new(path).max_depth(1) {
                    let entry = entry.ok()?;
                    if entry.file_name().to_string_lossy() == FFMPEG_CLI {
                        return Some(entry.into_path());
                    }
                }
            }
            None
        }
        None => which("ffmpeg").ok(),
    }
}

async fn ffmpeg_cli(options: Option<Vec<&str>>) -> Result<()> {
    let ffmpeg = ffmpeg_cmd::<&'static str>(None).ok_or(FfmpegError::FfmpegNotFound)?;
    let mut ffmpeg = Command::new(ffmpeg);
    if let Some(input_op) = options {
        ffmpeg.args(input_op);
    }
    let output = ffmpeg.output().await?;
    println!(
        "ffmpeg output:\n{}",
        String::from_utf8_lossy(&output.stdout[..])
    );
    Ok(())
}

pub async fn extract_sub_srt<P: AsRef<Path>>(file: P, sub: P) -> Result<()> {
    let options: Vec<&str> = vec![
        "-i",
        file.as_ref().to_str().unwrap_or(""),
        "-map",
        "0:s:0",
        "-c",
        "copy",
        sub.as_ref().to_str().unwrap_or(""),
    ];
    ffmpeg_cli(Some(options)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    #[test]
    fn test_ffmpeg_cmd() {
        assert_eq!(
            ffmpeg_cmd::<&'static str>(None),
            Some(PathBuf::from_str("/opt/homebrew/bin/ffmpeg").unwrap())
        );
        assert_eq!(ffmpeg_cmd(Some("/path")), None);
        assert_eq!(
            ffmpeg_cmd(Some("/opt/homebrew/bin")),
            Some(PathBuf::from_str("/opt/homebrew/bin/ffmpeg").unwrap())
        );
    }

    #[tokio::test]
    async fn test_ffmpeg_cli() {
        ffmpeg_cli(Some(vec!["-help"])).await.unwrap();
    }

    #[tokio::test]
    async fn test_extract_sub_srt() {
        let film =
            "~/Movies/parasite/Parasite.2019.REPACK.2160p.4K.BluRay.x265.10bit.AAC7.1-[YTS.MX].mkv";
        let film = shellexpand::tilde(film).to_string();
        let sub = "./test_sub.srt";
        extract_sub_srt(film, sub.to_string()).await.unwrap();
    }
}
