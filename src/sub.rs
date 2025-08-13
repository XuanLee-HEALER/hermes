//! sub module
//!
//! 关于字幕处理的一些方法
//!
//! 1.对于srt类型的文件，调整其时间

mod srt;
// use std::{
//     fmt::{Display, Write as FmtWrite},
//     fs::File,
//     io::{BufRead, BufReader, Write},
//     path::Path,
// };

// use anyhow::Result;
// use chrono::{NaiveTime, TimeDelta};
// use regex::Regex;
// use thiserror::Error;

// use crate::common::same_path_with;

// const LINE_REGEX: &'static str =
//     r#"^(\d{2}):(\d{2}):(\d{2}),(\d{3}) --> (\d{2}):(\d{2}):(\d{2}),(\d{3})$"#;
// const TIME_FMT: &'static str = "%H:%M:%S,%3f";

// fn parse_time_range_line(line: &str, re: &Regex) -> Result<(NaiveTime, NaiveTime)> {
//     let captures = re.captures(line).ok_or(SrtError::InvalidFormat)?;
//     let h1: u32 = captures[1].parse()?;
//     let m1: u32 = captures[2].parse()?;
//     let s1: u32 = captures[3].parse()?;
//     let mil1: u32 = captures[4].parse()?;
//     let h2: u32 = captures[5].parse()?;
//     let m2: u32 = captures[6].parse()?;
//     let s2: u32 = captures[7].parse()?;
//     let mil2: u32 = captures[8].parse()?;
//     let beg = NaiveTime::from_hms_milli_opt(h1, m1, s1, mil1).ok_or(SrtError::InvalidFormat)?;
//     let end = NaiveTime::from_hms_milli_opt(h2, m2, s2, mil2).ok_or(SrtError::InvalidFormat)?;
//     Ok((beg, end))
// }

// /// 对于srt中的时间行 00:00:23,757 --> 00:00:26,726
// /// 增加/减小某个时间 `[ms]`
// /// 返回结果时间行
// fn incr_time_line(line: &str, re: &Regex, ms: i32) -> String {
//     let (beg, end) = parse_time_range_line(line, re).expect("failed to parse the time range line");

//     let (new_beg, delta) =
//         beg.overflowing_add_signed(TimeDelta::try_milliseconds(ms.into()).unwrap());
//     if delta != 0 {
//         return line.into();
//     }
//     let (new_end, _) = end.overflowing_add_signed(TimeDelta::try_milliseconds(ms.into()).unwrap());
//     format!(
//         "{} --> {}",
//         new_beg.format(TIME_FMT).to_string(),
//         new_end.format(TIME_FMT).to_string()
//     )
// }

// struct SrtSubtitles {
//     seq: Vec<SrtItem>,
// }

// impl SrtSubtitles {
//     /// 将srt文件转换为 `SrtSubtitles` 实例
//     /// # Error
//     /// * 文件打开失败
//     /// * 文件内容格式与srt标准不符
//     fn from_file(file: impl AsRef<Path>) -> Result<Self> {
//         let mut result = SrtSubtitles { seq: vec![] };
//         let f = File::open(file)?;
//         let br = BufReader::new(f);
//         let mut buf: Vec<String> = vec![];
//         let re = Regex::new(LINE_REGEX)?;
//         for line in br.lines() {
//             let line = line.expect("an error occurred while read the srt file");
//             if line == "" {
//                 if buf.len() < 3 {
//                     return Err(SrtError::InvalidFormat.into());
//                 }
//                 let id: usize = buf[0].parse()?;
//                 let (beg_time, end_time) = parse_time_range_line(&buf[1], &re)?;
//                 let (text, translate_items) = Self::content_and_translate_str(&buf[2..]);
//                 result.seq.push(SrtItem {
//                     id,
//                     beg_time,
//                     end_time,
//                     text,
//                     translate_items,
//                 });
//                 buf.clear();
//             } else {
//                 buf.push(line);
//             }
//         }
//         Ok(result)
//     }

//     fn content_and_translate_str(lines: &[String]) -> (String, Vec<String>) {
//         let mut content;
//         let mut translate_str = vec![];
//         match lines.len() {
//             0 => unreachable!(),
//             1 => {
//                 content = lines[0].to_owned();
//                 translate_str.push(lines[0].to_owned());
//             }
//             _ => {
//                 content = lines[0].to_owned();
//                 translate_str.push(lines[0].to_owned());
//                 for rem in &lines[1..] {
//                     if rem.starts_with("-") {
//                         content.push('\n');
//                     } else {
//                         content.push(' ');
//                     }
//                     content.push_str(rem);
//                     translate_str.push(rem.to_owned());
//                 }
//             }
//         }
//         (content, translate_str)
//     }
// }

// impl Display for SrtSubtitles {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         for item in self.seq.iter() {
//             f.write_fmt(format_args!(
//                 "id={} text={}\ntranslate_items={:?}\n",
//                 item.id, item.text, item.translate_items
//             ))?;
//         }
//         Ok(())
//     }
// }

pub use srt::OverlapFixMode;
pub use srt::SrtFile;

use std::{fs::File, path::Path};

use crate::{
    common::same_path_with,
    ffmpeg::{FfmpegError, FfmpegTool},
};

/// 将 file 视频容器中的字幕流以srt文件的格式提取到 sub 路径中
pub async fn extract_sub_srt<P: AsRef<Path>>(file: P, sub: P) -> Result<(), FfmpegError> {
    let options = vec![
        "-i",
        file.as_ref().to_str().unwrap_or(""),
        "-map",
        "0:s:0",
        "-c",
        "copy",
        sub.as_ref().to_str().unwrap_or(""),
    ];
    FfmpegTool::Ffmpeg
        .exec_with_options(None::<&'static str>, Some(options))
        .await?;
    Ok(())
}

/// 更新srt字幕文件 `file` 中的所有时间戳向前或向后移动 `ms` 毫秒
/// # Panic
/// 如果移动后的结果超过时间范围则程序退出，不生成修改后的文件
pub fn update_srt_time<P: AsRef<Path>>(file: P, ms: i64, mode: OverlapFixMode) {
    let p = file.as_ref();
    let new_file = same_path_with(p, "mod", "_").expect("new file name error");
    let f = File::open(p).expect("failed to open the subtitle file");
    let mut srt_file = SrtFile::read(f).expect("failed to parse the srt file");
    srt_file
        .adjust_timestamps(ms, mode)
        .expect("failed to adjust the timestamps");
    let mut nf = File::create(new_file).expect("failed to create the new srt file");
    srt_file
        .write(&mut nf)
        .expect("failed to write content to new srt file");
}

// #[cfg(test)]
// mod tests {
//     use std::{
//         fs::{self, File},
//         io::Write,
//     };

//     use super::*;

//     #[test]
//     fn test_parse_srt() {
//         let file = "parasite.srt";
//         let srt = SrtSubtitles::from_file(file).unwrap();
//         assert_eq!(srt.seq.len(), 1659);
//         println!("{}", srt);
//     }

//     #[test]
//     #[should_panic(expected = "invalid srt subtitle file format")]
//     fn test_parse_time_range_line() {
//         let re = Regex::new(LINE_REGEX).unwrap();
//         let line = "00:00:23,757 --> 00:00:26,726";
//         let (beg, end) = (
//             NaiveTime::from_hms_milli_opt(0, 0, 23, 757).unwrap(),
//             NaiveTime::from_hms_milli_opt(0, 0, 26, 726).unwrap(),
//         );
//         let (beg1, end1) = parse_time_range_line(line, &re).unwrap();
//         assert_eq!(beg1, beg);
//         assert_eq!(end1, end);
//         let line = " xxx ";
//         let _ = parse_time_range_line(line, &re).unwrap();
//     }

//     #[test]
//     fn test_incr_time_line() {
//         let re = Regex::new(LINE_REGEX).unwrap();
//         assert_eq!(
//             incr_time_line("00:00:23,757 --> 00:00:26,726", &re, 100),
//             "00:00:23,857 --> 00:00:26,826"
//         );
//         assert_eq!(
//             incr_time_line("00:00:23,757 --> 00:00:26,726", &re, 300),
//             "00:00:24,057 --> 00:00:27,026"
//         );
//         assert_eq!(
//             incr_time_line("00:00:23,757 --> 00:00:26,726", &re, -100),
//             "00:00:23,657 --> 00:00:26,626"
//         );
//         assert_eq!(
//             incr_time_line("00:00:23,757 --> 00:00:26,726", &re, -800),
//             "00:00:22,957 --> 00:00:25,926"
//         );
//     }

//     #[test]
//     fn test_update_srt_time() {
//         let mut f = File::create_new("test.srt").unwrap();
//         let ori_str = r#"1
// 00:00:19,319 --> 00:00:23,278
// 导演:奥森威尔斯

// 2
// 00:00:23,757 --> 00:00:26,726
// 片名:大国民

// 3
// 00:00:37,337 --> 00:00:40,306
// 禁止入内

// 4
// 00:02:33,653 --> 00:02:36,622
// 玫瑰花蕾"#;
//         f.write_all(ori_str.as_bytes()).unwrap();
//         update_srt_time("test.srt", 100);
//         drop(f);
//         fs::remove_file("test.srt").unwrap()
//     }
// }
