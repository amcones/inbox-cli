# inbox

[English](README.md) · [简体中文](README.zh-CN.md)

**极简 CLI 灵感笔记工具。**

inbox 是 Rust 编写的轻量命令行应用，将灵感按天保存在本地 Markdown 文件中。无需账号，也无需后台服务。

- **快速记录**：正文加标签，或通过管道接收多行内容。
- **方便找回**：全文搜索、标签筛选，按时间或结合近期记录与浏览次数的隐藏优先级排序。
- **可编辑、可恢复**：修改正文与标签，删除后进入回收站，恢复时保留原始身份。
- **数据自己掌握**：每天一个可读的 Markdown 文件，macOS、Linux、Windows 均提供单文件程序。
- **中英文支持**：按系统语言自动选择，也可手动指定。

## 安装

从 [GitHub Releases](https://github.com/amcones/inbox-cli/releases/latest) 下载对应压缩包；每个压缩包都附有 `.sha256` 校验文件。

| 平台 | 压缩包 |
|---|---|
| macOS Apple Silicon | `inbox-macos-aarch64.tar.gz` |
| macOS Intel | `inbox-macos-x86_64.tar.gz` |
| Linux ARM64 | `inbox-linux-aarch64.tar.gz` |
| Linux x86_64 | `inbox-linux-x86_64.tar.gz` |
| Windows x86_64 | `inbox-windows-x86_64.zip` |

macOS 和 Linux 解压后，将程序放入 `PATH`。以 Apple Silicon 为例：

```bash
tar -xzf inbox-macos-aarch64.tar.gz
mkdir -p ~/.local/bin
install -m 755 inbox-macos-aarch64/inbox ~/.local/bin/inbox
inbox --version
```

确保 `~/.local/bin` 已在 `PATH` 中。Windows 解压 ZIP 后，将 `inbox.exe` 所在目录加入 `PATH`，在 PowerShell 中运行 `inbox --version`。

使用 Rust 1.89 或更新版本时，也可以克隆仓库，通过脚本编译或安装：

```bash
git clone https://github.com/amcones/inbox-cli
cd inbox-cli
./scripts/build.sh
./scripts/install.sh
```

Unix 安装脚本默认将程序放到 `~/.local/bin`，并为检测到的 Bash、Zsh 或 Fish 安装补全。可通过 `--bin-dir` 和 `--shell` 修改，运行 `./scripts/install.sh --help` 查看参数。Windows 使用：

```powershell
git clone https://github.com/amcones/inbox-cli
Set-Location inbox-cli
.\scripts\build.ps1
.\scripts\install.ps1
```

PowerShell 安装脚本默认安装到 `%LOCALAPPDATA%\Programs\inbox`，并在同一目录生成 `inbox-completion.ps1`。如果安装目录尚未加入 `PATH`，脚本会提示。也可使用 `cargo install --git ssh://git@github.com/amcones/inbox-cli.git --locked` 直接安装已发布的源码版本。

## 记录、查找和完善

```bash
inbox add '为博客增加阅读模式' -t 产品 -t 博客
printf '第一行\n第二行\n' | inbox add - -t 灵感
inbox                              # 最近 20 条
inbox search '阅读模式'              # 搜索完整正文，不区分大小写
inbox list -t 产品 -n 5              # 此标签下最近 5 条
inbox list --sort priority          # 近期或经常查看的灵感
inbox review                        # 回顾 5 条优先候选并显示原因
inbox review -n 3                   # 将候选限制为 3 条
```

新增后会输出短 ID。将下面的 ID 替换为实际值：

```bash
inbox show a83f912b
inbox edit a83f912b '让读者调整字号'
inbox edit a83f912b -t 产品 -t 待办
inbox edit a83f912b --clear-tags
```

编辑保留 ID、创建时间和浏览次数。不指定标签选项就保留标签；`-t` 替换全部标签。省略正文可只改标签；`edit <ID> -` 从标准输入读取正文。

| 需求 | 命令或行为 |
|---|---|
| 同时匹配多个标签 | `inbox list -t 产品 -t 博客` |
| 匹配任意一个标签 | 加上 `--any` |
| 搜索与筛选组合 | `inbox search '阅读' -t 产品 --sort priority -n 5` |
| 查看标签及数量 | `inbox tags` |
| 查看但不计浏览次数 | `inbox show <ID> --no-track` |
| 完整用法 | `inbox --help` |

标签中的英文字母统一转换为小写，因此 `Rust`、`RUST` 和 `rust` 是同一个标签；同时自动去重并去掉开头的 `#`。正文中的 `#文字` 不会自动变成标签。只有成功输出的 `show` 才计一次浏览，重定向到文件也计数；列表、搜索和回顾不计数。ID 支持至少四位的唯一前缀。单条正文最多 1 MiB，最多 64 个标签，每个标签最多 128 字节。

## 命令补全

安装脚本会自动生成补全。手动安装时，可为当前 shell 生成一次补全脚本，然后重启 shell。Zsh 示例：

```bash
mkdir -p ~/.zfunc
inbox completions zsh > ~/.zfunc/_inbox
echo 'fpath=(~/.zfunc $fpath)' >> ~/.zshrc
echo 'autoload -Uz compinit && compinit' >> ~/.zshrc
```

Bash 可在 shell 配置中加载生成的脚本；Fish 与 PowerShell 可直接加载对应脚本：

```bash
inbox completions bash > ~/.inbox-completion.bash
echo 'source ~/.inbox-completion.bash' >> ~/.bashrc
mkdir -p ~/.config/fish/completions
inbox completions fish > ~/.config/fish/completions/inbox.fish
inbox completions powershell > inbox-completion.ps1
```

PowerShell 用户需在配置文件中点加载 `inbox-completion.ps1`。支持的名称是 `bash`、`zsh`、`fish` 和 `powershell`。

## 删除与恢复

```bash
inbox delete a83f912b               # 单条移入回收站
inbox delete today                  # 确认后移动今天的灵感
inbox delete range -h 8 12          # 今天 08:00（含）至 12:00（不含）
inbox delete range -y 26 -m 9 -d 29 -h 0 24
inbox delete all                    # 确认后移动全部灵感
inbox trash
inbox restore a83f912b
inbox trash empty                   # 确认后永久清空
```

`delete range` 省略年、月、日时使用当前本地日期，省略小时范围时使用 `0 24`。两位年份代表 2000–2099 年；匹配使用灵感记录时的日期和小时。在该命令中，`-y` 表示年份，`-h` 表示小时，请用 `--yes` 跳过确认、用 `--help` 查看帮助。

批量删除输入 `yes`、`y`、`是` 或 `确认` 才执行，其他输入或标准输入结束均取消。脚本可用 `--yes` 跳过提示。确认期间新增的灵感会保留。删除会清除浏览记录；恢复保留正文、标签、ID 和原创建时间，不恢复历史浏览次数。**清空回收站后无法恢复。**

## 数据、语言与备份

数据默认保存到 `~/inbox/YYYY/MM/YYYY-MM-DD.md`。通过 `--dir` 或 `INBOX_DIR` 更改位置，命令行选项优先。

```bash
inbox --dir ~/Notes/inbox add '一个想法'
inbox --lang zh --help
inbox --lang en --help
```

语言优先级为 `--lang` > `INBOX_LANG` > 系统语言，支持 `auto`、`zh`、`en`；其他系统语言回退到英文。正文与标签不会翻译。从 v0.3.0 开始使用 `add` 代替旧的 `-m` 记录选项。如果正文以连字符开头，请先写选项，再用 `--` 分隔，例如 `inbox add -- '--一个想法'`。

**备份整个数据目录，包括 `.inbox`**，其中包含浏览历史和回收站。复制前先停止 inbox 命令。隐藏文件不等于加密；不要再用不支持回收站的旧版本操作同一个库。

发生数据损坏时先停止写入、备份，然后运行 `inbox doctor` 检查正文、浏览日志和回收站。它不会猜测如何修复损坏内容。手动编辑 Markdown 时应保留元数据与记录标记，避免并发写入。详见[存储与恢复说明](docs/FORMAT.md)。

## 项目文档

[开发与贡献](CONTRIBUTE.md) · [性能测量](docs/BENCHMARK.md) · [架构评估](docs/ARCHITECTURE.md) · [更新记录](CHANGELOG.md) · [MIT 许可证](LICENSE)
