use std::{io, path::Path};

use chrono::{DateTime, Local};
use octocrab::models::repos::Content;
use thiserror::Error;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

const TIME_FMT: &'static str = "%Y-%m-%d %H:%M:%S %z";
const UPDATE_TIME_RECORD: &'static str = "update_record";
const OWNER: &'static str = "ngosang";
const REPO: &'static str = "trackerslist";

#[derive(Debug, Error)]
pub enum TorrentError {
    #[error("Unexpected file type or file not exists")]
    InvalidFile,
    #[error("Network transfer error.")]
    NetworkError,
    #[error("Unknown last modify time error")]
    UnknownLastModifyTimeError,
    #[error("IO error: {0}")]
    IoError(io::Error),
    #[error("Octocrab error: {0}")]
    OctocrabError(octocrab::Error),
}

impl From<octocrab::Error> for TorrentError {
    fn from(value: octocrab::Error) -> Self {
        Self::OctocrabError(value)
    }
}

pub async fn download_newest_tracker() -> Result<(), TorrentError> {
    let req_path = format!("/repos/{}/{}/contents/trackers_all.txt", OWNER, REPO);
    let github = octocrab::instance();
    let content = github._get(&req_path).await?;
    let last_modified = content
        .headers()
        .get("last-modified")
        .ok_or(TorrentError::UnknownLastModifyTimeError)?
        .to_str()
        .map_err(|_| TorrentError::NetworkError)?;
    let last_modified = DateTime::parse_from_rfc2822(last_modified)
        .map_err(|_| TorrentError::NetworkError)?
        .with_timezone(&Local);
    match update_time_record().await {
        Ok(last_update) => {
            if last_modified <= last_update {
                return Ok(());
            } else {
                File::options()
                    .write(true)
                    .append(true)
                    .open(UPDATE_TIME_RECORD)
                    .await
                    .map_err(|e| TorrentError::IoError(e))?
                    .write_all(format!("{}\n", last_modified.format(TIME_FMT)).as_bytes())
                    .await
                    .map_err(|e| TorrentError::IoError(e))?;
            }
        }
        Err(err) => {
            eprintln!("retrieve last update time error: {:?}", err);
            File::create_new(UPDATE_TIME_RECORD)
                .await
                .map_err(|e| TorrentError::IoError(e))?
                .write_all(format!("{}", last_modified.format(TIME_FMT)).as_bytes())
                .await
                .map_err(|e| TorrentError::IoError(e))?;
        }
    }

    let content: Content = github.get(&req_path, None::<&()>).await.unwrap();
    File::create("tracker_all.txt")
        .await
        .map_err(|e| TorrentError::IoError(e))?
        .write_all(content.decoded_content().unwrap().as_bytes())
        .await
        .map_err(|e| TorrentError::IoError(e))?;
    Ok(())
}

/// 查找上一次tracker文件的更新时间
/// # Error
/// * 打开记录文件可能发生错误
/// * 时间的反序列化可能产生错误
async fn update_time_record() -> Result<DateTime<Local>, TorrentError> {
    let path = Path::new(UPDATE_TIME_RECORD);
    if path.is_file() {
        let mut buf = String::new();
        let _ = File::open(UPDATE_TIME_RECORD)
            .await
            .map_err(|e| TorrentError::IoError(e))?
            .read_to_string(&mut buf)
            .await
            .map_err(|e| TorrentError::IoError(e))?;
        let last_record = buf.rsplit("\n").next().ok_or(TorrentError::InvalidFile)?;
        Ok(DateTime::parse_from_str(last_record, TIME_FMT)
            .map_err(|_| TorrentError::InvalidFile)?
            .with_timezone(&Local))
    } else {
        Err(TorrentError::IoError(io::Error::new(
            io::ErrorKind::NotFound,
            "the old trake file is not found",
        )))
    }
}

#[cfg(test)]
mod tests {

    use chrono::DateTime;
    use tokio::{
        fs::{self, File},
        io::AsyncWriteExt,
    };

    use super::*;

    #[tokio::test]
    async fn test_update_time_record() {
        // 如果没有记录文件
        assert!(update_time_record().await.is_err());
        // 如果文件没有有效记录
        let mut f = File::create(UPDATE_TIME_RECORD).await.unwrap();
        f.write_all("2025-07-22 15:24:44".as_bytes()).await.unwrap();
        assert!(update_time_record().await.is_err());
        drop(f);
        fs::remove_file(UPDATE_TIME_RECORD).await.unwrap();
        // 文件正确，读取的记录也正确
        let mut f = File::create(UPDATE_TIME_RECORD).await.unwrap();
        f.write_all("2025-07-22 15:24:44 +0800".as_bytes())
            .await
            .unwrap();
        let tt = DateTime::parse_from_str("2025-07-22 15:24:44 +0800", TIME_FMT).unwrap();
        assert_eq!(update_time_record().await.unwrap(), tt);
        drop(f);
        fs::remove_file(UPDATE_TIME_RECORD).await.unwrap();
    }
}
