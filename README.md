# inbox

用一条命令记录灵感。Rust 编写，单二进制，本地 Markdown 存储，无后台进程。

## 安装

推荐从 [GitHub Releases](https://github.com/amcones/inbox-cli/releases/latest) 下载对应平台的压缩包。每个压缩包旁都提供同名 `.sha256` 校验文件。

| 系统 | 架构 | 下载文件 | 持续验证 |
|---|---|---|---|
| macOS | Apple Silicon / ARM64 | `inbox-macos-aarch64.tar.gz` | 构建、测试、运行检查 |
| macOS | Intel x86_64 | `inbox-macos-x86_64.tar.gz` | 构建、测试、运行检查 |
| Linux | ARM64 | `inbox-linux-aarch64.tar.gz` | 构建、测试、运行检查 |
| Linux | x86_64 | `inbox-linux-x86_64.tar.gz` | 构建、测试、运行检查 |
| Windows | x86_64 | `inbox-windows-x86_64.zip` | 构建、测试、运行检查 |

macOS 和 Linux 解压后，将 `inbox` 放入 `PATH`：

```bash
tar -xzf inbox-<平台>.tar.gz
mkdir -p ~/.local/bin
install -m 755 inbox-<平台>/inbox ~/.local/bin/inbox
inbox --version
```

Windows 解压 `inbox-windows-x86_64.zip`，将其中 `inbox.exe` 所在目录加入 `PATH`，然后在 PowerShell 运行：

```powershell
inbox --version
```

也可以使用 Rust 1.89 或更新版本从源码安装：

```bash
cargo install --git https://github.com/amcones/inbox-cli --tag v0.2.2 --locked
```

确保 `~/.cargo/bin` 在 `PATH` 中。开发者也可以在源码目录直接构建：

```bash
cargo build --release --locked
./target/release/inbox --help
```

## 使用

```bash
inbox add '为博客增加阅读模式' -t 产品 -t 博客
inbox add '正文中的 #文字 不会自动变成标签'
printf '第一行\n第二行\n' | inbox add - -t 灵感

inbox                              # 最近 20 条
inbox list -n 5                    # 最近 5 条
inbox list -t 产品                  # 标签精确匹配
inbox list -t 产品 -t 博客           # 两个标签都匹配
inbox list -t 产品 -t 博客 --any     # 匹配任意标签
inbox list --sort priority          # 隐藏优先级排序
inbox search '阅读模式'              # 搜索完整正文，不区分大小写
inbox search '阅读模式' -t 产品       # 搜索并按标签筛选
inbox search '阅读模式' --sort priority -n 5
inbox show a83f912b                 # ID 请替换为实际记录的 ID
inbox show a83f912b --no-track      # 查看但不计浏览次数
inbox edit a83f912b '修改后的正文'    # 保留原 ID、创建时间和浏览记录
inbox edit a83f912b -t 产品 -t 待办   # 替换全部标签
inbox edit a83f912b --clear-tags     # 清空标签
printf '多行\n新正文\n' | inbox edit a83f912b -
inbox delete a83f912b               # 移入回收站并清除浏览记录
inbox delete today                  # 确认后将今天的灵感移入回收站
inbox delete all                    # 确认后将全部灵感移入回收站
inbox delete all --yes              # 已由脚本明确确认，跳过提示
inbox trash                         # 查看回收站
inbox restore a83f912b              # 用原 ID 和创建时间恢复
inbox trash empty                   # 确认后永久清空回收站
inbox tags                         # 标签和对应的记录数量
inbox doctor                       # 检查格式、重复 ID、浏览日志
```

新增成功仅输出 8 位短 ID，方便脚本使用。完整 ID 为 UUID；`show`、`edit`、`delete` 和 `restore` 接受至少 4 位的唯一前缀，遇到冲突会要求更长的 ID。

标签区分大小写，自动去重，去掉开头的 `#`，不允许空白和控制字符。每条最多 64 个标签，每个最多 128 字节。单条内容最多 1 MiB，支持中文、Emoji、多行和 Markdown。CRLF 转为 LF，其余正文保存原样；终端输出会过滤控制字符。列表摘要最多 80 个 Unicode 字符。

默认数据目录为 `~/inbox`。优先级为 `--dir` > `INBOX_DIR` > 默认目录：

```bash
export INBOX_DIR="$HOME/Notes/inbox"
inbox --dir ~/Notes/scratch add '临时想法'
```

退出码：`0` 成功，`1` 内容/存储/运行错误，`2` 命令行参数错误。管道提前关闭按正常退出处理，不增加未成功输出的浏览次数。新增内容可能已保存但成功提示未送达；输出或同步错误后应先检查文件再重试，避免重复记录。

## 语言

默认根据系统语言选择中文或英文；其他语言回退到英文。可用全局选项覆盖（放在子命令前后均可）：

```bash
inbox --lang zh --help
inbox --help --lang en
inbox --lang auto doctor
export INBOX_LANG=en               # 后续命令默认英文
```

优先级为 `--lang` > `INBOX_LANG` > 系统语言。系统语言按 `LC_ALL`、`LC_MESSAGES`、`LANG` 中第一个非空值识别；`zh_CN`、`zh-TW`、`zh-Hant` 等均使用中文，`C`/`POSIX` 使用英文。macOS 未设置这些变量时读取系统的首选语言。`auto` 始终回到系统语言。选项支持 `auto`、`zh`、`en`。

帮助、应用提示和应用错误支持中英文；底层操作系统/库的诊断保留原文。灵感正文、标签、存储格式和命令名称不随界面语言变化。

## 编辑与删除

`inbox edit` 可修改正文，也可用 `-t` 替换全部标签或用 `--clear-tags` 清空标签。未指定标签选项时保留原标签；只指定标签选项时保留原正文。编辑保留 ID、创建时间和浏览次数。

`inbox delete <ID前缀>` 会将单条记录移入回收站。`delete today` 和 `delete all` 会显示待移动数量，只有输入 `yes`、`y`、`是` 或 `确认` 才执行；脚本可用 `--yes`/`-y` 跳过批量确认。确认针对提示时生成的记录快照，因此确认期间新增的灵感不会被移动。删除会清除浏览记录；恢复保留原正文、标签、ID 和创建时间，不恢复浏览次数。

`inbox trash` 列出已删除灵感；`inbox restore` 恢复一条；`inbox trash empty` 经确认后永久删除全部回收站内容。永久清空可用 `--yes`/`-y`。未知、歧义或损坏的数据会报错，不猜测目标。

删除使用独占锁，在同一可恢复事务中准备 Markdown、浏览日志和回收站副本。若提交后进程中断，下次命令会先完成整批操作；提交前中断则保留原始数据。恢复目录位于 `.inbox/delete-pending/`，不要手动移除已提交的清单。

本版本直接读取 v0.1/v0.2 数据，并兼容恢复 v0.2 已提交的单条删除，无需迁移；创建回收站后，请勿再用不支持回收站的旧版程序操作同一个库。

## 数据和备份

```text
~/inbox/
├── 2026/09/2026-09-29.md
└── .inbox/
    ├── format-version
    ├── write.lock
    ├── views.log
    └── trash/<完整 UUID>.json
```

当天第一次记录创建 Markdown，之后只追加。日期按记录时的本地时区确定，元数据保存小数秒及 UTC 偏移，切换时区不迁移已有文件。时区来自操作系统或 `TZ`；检测失败明确报错。

**备份整个数据目录，包括 `.inbox`。** Markdown 包含正文、标签、ID 和创建时间；浏览次数只存在 `views.log` 中，无法从 Markdown 恢复。不要删除或替换正在使用的 `write.lock`。复制备份时先停止所有 inbox 命令，避免获得部分写入的快照。

可手动编辑 Markdown 正文，但应保留元数据、时间前缀、缩进与结束标记；编辑时不要同时运行 inbox 写入命令。格式详见 [FORMAT.md](docs/FORMAT.md)。新建文件权限为 `0600`、目录为 `0700`（Unix），不会修改已有目录权限。隐藏目录不等于加密。

## 优先级

```text
score = 0.65 × 2^(-age_days / 14) + 0.35 × views / (views + 5)
```

同样浏览次数下，越新越靠前；同样时间下，浏览越多越靠前，浏览收益逐渐递减。未来时间的年龄按 0 计算。相同分数按创建时间、完整 ID 降序稳定排序。

只有 `show` 成功输出后才追加一次浏览，重定向到文件也计数；`list`、`search`、`tags`、`doctor` 和 `show --no-track` 均不计数。排序时实时计算，分数不显示、不写回 Markdown。当前版本参数固定。

## 性能与可靠性

- 记录时只读当天文件末尾最多 256 字节，追加并调用 `sync_all`，不会扫描历史内容。
- 最近列表保留 Top-N 并尽早停止；会考虑跨时区导致的日期重叠。发现日期文件仍需枚举目录。
- 标签筛选按时间顺序扫描；全文搜索、优先级和单条 ID 查找需要全库扫描。
- 优先级使用大小为 N 的堆，排序开销为 `O(M log N)`；浏览计数内存与被浏览记录数相关。
- 写操作使用 OS 文件锁；锁随进程退出释放，纯读操作使用共享锁。默认 `show` 会写浏览记录，因此持有独占锁直到输出和计数完成，避免并发删除产生悬空浏览记录。编辑通过同目录暂存文件原子替换日期文件。外部编辑器或同步软件不一定遵守这些锁。
- 追加不是完整条目的原子事务。未完成的尾部不会被自动删除；后续写入拒绝覆盖，错误提供文件位置。
- 默认调用文件同步，新目录和文件同步父目录（Unix）。不承诺超出操作系统/磁盘硬件保障的断电原子性。

实测数据和复现方法见 [BENCHMARK.md](docs/BENCHMARK.md)。首次启动、文件缓存未命中、云盘和慢盘延迟可能更高。当前不包含同步、数据库索引、TUI 或 AI。

## 故障检查

```bash
inbox doctor
```

该命令读取全部规范日期文件，检查条目边界、元数据、重复 ID、浏览日志和回收站条目。读取现有目录时，如果缺少 `.inbox`，会初始化版本文件与锁；若有已提交的删除会先恢复，否则不会修复或覆盖正文。尚不存在的 inbox 作为空库读取，不创建文件。

发现半条记录时：停止写入，完整备份数据目录，然后在错误指示的 Markdown 中检查最后一个完整结束标记。把尾部残片另存后，手动修复为完整条目，或从原文件移出残片后用 `inbox add` 重新记录；最后再次运行 `doctor`。应用不会自动猜测或丢弃内容。

浏览日志损坏时，仍可新增、按时间列出及 `show --no-track`；优先级排序会报错。备份日志后修复错误行即可。整体移走 `views.log` 会重置所有浏览次数，正文不受影响。

## 开发与验证

```bash
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --release --locked
python3 scripts/benchmark.py
```

`release` 使用体积优化、LTO、单 codegen unit 和符号裁剪；`cargo build --profile speed` / `--profile small` 用于比较速度优先和更激进体积优化。脚本只使用临时测试库，不访问真实记录。
