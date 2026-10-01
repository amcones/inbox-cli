# inbox

[产品主页](https://amcones.cn/inbox-cli/) · [English](README.md) · [简体中文](README.zh-CN.md)

**极简 CLI 灵感笔记工具。**

inbox 是 Rust 编写的轻量命令行应用，将灵感按天保存在本地 Markdown 文件中。无需账号，也无需后台服务。

- **快速记录**：正文加标签，或通过管道接收多行内容。
- **方便找回**：全文搜索、标签筛选，按时间或结合近期记录与浏览次数的隐藏优先级排序。
- **可编辑、可恢复**：修改正文与标签，误删可从回收站恢复，并能创建经过校验的整库备份。
- **数据自己掌握**：每天一个可读的 Markdown 文件，macOS、Linux、Windows 均提供单文件程序。
- **中英文支持**：按系统语言自动选择，也可手动指定。

## 安装

### 下载预编译版本

从 [GitHub Releases](https://github.com/amcones/inbox-cli/releases/latest) 下载对应压缩包；每个压缩包都附有 `.sha256` 校验文件。

#### macOS

| 平台 | 压缩包 |
|---|---|
| macOS Apple Silicon | `inbox-macos-aarch64.tar.gz` |
| macOS Intel | `inbox-macos-x86_64.tar.gz` |
| Linux ARM64 | `inbox-linux-aarch64.tar.gz` |
| Linux x86_64 | `inbox-linux-x86_64.tar.gz` |
| Windows x86_64 | `inbox-windows-x86_64.zip` |

解压后，将程序放入 `PATH`。以 Apple Silicon 为例：

```bash
tar -xzf inbox-macos-aarch64.tar.gz
mkdir -p ~/.local/bin
install -m 755 inbox-macos-aarch64/inbox ~/.local/bin/inbox
install -m 755 inbox-macos-aarch64/update.sh ~/.local/bin/inbox-update
inbox --version
```

确保 `~/.local/bin` 已在 `PATH` 中。

#### Linux

选择 ARM64 或 x86_64 压缩包，解压后将 `inbox` 放入 `PATH` 中的目录。

#### Windows

解压 ZIP，将 `inbox.exe` 所在目录加入 `PATH`，在 PowerShell 中运行 `inbox --version`。

### 从源码编译安装

#### macOS 和 Linux

使用 Rust 1.89 或更新版本：

```bash
git clone https://github.com/amcones/inbox-cli
cd inbox-cli
./scripts/build.sh
./scripts/install.sh
```

安装脚本会把 PATH 和补全配置写入检测到的 shell 配置文件，并将 `inbox-update` 安装到程序所在目录。可通过 `--bin-dir` 和 `--shell` 修改，运行 `./scripts/install.sh --help` 查看参数。

#### Windows

```powershell
git clone https://github.com/amcones/inbox-cli
Set-Location inbox-cli
.\scripts\build.ps1
.\scripts\install.ps1
```

PowerShell 安装脚本默认安装到 `%LOCALAPPDATA%\Programs\inbox`，会持久化用户 PATH、从 PowerShell 配置文件加载补全，并在同一目录安装 `inbox-update.ps1`。

### Cargo

#### macOS、Linux 和 Windows

使用 Rust 1.89 或更新版本直接安装已发布源码：

```bash
cargo install --git ssh://git@github.com/amcones/inbox-cli.git --tag v0.6.0 --locked
```

## 更新

通过 Release 或源码脚本安装时会同时安装更新器。更新器先验证压缩包的 SHA-256，再替换本地程序；发布包包含补全文件时，也会刷新已有的补全文件。

### macOS 和 Linux

```bash
inbox-update                    # 更新到最新版
inbox-update --version v0.6.0  # 更新到指定版本
```

如果 `inbox` 不在 `PATH` 中，请用 `--bin-dir` 指定其目录。

### Windows

```powershell
inbox-update.ps1
inbox-update.ps1 -Version v0.6.0
```

如果 `inbox.exe` 不在 `PATH` 中，请使用 `-BinDir`。通过 Cargo 安装的版本仍由 Cargo 管理，请使用对应的 `cargo install` 命令更新。

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

列表、搜索、回顾和回收站使用粗略相对时间，并显示结果总数；`inbox show <ID>` 仍提供完整时间。在终端中，时间、ID、标签和搜索匹配文本使用不同颜色；重定向时输出保持纯文本。总数写入标准错误，因此对灵感行使用管道的脚本不受影响。设置 [`NO_COLOR`](https://no-color.org/) 可明确关闭颜色。

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
| 查看构建信息、数据目录和库统计 | `inbox info` |
| 查看但不计浏览次数 | `inbox show <ID> --no-track` |
| 完整用法 | `inbox help` |

标签中的英文字母统一转换为小写，因此 `Rust`、`RUST` 和 `rust` 是同一个标签；同时自动去重并去掉开头的 `#`。正文中的 `#文字` 不会自动变成标签。只有成功输出的 `show` 才计一次浏览，重定向到文件也计数；列表、搜索和回顾不计数。ID 支持至少四位的唯一前缀。单条正文最多 1 MiB，最多 64 个标签，每个标签最多 128 字节。

## 命令补全

安装脚本会部署 Bash、Zsh、Fish 和 PowerShell 的动态补全，覆盖命令、各命令可用选项、标签、排序与语言值、常用数量、日期与小时。`show`、`edit` 和 `delete` 会提示活动灵感 ID，`restore` 会提示回收站 ID。Unix 安装脚本会把所需设置写入检测到的 shell 配置文件；PowerShell 安装脚本会更新用户配置文件。

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

`delete range` 省略年、月、日时使用当前本地日期，省略小时范围时使用 `0 24`。两位年份代表 2000–2099 年；匹配使用灵感记录时的日期和小时。在该命令中，`-y` 表示年份、`-h` 表示小时；请用 `--yes` 跳过确认、用 `inbox help` 查看帮助。

批量删除按下 `y` 键执行，其他任意键或标准输入结束都会立即取消，无需回车。脚本可用 `--yes` 跳过提示。确认期间新增的灵感会保留。删除会清除浏览记录；恢复保留正文、标签、ID 和原创建时间，不恢复历史浏览次数。**清空回收站后无法恢复。**

## 备份与还原

完整备份 Markdown、浏览记录和回收站：

```bash
inbox backup ~/Backups/inbox-2026-09-30
inbox backup verify ~/Backups/inbox-2026-09-30
inbox restore --from ~/Backups/inbox-2026-09-30
```

备份时会锁定数据目录，检查全部灵感、浏览记录和回收站，再生成带清单的独立目录。`backup verify` 会重复完整检查，但不会锁定或修改快照，也可用于只读介质。目标目录必须尚不存在，且不能放在数据目录内部。备份仍是可直接读取的本地文件，不会加密。

整库还原会先校验快照，再等待按下 `y`；其他任意键都会取消。自动化脚本可用 `--yes` 跳过确认。覆盖前会在当前数据目录旁自动创建名为 `inbox.before-restore-…` 的安全快照，并输出其路径。已经确认的还原若被异常中断，下次打开库时会自动继续。若要防范磁盘损坏，应把备份保存到其他存储设备。

## 数据与语言

数据默认保存到 `~/inbox/YYYY/MM/YYYY-MM-DD.md`。通过 `--dir` 或 `INBOX_DIR` 更改位置，命令行选项优先。

```bash
inbox --dir ~/Notes/inbox add '一个想法'
inbox help --lang zh
inbox help --lang en
```

语言优先级为 `--lang` > `INBOX_LANG` > 系统语言，支持 `auto`、`zh`、`en`；其他系统语言回退到英文。正文与标签不会翻译。从 v0.3.0 开始使用 `add` 代替旧的 `-m` 记录选项。如果正文以连字符开头，请先写选项，再用 `--` 分隔，例如 `inbox add -- '--一个想法'`。

运行 `inbox info` 可查看 ASCII 应用图标、当前程序版本、UTC 构建时间、开源协议、作者、解析后的数据目录、存储状态，以及日期文件、活动灵感、浏览和回收站数量。它会执行与 `doctor` 相同的数据校验，并输出简洁的运行摘要。

发生数据损坏时先停止写入、备份，然后运行 `inbox doctor` 检查正文、浏览日志和回收站。它不会猜测如何修复损坏内容。手动编辑 Markdown 时应保留元数据与记录标记，避免并发写入。详见[存储与恢复说明](docs/FORMAT.md)。

## 项目文档

[产品主页](https://amcones.cn/inbox-cli/) · [品牌美术资源](art/README.md) · [开发与贡献](CONTRIBUTE.md) · [性能测量](docs/BENCHMARK.md) · [架构评估](docs/ARCHITECTURE.md) · [更新记录](CHANGELOG.md) · [MIT 许可证](LICENSE)
