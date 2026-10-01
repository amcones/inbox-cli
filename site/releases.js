// Generated from CHANGELOG.md. Do not edit by hand.
window.INBOX_RELEASES = [
  {
    "version": "0.6.0",
    "date": "2026-10-01",
    "changes": {
      "en": [
        "Redesign list, search, review, and trash output with compact relative times, result counts, and terminal-aware colors while keeping redirected note output plain and script-friendly.",
        "Highlight case-insensitive search matches and center long excerpts around the first match.",
        "Expand inbox info with an ASCII icon, version, UTC build time, license, author, storage location, and library counts."
      ],
      "zh": [
        "重新设计列表、搜索、回顾和回收站输出，使用简洁的相对时间、结果总数和终端感知配色，同时保持重定向输出为纯文本并兼容脚本。",
        "高亮不区分大小写的搜索结果，并让长内容摘要围绕首次匹配位置显示。",
        "扩展 inbox info，展示 ASCII 图标、版本、UTC 构建时间、许可证、作者、存储位置和资料库统计。"
      ]
    }
  },
  {
    "version": "0.5.1",
    "date": "2026-09-30",
    "changes": {
      "en": [
        "Add inbox backup verify <directory> for read-only validation of backup manifests, notes, view history, and trash.",
        "Test that v0.5.0 backup format restores in v0.5.1 and reject unknown backup format versions without modification.",
        "Add inbox info with the installed version, resolved data directory, storage state, and library counts.",
        "Complete the new command and backup action across Bash, Zsh, Fish, and PowerShell."
      ],
      "zh": [
        "新增 inbox backup verify <目录>，以只读方式校验备份清单、灵感、浏览记录和回收站。",
        "验证 v0.5.0 备份可在 v0.5.1 中还原，并在不修改数据的前提下拒绝未知备份格式。",
        "新增 inbox info，展示安装版本、解析后的数据目录、存储状态和资料库统计。",
        "为 Bash、Zsh、Fish 和 PowerShell 补齐新命令与备份操作的补全支持。"
      ]
    }
  },
  {
    "version": "0.5.0",
    "date": "2026-09-30",
    "changes": {
      "en": [
        "Add inbox backup <directory> for locked, validated snapshots of Markdown notes, view history, and trash.",
        "Add confirmed full-library restore with inbox restore --from <directory>, automatic pre-restore safety backups, and --yes for automation.",
        "Make committed restore transactions recoverable after interruption while preserving the active lock file.",
        "Complete backup paths and restore options in Bash, Zsh, Fish, and PowerShell."
      ],
      "zh": [
        "新增 inbox backup <目录>，在锁定并校验后创建 Markdown 灵感、浏览记录和回收站快照。",
        "新增需确认的整库还原、还原前自动安全备份，以及适用于自动化的 --yes 选项。",
        "已提交的还原事务在中断后可以恢复，同时保留当前锁文件。",
        "为 Bash、Zsh、Fish 和 PowerShell 补齐备份路径和还原选项的补全支持。"
      ]
    }
  },
  {
    "version": "0.4.3",
    "date": "2026-09-30",
    "changes": {
      "en": [
        "Replace the -h / --help flags with the inbox help command; delete range -h remains the hour-range option.",
        "Add verified update scripts for macOS, Linux, and Windows that replace the installed binary and refresh existing shell completion files.",
        "Install the updater beside the binary and include updater and completion scripts in release archives."
      ],
      "zh": [
        "用 inbox help 命令替代 -h / --help；delete range -h 继续表示小时范围。",
        "为 macOS、Linux 和 Windows 新增经过校验的更新脚本，可替换已安装程序并刷新现有命令补全文件。",
        "将更新器安装在程序旁，并在发布归档中包含更新器和命令补全脚本。"
      ]
    }
  }
];
