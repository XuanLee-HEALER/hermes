//! sub module
//!
//! 关于字幕处理的一些方法
//!
//! 1.对于srt类型的文件，调整其时间

use std::{
    fmt::Write as FmtWrite,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
};

use chrono::{NaiveTime, TimeDelta};
use regex::Regex;

/// 对于srt中的时间行 00:00:23,757 --> 00:00:26,726
/// 增加/减小某个时间 `[ms]`
/// 返回结果时间行
fn incr_time_line(line: &str, ms: i32) -> String {
    const LINE_REGEX: &'static str =
        r#"^(\d{2}):(\d{2}):(\d{2}),(\d{3}) --> (\d{2}):(\d{2}):(\d{2}),(\d{3})$"#;
    const TIME_FMT: &'static str = "%H:%M:%S,%3f";
    let re = Regex::new(LINE_REGEX).expect("regex fault");
    if let Some(captures) = re.captures(line) {
        let h1: u32 = captures[1].parse().unwrap();
        let m1: u32 = captures[2].parse().unwrap();
        let s1: u32 = captures[3].parse().unwrap();
        let mil1: u32 = captures[4].parse().unwrap();
        let h2: u32 = captures[5].parse().unwrap();
        let m2: u32 = captures[6].parse().unwrap();
        let s2: u32 = captures[7].parse().unwrap();
        let mil2: u32 = captures[8].parse().unwrap();
        let beg = NaiveTime::from_hms_milli_opt(h1, m1, s1, mil1).unwrap();
        let end = NaiveTime::from_hms_milli_opt(h2, m2, s2, mil2).unwrap();
        let (new_beg, _) =
            beg.overflowing_add_signed(TimeDelta::try_milliseconds(ms.into()).unwrap());
        let (new_end, _) =
            end.overflowing_add_signed(TimeDelta::try_milliseconds(ms.into()).unwrap());
        format!(
            "{} --> {}",
            new_beg.format(TIME_FMT).to_string(),
            new_end.format(TIME_FMT).to_string()
        )
    } else {
        panic!("time line incorrect format")
    }
}

pub fn update_srt_time(file: impl AsRef<Path>, ms: i32) {
    let p = file.as_ref();
    let new_file_name = p.parent().unwrap().join(format!(
        "{}_handled.srt",
        p.file_stem().unwrap().to_str().unwrap()
    ));
    let f = File::open(file).expect("failed to open the subtitle file");
    let mut nf = File::create(new_file_name).expect("failed to create the new file");
    let br = BufReader::new(f);
    let mut update_flag = false;
    let mut buf = String::new();
    for line in br.lines() {
        let line = line.unwrap();
        let wl = if let Ok(_) = line.parse::<i32>() {
            update_flag = true;
            line
        } else if update_flag {
            // 更新行
            update_flag = false;
            incr_time_line(&line, ms)
        } else {
            line
        };
        buf.write_fmt(format_args!("{}\n", wl)).unwrap();
    }
    nf.write_all(buf.as_bytes()).unwrap();
    nf.flush().unwrap();
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io::Write,
    };

    use crate::sub::{incr_time_line, update_srt_time};

    #[test]
    fn test_incr_time_line() {
        assert_eq!(
            incr_time_line("00:00:23,757 --> 00:00:26,726", 100),
            "00:00:23,857 --> 00:00:26,826"
        );
        assert_eq!(
            incr_time_line("00:00:23,757 --> 00:00:26,726", 300),
            "00:00:24,057 --> 00:00:27,026"
        );
        assert_eq!(
            incr_time_line("00:00:23,757 --> 00:00:26,726", -100),
            "00:00:23,657 --> 00:00:26,626"
        );
        assert_eq!(
            incr_time_line("00:00:23,757 --> 00:00:26,726", -800),
            "00:00:22,957 --> 00:00:25,926"
        );
    }

    #[test]
    fn test_update_srt_time() {
        let mut f = File::create_new("test.srt").unwrap();
        let ori_str = r#"1
00:00:19,319 --> 00:00:23,278
导演:奥森威尔斯

2
00:00:23,757 --> 00:00:26,726
片名:大国民

3
00:00:37,337 --> 00:00:40,306
禁止入内

4
00:02:33,653 --> 00:02:36,622
玫瑰花蕾"#;
        f.write_all(ori_str.as_bytes()).unwrap();
        update_srt_time("test.srt", 100);
        drop(f);
        fs::remove_file("test.srt").unwrap()
    }
}
