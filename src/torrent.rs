use std::path::Path;

use anyhow::Error as AnyError;
use chrono::{DateTime, Local};
use octocrab::models::repos::Content;
use thiserror::Error;
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt},
};

const TIME_FMT: &'static str = "%Y-%m-%d %H:%M:%S %z";
const UPDATE_TIME_RECORD: &'static str = "update_record";
const OWNER: &'static str = "ngosang";
const REPO: &'static str = "trackerslist";

#[derive(Debug, Error)]
enum Error {
    #[error("unexpected file type or file not exists")]
    NoValidFile,
    #[error("invalid value to use")]
    InvalidValue,
    #[error("don't need to update")]
    NoUpdate,
}

pub async fn download_newest_tracker() -> Result<(), AnyError> {
    let req_path = format!("/repos/{}/{}/contents/trackers_all.txt", OWNER, REPO);
    let github = octocrab::instance();
    let content = github._get(&req_path).await?;
    let last_modified = content
        .headers()
        .get("last-modified")
        .ok_or(Error::InvalidValue)?
        .to_str()?;
    let last_modified = DateTime::parse_from_rfc2822(last_modified)?.with_timezone(&Local);
    match update_time_record().await {
        Ok(last_update) => {
            if last_modified <= last_update {
                return Err(Error::NoUpdate.into());
            } else {
                let mut f = File::options()
                    .write(true)
                    .append(true)
                    .open(UPDATE_TIME_RECORD)
                    .await?;
                f.write_all(format!("{}\n", last_modified.format(TIME_FMT)).as_bytes())
                    .await?;
            }
        }
        Err(err) => {
            eprintln!("retrieve last update time error: {:?}", err);
            File::create_new(UPDATE_TIME_RECORD)
                .await?
                .write_all(format!("{}", last_modified.format(TIME_FMT)).as_bytes())
                .await?;
        }
    }

    let content: Content = github.get(&req_path, None::<&()>).await.unwrap();
    let mut f = File::create("tracker_all.txt").await?;
    f.write_all(content.decoded_content().unwrap().as_bytes())
        .await?;
    Ok(())
}

/// 查找上一次tracker文件的更新时间
/// # Error
/// * 打开记录文件可能发生错误
/// * 时间的反序列化可能产生错误
async fn update_time_record() -> Result<DateTime<Local>, AnyError> {
    if is_file(UPDATE_TIME_RECORD).await? {
        let mut buf = String::new();
        let _ = File::open(UPDATE_TIME_RECORD)
            .await?
            .read_to_string(&mut buf)
            .await?;
        let last_record = buf.rsplit("\n").next().ok_or(Error::NoValidFile)?;
        Ok(DateTime::parse_from_str(last_record, TIME_FMT)?.with_timezone(&Local))
    } else {
        Err(Error::NoValidFile.into())
    }
}

/// 判断指定路径是否为文件类型，如果是链接类型，应该判断其指向的真实文件类型
/// # Errors
/// 如果文件没有list权限或者打开文件获取元数据失败会返回错误
async fn is_file(path: impl AsRef<Path>) -> Result<bool, AnyError> {
    Ok(fs::try_exists(&path).await? && File::open(&path).await?.metadata().await?.is_file())
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
    async fn test_is_file() {
        const TEST_FILE_NAME: &'static str = "test.txt";
        const TEST_DIR_NAME: &'static str = "test";
        const TEST_FILE_LINK: &'static str = "test_file_link";
        const TEST_DIR_LINK: &'static str = "test_dir_link";
        // file
        let f = File::create(TEST_FILE_NAME).await.unwrap();
        assert_eq!(is_file(TEST_FILE_NAME).await.unwrap(), true);
        drop(f);
        // directory
        let _ = fs::create_dir(TEST_DIR_NAME).await.unwrap();
        assert_eq!(is_file(TEST_DIR_NAME).await.unwrap(), false);
        // link
        let _ = fs::symlink(TEST_FILE_NAME, TEST_FILE_LINK).await.unwrap();
        assert_eq!(is_file(TEST_FILE_LINK).await.unwrap(), true);
        let _ = fs::symlink(TEST_DIR_NAME, TEST_DIR_LINK).await.unwrap();
        assert_eq!(is_file(TEST_DIR_LINK).await.unwrap(), false);
        fs::remove_file(TEST_FILE_LINK).await.unwrap();
        fs::remove_file(TEST_DIR_LINK).await.unwrap();
        fs::remove_file(TEST_FILE_NAME).await.unwrap();
        fs::remove_dir(TEST_DIR_NAME).await.unwrap();
    }

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
