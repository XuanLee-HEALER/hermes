use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
enum CommonError {
    #[error("the provided path is incomplete; it's missing the final element")]
    PathMissingLeafError,
    #[error("the provided path contains characters that are not valid UTF-8 encoded")]
    NonUtf8PathError,
}

/// 根据给定的路径 `[ori_path]` 生成一个相同的路径，但是在路径的最后一个元素加上指定的分隔符 `[c]` 和后缀 `[suffix]`
/// # Error
/// * 如果路径没有“最后一个文件“，会返回错误
pub fn same_path_with(
    ori_path: impl AsRef<Path>,
    suffix: &str,
    c: &str,
) -> Result<impl AsRef<Path>> {
    let ori_path = ori_path.as_ref();
    let mut new_path = PathBuf::from(ori_path);
    let new_name = ori_path
        .file_stem()
        .ok_or(CommonError::PathMissingLeafError)?
        .to_str()
        .ok_or(CommonError::NonUtf8PathError)?;
    let new_name = format!("{}{}{}", new_name, c, suffix);
    new_path.set_file_name(new_name);
    new_path.set_extension(ori_path.extension().unwrap_or(OsStr::new("")));
    Ok(new_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_path_with() {
        assert!(same_path_with("/", "f", "_").is_err());
        assert_eq!(
            same_path_with("/root", "f", "_").unwrap().as_ref(),
            Path::new("/root_f")
        );
        assert_eq!(
            same_path_with("/root/test", "f", "_").unwrap().as_ref(),
            Path::new("/root/test_f")
        );
        assert_eq!(
            same_path_with("/root/test.rs", "f", "_").unwrap().as_ref(),
            Path::new("/root/test_f.rs")
        );
    }
}
