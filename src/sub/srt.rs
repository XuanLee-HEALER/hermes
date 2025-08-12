//! srt module
//! 负责对srt文件的解析和生成工作
//!
//! # 解析（parse）
//! 从一个 impl Read 解析，

use std::{
    fmt::Display,
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    result,
    sync::LazyLock,
};

use chrono::Duration;
use regex::Regex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SrtError {
    #[error("Parse time error: {0}")]
    ParseTimeError(String),
    #[error("Parse error occurred while read content: {0}")]
    ParseTextError(String),
    #[error("IO error")]
    IoError(io::Error),
}

pub type Result<T> = result::Result<T, SrtError>;

#[derive(Debug)]
pub struct SrtFile {
    entries: Vec<SubtitleEntry>,
}

impl SrtFile {
    fn read<R: Read>(r: R) -> Result<Self> {
        let br = BufReader::new(r);
        let mut lines = br
            .lines()
            .map(|line| line.map_err(|e| SrtError::IoError(e)));
        let mut entries = Vec::new();

        loop {
            let index_line = match lines.next() {
                Some(Ok(l)) => l,
                Some(Err(e)) => return Err(e),
                None => break,
            };

            let index_line = index_line.trim();
            if index_line.is_empty() {
                continue;
            }
            let index = index_line
                .parse::<u32>()
                .map_err(|_| SrtError::ParseTextError(index_line.to_string()))?;

            let ts_line = lines
                .next()
                .ok_or_else(|| SrtError::ParseTextError("Missing timestamp line".to_string()))??;
            let timestamp = SrtTime::from_line(&ts_line)?;

            let mut text = String::new();
            while let Some(Ok(l)) = lines.next() {
                if l.trim().is_empty() {
                    break;
                }
                if !text.is_empty() {
                    #[cfg(target_family = "unix")]
                    text.push('\n');
                    #[cfg(target_family = "windows")]
                    text.push_str("\r\n");
                }
                text.push_str(&l);
            }

            entries.push(SubtitleEntry {
                index,
                timestamp,
                text,
            });
        }

        Ok(Self { entries })
    }

    fn write<W: Write>(&self, w: &mut W) -> Result<()> {
        let mut bw = BufWriter::new(w);
        for entry in &self.entries {
            #[cfg(target_family = "unix")]
            bw.write_fmt(format_args!(
                "{}\n{}\n{}\n\n",
                entry.index, entry.timestamp, entry.text
            ))
            .map_err(|e| SrtError::IoError(e))?;
        }
        bw.flush().map_err(|e| SrtError::IoError(e))?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct SubtitleEntry {
    pub index: u32,
    timestamp: SrtTime,
    pub text: String,
}

static SRT_TIME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\d{2}):(\d{2}):(\d{2}),(\d{3}) --> (\d{2}):(\d{2}):(\d{2}),(\d{3})$").unwrap()
});

#[derive(Debug, Default, Clone)]
struct SrtTime {
    beg_ts: Duration,
    end_ts: Duration,
    dur: Duration,
}

impl Display for SrtTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} --> {}",
            Self::dur_to_timestamp(self.beg_ts),
            Self::dur_to_timestamp(self.end_ts)
        ))
    }
}

impl SrtTime {
    pub fn new(beg_ts: Duration, end_ts: Duration) -> Self {
        Self {
            beg_ts,
            end_ts,
            dur: end_ts - beg_ts,
        }
    }

    pub fn entry_dur_secs(&self) -> i64 {
        self.dur.num_seconds()
    }

    pub fn entry_dur_millis(&self) -> i64 {
        self.dur.num_milliseconds()
    }

    fn dur_to_timestamp(dur: Duration) -> String {
        let ms = dur.num_milliseconds();
        let sig = if ms.is_negative() { "-" } else { "" };
        let ms = ms.abs();
        let s = ms / 1_000;
        let ms = ms % 1_000;
        let m = s / 60;
        let s = s % 60;
        let h = m / 60;
        let m = m % 60;
        return format!("{}{:02}:{:02}:{:02},{:03}", sig, h, m, s, ms);
    }

    fn from_line(ts_line: &str) -> Result<Self> {
        if SRT_TIME_RE.is_match(ts_line) {
            let mat = SRT_TIME_RE.captures(ts_line).unwrap();
            let (beg_h, beg_m, beg_s, beg_ms) = (&mat[1], &mat[2], &mat[3], &mat[4]);
            let beg_ts = Self::from_seg(beg_h, beg_m, beg_s, beg_ms)?;
            let (end_h, end_m, end_s, end_ms) = (&mat[5], &mat[6], &mat[7], &mat[8]);
            let end_ts = Self::from_seg(end_h, end_m, end_s, end_ms)?;
            if end_ts < beg_ts {
                return Err(SrtError::ParseTimeError(
                    "endtime is greater then begintime".to_string(),
                ));
            }
            Ok(Self {
                beg_ts,
                end_ts,
                dur: end_ts - beg_ts,
            })
        } else {
            Err(SrtError::ParseTimeError(ts_line.to_string()))
        }
    }

    fn from_seg(h: &str, m: &str, s: &str, ms: &str) -> Result<Duration> {
        let mut start = Duration::zero();
        let h: i64 = h.parse().map_err(|_| {
            SrtError::ParseTimeError(format!("invalid hour value {}", h.to_string()))
        })?;
        let m: i64 = m.parse().map_err(|_| {
            SrtError::ParseTimeError(format!("invalid minute value {}", m.to_string()))
        })?;
        let s: i64 = s.parse().map_err(|_| {
            SrtError::ParseTimeError(format!("invalid second value {}", s.to_string()))
        })?;
        let ms: i64 = ms.parse().map_err(|_| {
            SrtError::ParseTimeError(format!("invalid millisecond value {}", ms.to_string()))
        })?;
        if h > 99 || h < 0 || m > 59 || m < 0 || s > 59 || s < 0 || ms > 999 || ms < 0 {
            return Err(SrtError::ParseTimeError(format!(
                "invalid timestamp: {:02}:{:02}:{:02}.{:03}",
                h, m, s, ms
            )));
        }
        start += Duration::hours(h)
            + Duration::minutes(m)
            + Duration::seconds(s)
            + Duration::milliseconds(ms);
        Ok(start)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Seek, SeekFrom};

    use super::*;
    use chrono::Duration;

    #[test]
    fn test_write_single_entry() {
        let entry = SubtitleEntry {
            index: 1,
            timestamp: SrtTime::new(Duration::seconds(1), Duration::seconds(2)),
            text: "Hello world!".to_string(),
        };
        let srt_file = SrtFile {
            entries: vec![entry],
        };
        let mut output = Cursor::new(Vec::new());
        srt_file.write(&mut output).unwrap();
        output.seek(SeekFrom::Start(0)).unwrap();
        let mut output_str = String::new();
        output.read_to_string(&mut output_str).unwrap();

        let expected = "1\n00:00:01,000 --> 00:00:02,000\nHello world!\n\n";
        assert_eq!(&output_str, expected);
    }

    #[test]
    fn test_write_multiple_entries() {
        let entry1 = SubtitleEntry {
            index: 1,
            timestamp: SrtTime::new(Duration::seconds(1), Duration::seconds(2)),
            text: "Hello.".to_string(),
        };
        let entry2 = SubtitleEntry {
            index: 2,
            timestamp: SrtTime::new(Duration::seconds(3), Duration::seconds(4)),
            text: "World.".to_string(),
        };
        let srt_file = SrtFile {
            entries: vec![entry1, entry2],
        };

        let mut output = Cursor::new(Vec::new());
        srt_file
            .write(&mut output)
            .expect("Failed to write the srt entries");

        let expected = "1\n00:00:01,000 --> 00:00:02,000\nHello.\n\n2\n00:00:03,000 --> 00:00:04,000\nWorld.\n\n";
        output.seek(SeekFrom::Start(0)).unwrap();
        let mut output_str = String::new();
        output.read_to_string(&mut output_str).unwrap();
        assert_eq!(&output_str, expected);
    }

    #[test]
    fn test_standard_duration() {
        // 1小时15分30秒500毫秒
        let dur = Duration::hours(1)
            + Duration::minutes(15)
            + Duration::seconds(30)
            + Duration::milliseconds(500);
        assert_eq!(SrtTime::dur_to_timestamp(dur), "01:15:30,500");
    }

    #[test]
    fn test_just_minutes() {
        // 12分钟
        let dur = Duration::minutes(12);
        assert_eq!(SrtTime::dur_to_timestamp(dur), "00:12:00,000");
    }

    #[test]
    fn test_just_milliseconds() {
        // 999毫秒
        let dur = Duration::milliseconds(999);
        assert_eq!(SrtTime::dur_to_timestamp(dur), "00:00:00,999");
    }

    // --- 边界情况测试 ---
    #[test]
    fn test_zero_duration() {
        // 0毫秒
        let dur = Duration::zero();
        assert_eq!(SrtTime::dur_to_timestamp(dur), "00:00:00,000");
    }

    #[test]
    fn test_max_srt_duration() {
        // SRT 最大时间：99:59:59,999
        let dur = Duration::hours(99)
            + Duration::minutes(59)
            + Duration::seconds(59)
            + Duration::milliseconds(999);
        assert_eq!(SrtTime::dur_to_timestamp(dur), "99:59:59,999");
    }

    #[test]
    fn test_overflow_to_minutes() {
        // 59秒999毫秒加1毫秒，进位到1分钟
        let dur = Duration::seconds(59) + Duration::milliseconds(1000);
        assert_eq!(SrtTime::dur_to_timestamp(dur), "00:01:00,000");
    }

    #[test]
    fn test_overflow_to_hours() {
        // 59分59秒999毫秒加1毫秒，进位到1小时
        let dur = Duration::minutes(59) + Duration::seconds(59) + Duration::milliseconds(1000);
        assert_eq!(SrtTime::dur_to_timestamp(dur), "01:00:00,000");
    }

    // --- 特殊情况测试 ---
    #[test]
    fn test_large_duration_beyond_srt_limit() {
        // 超过100小时，你的函数仍然能正确处理，但它会超出SRT的格式
        let dur = Duration::hours(101);
        assert_eq!(SrtTime::dur_to_timestamp(dur), "101:00:00,000");
    }

    // --- 负数时间测试 ---
    #[test]
    fn test_negative_duration() {
        // 负数时间，SRT规范中不常见，但你的函数应该能正确处理
        let dur = Duration::hours(-1);
        // chrono::Duration::num_milliseconds() 对负数返回负数，所以结果会是负数
        assert_eq!(SrtTime::dur_to_timestamp(dur), "-01:00:00,000");
    }

    // 辅助函数，将字符串转换为 Read Trait 的实现
    fn read_from_str(s: &str) -> Result<SrtFile> {
        SrtFile::read(Cursor::new(s))
    }

    // --- 正常情况测试 ---
    #[test]
    fn test_valid_single_entry() {
        let srt_content = "1\n00:00:01,000 --> 00:00:02,000\nHello world!\n\n";
        let srt_file = read_from_str(srt_content).unwrap();
        assert_eq!(srt_file.entries.len(), 1);
        let entry = &srt_file.entries[0];
        assert_eq!(entry.index, 1);
        assert_eq!(entry.timestamp.beg_ts, Duration::seconds(1));
        assert_eq!(entry.timestamp.end_ts, Duration::seconds(2));
        assert_eq!(entry.text, "Hello world!");
    }

    #[test]
    fn test_valid_multiple_entries() {
        let srt_content = "1\n00:00:01,000 --> 00:00:02,000\nHello.\n\n2\n00:00:03,000 --> 00:00:04,000\nWorld.\n\n";
        let srt_file = read_from_str(srt_content).unwrap();
        assert_eq!(srt_file.entries.len(), 2);
        assert_eq!(srt_file.entries[0].text, "Hello.");
        assert_eq!(srt_file.entries[1].text, "World.");
    }

    #[test]
    fn test_multiline_text() {
        let srt_content = "1\n00:00:01,000 --> 00:00:02,000\nHello\nworld!\n\n";
        let srt_file = read_from_str(srt_content).unwrap();
        assert_eq!(srt_file.entries.len(), 1);
        assert_eq!(srt_file.entries[0].text, "Hello\nworld!");
    }

    #[test]
    fn test_file_without_trailing_empty_line() {
        let srt_content = "1\n00:00:01,000 --> 00:00:02,000\nHello world!";
        let srt_file = read_from_str(srt_content).unwrap();
        assert_eq!(srt_file.entries.len(), 1);
        assert_eq!(srt_file.entries[0].text, "Hello world!");
    }

    #[test]
    fn test_invalid_index() {
        let srt_content = "one\n00:00:01,000 --> 00:00:02,000\nHello.";
        let err = read_from_str(srt_content).unwrap_err();
        assert!(matches!(err, SrtError::ParseTextError(_)));
    }

    #[test]
    fn test_invalid_timestamp_format() {
        let srt_content = "1\n00:00:01,000 - 00:00:02,000\nHello."; // 缺少 '>'
        let err = read_from_str(srt_content).unwrap_err();
        assert!(matches!(err, SrtError::ParseTimeError(_)));
    }

    #[test]
    fn test_empty_file() {
        let srt_content = "";
        let srt_file = read_from_str(srt_content).unwrap();
        assert_eq!(srt_file.entries.len(), 0);
    }

    #[test]
    fn test_only_empty_lines() {
        let srt_content = "\n\n\n";
        let srt_file = read_from_str(srt_content).unwrap();
        assert_eq!(srt_file.entries.len(), 0);
    }

    #[test]
    fn test_valid_timestamp_line() {
        // 正常的时间戳行
        let line = "00:01:10,250 --> 00:01:15,500";
        let result = SrtTime::from_line(line).unwrap();

        let expected_beg =
            Duration::minutes(1) + Duration::seconds(10) + Duration::milliseconds(250);
        let expected_end =
            Duration::minutes(1) + Duration::seconds(15) + Duration::milliseconds(500);

        assert_eq!(result.beg_ts, expected_beg);
        assert_eq!(result.end_ts, expected_end);
    }

    // --- 边界情况测试 ---
    #[test]
    fn test_boundary_timestamps() {
        // 1. 开始时间为0，结束时间为最大值
        let line = "00:00:00,000 --> 99:59:59,999";
        let result = SrtTime::from_line(line).unwrap();

        let expected_beg = Duration::zero();
        let expected_end = Duration::hours(99)
            + Duration::minutes(59)
            + Duration::seconds(59)
            + Duration::milliseconds(999);

        assert_eq!(result.beg_ts, expected_beg);
        assert_eq!(result.end_ts, expected_end);

        // 2. 开始和结束时间相同
        let line = "01:00:00,000 --> 01:00:00,000";
        let result = SrtTime::from_line(line).unwrap();

        let expected_beg_end = Duration::hours(1);
        assert_eq!(result.beg_ts, expected_beg_end);
        assert_eq!(result.end_ts, expected_beg_end);
    }

    // --- 错误情况测试 ---
    #[test]
    fn test_invalid_line_format() {
        // 1. 格式不匹配（缺少 -->）
        let line = "00:01:10,250 00:01:15,500";
        let err = SrtTime::from_line(line).unwrap_err();
        assert!(matches!(err, SrtError::ParseTimeError(_)));
        assert!(format!("{}", err).contains(line));

        // 2. 格式不匹配（缺少毫秒）
        let line = "00:01:10,250 --> 00:01:15";
        let err = SrtTime::from_line(line).unwrap_err();
        assert!(matches!(err, SrtError::ParseTimeError(_)));
        assert!(format!("{}", err).contains(line));

        // 3. 结束时间早于开始时间
        let line = "00:01:15,500 --> 00:01:10,250";
        let err = SrtTime::from_line(line).unwrap_err();
        assert!(matches!(err, SrtError::ParseTimeError(_)));
        assert!(format!("{}", err).contains("endtime is greater then begintime"));
    }

    #[test]
    fn test_valid_time_components() {
        // 1. 正常时间
        let result = SrtTime::from_seg("01", "15", "30", "500").unwrap();
        let expected = Duration::hours(1)
            + Duration::minutes(15)
            + Duration::seconds(30)
            + Duration::milliseconds(500);
        assert_eq!(result, expected);

        // 2. 只有毫秒
        let result = SrtTime::from_seg("00", "00", "00", "123").unwrap();
        let expected = Duration::milliseconds(123);
        assert_eq!(result, expected);

        // 3. 所有部分都是零
        let result = SrtTime::from_seg("00", "00", "00", "000").unwrap();
        let expected = Duration::zero();
        assert_eq!(result, expected);

        // 4. 两位数的小时
        let result = SrtTime::from_seg("12", "00", "00", "000").unwrap();
        let expected = Duration::hours(12);
        assert_eq!(result, expected);
    }

    // --- 边界情况测试 ---
    #[test]
    fn test_boundary_values() {
        // 1. 最大时间戳
        let result = SrtTime::from_seg("99", "59", "59", "999").unwrap();
        let expected = Duration::hours(99)
            + Duration::minutes(59)
            + Duration::seconds(59)
            + Duration::milliseconds(999);
        assert_eq!(result, expected);

        // 2. 毫秒为 0
        let result = SrtTime::from_seg("01", "02", "03", "000").unwrap();
        let expected = Duration::hours(1) + Duration::minutes(2) + Duration::seconds(3);
        assert_eq!(result, expected);

        // 3. 00:00:00.000
        let result = SrtTime::from_seg("00", "00", "00", "000").unwrap();
        assert_eq!(result, Duration::zero());
    }

    // --- 错误情况测试 ---
    #[test]
    fn test_invalid_input_and_parsing_errors() {
        // 1. 超过最大小时数
        let err = SrtTime::from_seg("100", "00", "00", "000").unwrap_err();
        assert!(matches!(err, SrtError::ParseTimeError(_)));
        assert!(format!("{}", err).contains("invalid timestamp"));

        // 2. 超过最大分钟数
        let err = SrtTime::from_seg("00", "60", "00", "000").unwrap_err();
        assert!(matches!(err, SrtError::ParseTimeError(_)));
        assert!(format!("{}", err).contains("invalid timestamp"));

        // 3. 超过最大秒数
        let err = SrtTime::from_seg("00", "00", "60", "000").unwrap_err();
        assert!(matches!(err, SrtError::ParseTimeError(_)));
        assert!(format!("{}", err).contains("invalid timestamp"));

        // 4. 超过最大毫秒数
        let err = SrtTime::from_seg("00", "00", "00", "1000").unwrap_err();
        assert!(matches!(err, SrtError::ParseTimeError(_)));
        assert!(format!("{}", err).contains("invalid timestamp"));

        // 5. 非数字输入
        let err = SrtTime::from_seg("a", "00", "00", "000").unwrap_err();
        assert!(matches!(err, SrtError::ParseTimeError(_)));
        assert!(format!("{}", err).contains("invalid hour"));

        // 6. 负数输入（虽然 SRT 不会出现，但健壮性检查是好的）
        let err = SrtTime::from_seg("-1", "00", "00", "000").unwrap_err();
        assert!(matches!(err, SrtError::ParseTimeError(_)));
        assert!(format!("{}", err).contains("invalid timestamp"));
    }
}
