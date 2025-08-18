//! sub module
//!
//! 关于字幕处理的一些方法
//!
//! 1.对于srt类型的文件，调整其时间

mod srt;

use clap::ValueEnum;
use prettytable::Cell;
use prettytable::Row;
use prettytable::Table;
use serde::Deserialize;
use serde::Serialize;
pub use srt::OverlapFixMode;
pub use srt::SrtFile;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::io::stdin;

use std::io::Cursor;
use std::{fs::File, path::Path};

use crate::common::read_multiple_file_to_string;
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

/// 视频流的顶层结构体，用于解析 ffprobe 的 JSON 输出。
///
/// ffprobe -show_streams -select_streams s 命令的输出格式为：
/// {
///   "streams": [
///     { ... }, // 字幕流 1
///     { ... }  // 字幕流 2
///   ]
/// }
#[derive(Debug, Deserialize, Serialize)]
struct FfprobeOutput {
    streams: Vec<FfprobeStream>,
}

/// 单个流的详细信息
#[derive(Debug, Deserialize, Serialize)]
struct FfprobeStream {
    /// 流的索引，从 0 开始。
    index: u32,
    /// 编解码器类型
    codec_type: String,
    /// 编解码器名称
    codec_name: String,
    // 开始时间
    start_time: String,
    // 持续时间，单位ms
    duration_ts: u32,
    /// 元数据标签，其中可能包含语言信息
    tags: Option<FfprobeTags>,
}

/// 流的标签信息，用于获取语言等元数据
#[derive(Debug, Deserialize, Serialize)]
struct FfprobeTags {
    /// 字幕语言，如 "chi" (中文), "eng" (英文) 等
    language: Option<String>,
    /// 标题或流名称
    title: Option<String>,
}

/// 最终返回给调用者的字幕流信息结构体
#[derive(Debug, Clone, Serialize)]
pub struct SubtitleStreamInfo {
    /// 流的索引
    pub index: u32,
    /// 编解码器名称
    pub codec_name: String,
    /// 持续时间，单位ms
    pub duration: u32,
    /// 语言标签，如果存在的话
    pub language: Option<String>,
    /// 标题标签，如果存在的话
    pub title: Option<String>,
}

/// 定义输出格式的枚举类型。
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    /// 输出为 JSON 格式
    Json,
    /// 输出为表格格式
    #[clap(name = "tab")]
    Table,
    /// 输出为列表格式
    List,
}

/// 异步函数，列出视频文件中所有的字幕流信息并直接打印。
///
/// 该函数利用 ffprobe 工具解析视频文件，并根据指定的格式打印字幕流的元数据。
///
/// # 参数
/// * `file`: 一个实现了 `AsRef<Path>` trait 的文件路径。
/// * `format`: 指定输出的格式，可以是 `OutputFormat::Json` 或 `OutputFormat::Table`。
///
/// # 返回值
/// `Result` 包含一个空元组 `()` 或一个实现了 `std::error::Error` trait 的 Box 对象。
///
/// # 依赖
/// 你需要在你的 `Cargo.toml` 中添加 `prettytable-rs` 和 `serde_json` 依赖：
/// ```toml
/// [dependencies]
/// serde = { version = "1.0", features = ["derive"] }
/// serde_json = "1.0"
/// prettytable-rs = "0.10.0"
/// ```
pub async fn list_all_subtitle_stream(
    file: impl AsRef<Path>,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    // 将文件路径转换为 PathBuf
    let file_path = file.as_ref();
    if !file_path.exists() {
        return Err(format!("文件不存在: {}", file_path.display()).into());
    }

    // 1. 构建 ffprobe 命令
    let mut args: Vec<String> = vec![
        "-v".to_string(),
        "quiet".to_string(),
        "-print_format".to_string(),
        "json".to_string(),
        "-show_streams".to_string(),
        "-select_streams".to_string(),
        "s".to_string(),
        "--".to_string(), // 分隔符，将选项与文件名分开
    ];
    args.push(file_path.to_string_lossy().to_string());

    // 2. 调用外部 ffprobe 工具并获取标准输出
    // 假设你有一个名为 `run_external_command` 的异步函数来执行此操作。
    // 该函数应返回一个 `Result<(String, String), Box<dyn std::error::Error>>`。
    let (stdout, _) = FfmpegTool::Ffprobe
        .exec_with_options(None::<&'static str>, Some(args))
        .await?;

    // 如果 ffprobe 输出为空，说明没有找到字幕流。
    if stdout.is_empty() {
        println!("未找到任何字幕流。");
        return Ok(());
    }

    // 3. 解析 JSON 输出
    let output: FfprobeOutput = serde_json::from_str(&stdout)?;
    let subtitle_streams: Vec<SubtitleStreamInfo> = output
        .streams
        .into_iter()
        .map(|stream| SubtitleStreamInfo {
            index: stream.index,
            codec_name: stream.codec_name,
            language: stream.tags.as_ref().and_then(|tags| tags.language.clone()),
            title: stream.tags.as_ref().and_then(|tags| tags.title.clone()),
            duration: stream.duration_ts,
        })
        .collect();

    // 4. 根据指定的格式打印结果
    match format {
        OutputFormat::Json => {
            let json_output = serde_json::to_string_pretty(&subtitle_streams)?;
            println!("{}", json_output);
        }
        OutputFormat::Table => {
            let mut table = Table::new();
            table.add_row(Row::new(vec![
                Cell::new("Index"),
                Cell::new("Codec Name"),
                Cell::new("Duration(ms)"),
                Cell::new("Language"),
                Cell::new("Title"),
            ]));
            for stream in subtitle_streams {
                table.add_row(Row::new(vec![
                    Cell::new(&stream.index.to_string()),
                    Cell::new(&stream.codec_name),
                    Cell::new(&stream.duration.to_string()),
                    Cell::new(&stream.language.unwrap_or_else(|| "N/A".to_string())),
                    Cell::new(&stream.title.unwrap_or_else(|| "N/A".to_string())),
                ]));
            }
            table.printstd();
        }
        OutputFormat::List => {
            for stream in subtitle_streams {
                println!(
                    "Index({}) Codec Name({}) Duration({}ms) Language({}) Title({})",
                    stream.index,
                    stream.codec_name,
                    stream.duration,
                    stream.language.unwrap_or_else(|| "N/A".to_string()),
                    stream.title.unwrap_or_else(|| "N/A".to_string())
                );
            }
        }
    }

    Ok(())
}

pub async fn compare_two_srt_file<P: AsRef<Path> + Send + 'static>(
    file_1: P,
    file_2: P,
    interactive: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file_contents = read_multiple_file_to_string(vec![file_1, file_2]).await?;
    let srt_1 = SrtFile::read(Cursor::new(file_contents.remove(0)))?;
    let srt_2 = SrtFile::read(Cursor::new(file_contents.remove(0)))?;
    let mut print_stream = srt_1.iter().zip(srt_2.iter()).map(|(entry_1, entry_2)| {
        let l_str_vec = entry_1.to_entry_str();
        let r_str_vec = entry_2.to_entry_str();
        let mut table = Table::new();
        l_str_vec.iter().zip(r_str_vec.iter()).for_each(|(l, r)| {
            table.add_row(Row::new(vec![Cell::new(l), Cell::new(r)]));
        });
        table
    });
    let mut reader = BufReader::new(stdin());
    let mut input_buf = String::new();
    let mut counter = 1;
    loop {
        while counter > 0 {
            if let Some(nxt) = print_stream.next() {
                println!("{}", nxt);
                if interactive {
                    counter -= 1;
                }
            } else {
                break;
            }
        }
        if counter > 0 {
            break;
        }
        println!("> n(next) q(quit) 1~9(show next 1~9)");

        reader.read_line(&mut input_buf).await?;
        let input = input_buf.trim();
        match input {
            "n" | "1" => {
                counter = 1;
            }
            "q" => {
                break;
            }
            "2" => counter = 2,
            "3" => counter = 3,
            "4" => counter = 4,
            "5" => counter = 5,
            "6" => counter = 6,
            "7" => counter = 7,
            "8" => counter = 8,
            "9" => counter = 9,
            _ => println!("invalid key input, retry"),
        }
        input_buf.clear();
    }
    Ok(())
}
