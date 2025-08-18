# blowup

> [Click here for Chinese version/中文版本点此链接](./README_zh.md)

![Maintenance Status](https://img.shields.io/badge/Status-Active-yellow?style=for-the-badge&logo=movistar&logoSize=wider) ![Version](https://img.shields.io/badge/Version-0.1.1-red?style=for-the-badge&logoSize=wider) ![License](https://img.shields.io/badge/License-MIT-darkgreen?style=for-the-badge&logoSize=wider)

> **blow-up [Michelangelo Antonioni]**: A fashion photographer unknowingly captures a death on film after following two lovers in a park.
>
> The best movie I've seen so far.
>
> 我认为的最好的电影，目前为止

---

`blowup` 是一个命令行工具（CLI），旨在简化和自动化我的个人观影体验中的技术环节，从管理电影种子追踪器（tracker）到处理字幕文件，让观影过程更加顺畅。

这是一个源于对电影的热爱而诞生的个人项目，其初衷是解决常见的技术痛点，从而把更多时间留给电影艺术本身。

## ✨ 主要功能

### 已实现功能

* Tracker 管理：从指定的 GitHub 仓库下载最新的 tracker 列表。
* 字幕流管理：
  * 列出视频容器中可用的字幕流（需要 ffprobe）。
  * 将视频容器中的指定字幕流导出为 SRT 文件（需要 ffmpeg）。
* SRT 字幕处理：
  * 对 SRT 字幕文件中的所有时间戳进行指定的时间平移。
  * 交互式对比并同步两个 SRT 字幕文件。

### 计划中功能

* 多源 Tracker 获取：支持从多种来源获取 tracker 列表，包括不同的 GitHub 仓库和直接 URL 下载。
* 本地字幕翻译：调用本地大语言模型（LLM）对字幕文件进行翻译。

## 🚀 安装与使用

您可以通过 `cargo` 直接从 crates.io 安装 `blowup`。

注意：字幕提取和列表功能需要您的系统已安装 ffmpeg 和 ffprobe，并且程序路径已添加到 PATH 环境变量中。您可以通过官方网站或包管理器下载它们。

```bash
cargo install blowup
# 安装后，如果您的系统 PATH 环境变量中没有 cargo 的二进制路径，请手动添加。

# 命令概览

# 本工具采用子命令结构，每个主要功能对应一个子命令。

# 主要命令
Usage: blowup <COMMAND>

Commands:
  tracker   与 tracker 列表相关的所有操作
  sub       字幕文件处理工具

# 更多详细用法和示例，请运行：
blowup --help
blowup <subcommand> --help
# 关于每个命令的详细用法，请参考内置的 --help 文档和项目的官方文档。
```

## 💡 项目灵感
创建 blowup 的核心灵感源于我个人的观影流程。我发现自己经常将大量时间花在重复性的任务上，比如手动更新种子 tracker 或校对字幕时间。

本项目正是为了自动化这些琐事而生的，让观影的技术环节变得无缝，从而把注意力重新集中到故事、摄影和表演上。

## 🤝 贡献指南

欢迎贡献！如果您也曾遇到类似的问题，或者对新功能有好的想法，欢迎提交议题（issue）或发起拉取请求（pull request）。

## 📜 许可协议

本项目采用 MIT 许可协议。完整的许可协议文本请参阅 LICENSE 文件。
