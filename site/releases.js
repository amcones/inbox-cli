// Generated from CHANGELOG.md. Do not edit by hand.
window.INBOX_RELEASES = [
  {
    "version": "0.6.2",
    "date": "2026-10-01",
    "changes": {
      "en": [
        "Add checksum-verified bootstrap installers that detect the platform and download the latest prebuilt release from GitHub on macOS, Linux, and Windows.",
        "Make elapsed-time output and review explanations independently configurable through INBOX_SHOW_ELAPSED and INBOX_REVIEW_REASONS.",
        "Replace the compact inbox info mark with the new high-resolution ASCII artwork."
      ],
      "zh": [
        "为 macOS、Linux 和 Windows 新增带校验的引导安装脚本，自动识别平台并从 GitHub 下载最新预编译版本。",
        "通过 INBOX_SHOW_ELAPSED 和 INBOX_REVIEW_REASONS 分别配置是否显示命令耗时和 review 原因。",
        "使用新的高分辨率 ASCII 字符画替换 inbox info 中的紧凑图标。"
      ]
    }
  },
  {
    "version": "0.6.1",
    "date": "2026-10-01",
    "changes": {
      "en": [
        "Increase the inbox info ASCII icon resolution and show inbox tags entries as the same colored #tag form used in note lists.",
        "Report elapsed time after every command on standard error, preserving standard output for scripts and pipelines.",
        "Replace all committed icon sizes with the latest direct Figma exports and refresh the website icon copies.",
        "Generate bilingual website release history from CHANGELOG.md at build time, removing the unreliable runtime fetch and its duplicate maintenance burden.",
        "Add an accessible, responsive back-to-top control and language-specific documentation links to the introducing website.",
        "Automatically validate and create the version tag after a prepared release is merged into main, then invoke the reusable release workflow so publication cannot be skipped by a missing manual tag."
      ],
      "zh": [
        "提高 inbox info 的 ASCII 图标分辨率，并让 inbox tags 使用与灵感列表一致的彩色 #tag 形式。",
        "每条命令执行后在标准错误中显示耗时，同时保持标准输出可用于脚本和管道。",
        "使用 Figma 最新直接导出替换仓库中的全部图标尺寸，并同步更新网站图标副本。",
        "构建时从 CHANGELOG.md 生成双语网站版本记录，移除不稳定的运行时请求，也无需维护第二份版本数据。",
        "为产品主页新增支持无障碍和响应式布局的回到顶部按钮，并按页面语言链接对应的中英文文档。",
        "发布准备合并到 main 后自动校验并创建版本标签，再调用可复用发布工作流，避免因遗漏手动推送标签而跳过发布。"
      ]
    }
  },
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
  }
];
