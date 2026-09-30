const copy = {
  en: {
    skip: "Skip to content", navFeatures: "Features", navWorkflow: "Workflow", navInstall: "Install", navGithub: "GitHub",
    eyebrow: "FAST CAPTURE · PLAIN FILES", heroTitle: "Catch the thought<br>before it slips.", heroLede: "A tiny command-line inbox for ideas. Write in a heartbeat, keep everything in local Markdown, and find the right note when it matters.",
    download: "Download latest", readGuide: "Read the guide", proofBinary: "Single binary", proofLocal: "Local Markdown", captured: "Captured",
    promise: "No account. No cloud dependency. No process waiting in the background.",
    featuresEyebrow: "BUILT FOR THE MOMENT", featuresTitle: "Less ceremony.<br>More ideas kept.", featuresLede: "inbox stays out of the way while giving every note enough structure to become useful later.",
    featureCaptureTitle: "Capture in one line", featureCaptureText: "Add a thought with tags, or pipe multiline text from another tool. The command returns immediately with a short ID.",
    featureFindTitle: "Recall without organizing first", featureFindText: "Search full note bodies, filter by tags, and use dynamic shell completion for commands, options, tags, and note IDs.",
    featureReviewTitle: "Let good ideas resurface", featureReviewText: "Review selects a small set using recency and view history, then tells you the simple reason each note was chosen.",
    featureSafeTitle: "Change things safely", featureSafeText: "Edit notes and tags, recover deletions from trash, and create validated backups before moving your library.",
    workflowEyebrow: "ONE SIMPLE LOOP", workflowTitle: "Capture now.<br>Shape later.", workflowLede: "Each day is a readable Markdown file. inbox adds the speed, IDs, search, and recovery around it while the words remain yours.", formatLink: "See the storage format",
    step1Title: "Add", step1Text: "Put the thought down before context switching.", step2Title: "Find", step2Text: "Search words, tags, dates, or recent IDs.", step3Title: "Review", step3Text: "Bring recent and often-viewed ideas back.", step4Title: "Keep", step4Text: "Edit, restore, back up, and verify confidently.",
    localEyebrow: "LOCAL BY DEFAULT", localTitle: "Your notes stay ordinary.", localText: "Open them with any editor. Sync them with any tool. Back them up like any other folder. inbox stores notes by day and keeps the format human-readable.", localPoint1: "Readable daily Markdown", localPoint2: "No account or proprietary database", localPoint3: "Built-in integrity checks and backup verification",
    installEyebrow: "START IN MINUTES", installTitle: "One small binary.<br>Three desktop platforms.", installHint: "Download the matching archive from GitHub Releases, extract it, and place inbox on your PATH.", copy: "Copy", copied: "Copied", copyLabel: "Copy command", allDownloads: "All downloads", installGuide: "Installation guide",
    closingEyebrow: "A PLACE FOR EVERY LINE.", closingTitle: "Make room for the next thought.", viewGithub: "View on GitHub", footer: "Open source under the MIT License."
  },
  zh: {
    skip: "跳到主要内容", navFeatures: "功能", navWorkflow: "工作方式", navInstall: "安装", navGithub: "GitHub",
    eyebrow: "快速记录 · 纯文本存储", heroTitle: "留住闪念，<br>趁它还没溜走。", heroLede: "一个轻巧的命令行灵感收件箱。即刻写下想法，用本地 Markdown 长久保存，在需要时快速找回。",
    download: "下载最新版", readGuide: "查看使用指南", proofBinary: "单一二进制", proofLocal: "本地 Markdown", captured: "已记录",
    promise: "无需账号，不依赖云端，也没有常驻后台进程。",
    featuresEyebrow: "为灵感出现的瞬间而生", featuresTitle: "少一点流程，<br>多留下一些想法。", featuresLede: "inbox 安静地待在命令行里，同时为每条灵感提供日后真正用得上的结构。",
    featureCaptureTitle: "一行命令，立即记录", featureCaptureText: "为想法添加标签，也可以从其他工具通过管道传入多行内容。命令会立即返回一个短 ID。",
    featureFindTitle: "先记录，之后再整理", featureFindText: "全文搜索、标签筛选，加上针对命令、选项、标签和灵感 ID 的动态补全。",
    featureReviewTitle: "让好想法重新浮现", featureReviewText: "review 根据时间和查看次数选出少量候选，并告诉你每条灵感入选的简单原因。",
    featureSafeTitle: "安心修改，随时恢复", featureSafeText: "编辑内容和标签，从回收站找回误删灵感，并在迁移前创建经过验证的完整备份。",
    workflowEyebrow: "一个简单循环", workflowTitle: "现在捕捉，<br>以后打磨。", workflowLede: "每天的内容都是一个可读的 Markdown 文件。inbox 在此基础上提供速度、ID、搜索与恢复能力，文字始终属于你。", formatLink: "查看存储格式",
    step1Title: "记录", step1Text: "在切换上下文前先把想法留下。", step2Title: "查找", step2Text: "按文字、标签、日期或最近 ID 查找。", step3Title: "回顾", step3Text: "让近期和常看的想法重新出现。", step4Title: "保管", step4Text: "编辑、恢复、备份和验证都有保障。",
    localEyebrow: "默认保存在本地", localTitle: "你的笔记依旧普通。", localText: "可以用任何编辑器打开，用任何工具同步，像普通文件夹一样备份。inbox 按日期存放灵感，并保持格式清晰可读。", localPoint1: "按天组织的可读 Markdown", localPoint2: "没有账号和专有数据库", localPoint3: "内置完整性检查和只读备份验证",
    installEyebrow: "几分钟即可开始", installTitle: "一个小巧二进制，<br>覆盖三个桌面平台。", installHint: "从 GitHub Releases 下载对应压缩包，解压后将 inbox 放入 PATH。", copy: "复制", copied: "已复制", copyLabel: "复制命令", allDownloads: "全部下载", installGuide: "安装指南",
    closingEyebrow: "A PLACE FOR EVERY LINE.", closingTitle: "为下一个想法，留一个位置。", viewGithub: "在 GitHub 查看", footer: "以 MIT 许可证开源。"
  }
};

const commands = {
  macos: "tar -xzf inbox-macos-aarch64.tar.gz",
  linux: "tar -xzf inbox-linux-x86_64.tar.gz",
  windows: "Expand-Archive inbox-windows-x86_64.zip"
};

let language = navigator.language.toLowerCase().startsWith("zh") ? "zh" : "en";

function applyLanguage(nextLanguage) {
  language = nextLanguage;
  document.documentElement.lang = language === "zh" ? "zh-CN" : "en";
  document.querySelectorAll("[data-i18n]").forEach((element) => {
    const value = copy[language][element.dataset.i18n];
    if (value) element.innerHTML = value;
  });
  document.querySelectorAll("[data-i18n-aria]").forEach((element) => {
    element.setAttribute("aria-label", copy[language][element.dataset.i18nAria]);
  });
  const toggle = document.querySelector(".language-toggle");
  toggle.setAttribute("aria-pressed", String(language === "zh"));
  toggle.setAttribute("aria-label", language === "zh" ? "Switch to English" : "切换到中文");
  toggle.querySelector(".language-current").textContent = language === "zh" ? "中" : "EN";
  toggle.querySelector(".language-other").textContent = language === "zh" ? "EN" : "中";
}

document.querySelector(".language-toggle").addEventListener("click", () => {
  applyLanguage(language === "en" ? "zh" : "en");
});

document.querySelectorAll(".platform-tab").forEach((tab) => {
  tab.addEventListener("click", () => {
    document.querySelectorAll(".platform-tab").forEach((item) => {
      const active = item === tab;
      item.classList.toggle("active", active);
      item.setAttribute("aria-selected", String(active));
    });
    document.querySelector("#install-command").textContent = commands[tab.dataset.platform];
  });
});

document.querySelector(".copy-button").addEventListener("click", async (event) => {
  const button = event.currentTarget;
  const label = button.querySelector("span");
  try {
    await navigator.clipboard.writeText(document.querySelector("#install-command").textContent);
    label.textContent = copy[language].copied;
    window.setTimeout(() => { label.textContent = copy[language].copy; }, 1400);
  } catch {
    document.querySelector("#install-command").focus();
  }
});

applyLanguage(language);
