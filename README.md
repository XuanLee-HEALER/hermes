# blowup

> [Click here for Chinese version/中文版本点此链接](./README_zh.md)

![Maintenance Status](https://img.shields.io/badge/Status-Active-yellow?style=for-the-badge&logo=movistar&logoSize=wider) ![Version](https://img.shields.io/badge/Version-0.1.1-red?style=for-the-badge&logoSize=wider) ![License](https://img.shields.io/badge/License-MIT-darkgreen?style=for-the-badge&logoSize=wider)

> **blow-up [Michelangelo Antonioni]**: A fashion photographer unknowingly captures a death on film after following two lovers in a park.
>
> The best movie I've seen so far.
>
> 我认为的最好的电影，目前为止

---

**blowup** is a command-line interface (CLI) tool designed to streamline and automate the technical aspects of my film-watching experience, from managing movie trackers to handling subtitle files.

It's a personal project born out of a passion for cinema and a desire to solve common technical frustrations, so I can spend more time enjoying the art itself.

## ✨ Features

### Current Features:

* Tracker Management: Download the latest tracker list from a specific GitHub repository.
* Subtitle Stream Management:
  * List available subtitle streams within a video container (requires ffprobe).
  * Export a specific subtitle stream from a video container to an SRT file (requires ffmpeg).
* SRT Subtitle Manipulation:
  * Shift all timestamps in an SRT subtitle file by a specified time offset.
  * Interactively compare and synchronize two SRT subtitle files.

### Planned Features:

* Flexible Tracker Sources: Support fetching tracker lists from multiple sources, including various GitHub repositories and direct URLs.
* Local Subtitle Translation: Translate subtitle files using a local large language model (LLM).

## 🚀 Installation & Usage

You can install `blowup` directly from `crates.io`.

> Note: The subtitle extraction and listing features require `ffmpeg` and `ffprobe` to be installed and accessible in your system's PATH. You can download them from the official website or a package manager.

```bash
cargo install blowup
# After installation, add the cargo binary path to your system's PATH if it's not already.

# Command Overview

# The tool is structured with subcommands for each major function.

# Main commands
Usage: blowup <COMMAND>

Commands:
  tracker  handle all things about tracker list
  sub      subtitle file processing tools

# For more detailed usage and examples, run:
blowup --help
blowup <subcommand> --help
# For detailed usage of each command, please refer to the built-in --help documentation and the project's official documentation.
```

## 💡 Motivation

The core inspiration behind blowup came from my personal film-watching workflow. I often found myself spending an excessive amount of time on repetitive tasks, such as manually updating torrent trackers or correcting subtitle synchronization.

This project is an attempt to automate these chores, making the technical part of movie-watching seamless, so the focus can remain on the story, cinematography, and performances.

## 🤝 Contributing

Contributions are welcome! If you've encountered a similar problem or have an idea for a new feature, feel free to open an issue or submit a pull request.


## 📜 License

This project is licensed under the MIT License. For the full license text, see the [LICENSE](./LICENSE.txt) file.
