#!/usr/bin/env python3
"""Reproducible process-level benchmarks; never touches the user's inbox."""

import argparse
import datetime as dt
import json
import math
import os
from pathlib import Path
import platform
import statistics
import subprocess
import tempfile
import time
import uuid


def dataset(root, size):
    state = root / ".inbox"
    state.mkdir(parents=True)
    (state / "format-version").write_text("1\n")
    (state / "write.lock").touch()
    now = dt.datetime.now().astimezone().replace(microsecond=0)
    body = "灵感记录 benchmark "
    body += "x" * (500 - len(body.encode()))
    last_id = None
    day_file = None
    previous_path = None
    with (state / "views.log").open("w") as log:
        try:
            for i in range(size):
                created = now - dt.timedelta(minutes=15 * (size - i))
                date = created.strftime("%Y-%m-%d")
                path = root / created.strftime("%Y/%m") / f"{date}.md"
                if path != previous_path:
                    if day_file:
                        day_file.close()
                    path.parent.mkdir(parents=True, exist_ok=True)
                    day_file = path.open("w")
                    day_file.write(f"# {date}\n\n")
                    previous_path = path
                last_id = str(uuid.uuid4())
                metadata = json.dumps({"id": last_id, "created": created.isoformat(), "tags": ["idea", f"topic{i % 16}"]}, ensure_ascii=False, separators=(",", ":"))
                day_file.write(f"<!-- inbox:note {metadata} -->\n- {created:%H:%M:%S} {body}\n<!-- inbox:end {last_id} -->\n\n")
                if i % 5 == 0:
                    log.write(f"{last_id}\t{now.isoformat()}\n" * 3)
        finally:
            if day_file:
                day_file.close()
    return last_id


def run(command):
    return subprocess.run(command, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, check=True)


def measure(command, samples):
    for _ in range(5):
        run(command)
    times = []
    for _ in range(samples):
        start = time.perf_counter_ns()
        run(command)
        times.append((time.perf_counter_ns() - start) / 1e6)
    times.sort()
    return statistics.median(times), times[math.ceil(len(times) * .95) - 1]


def rss(command):
    if hasattr(os, "wait4"):
        # Per-child rusage, rather than cumulative RUSAGE_CHILDREN. No ps or
        # system-wide sysctl access is needed. macOS uses bytes; Linux uses KiB.
        with subprocess.Popen(command, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE) as child:
            _, status, usage = os.wait4(child.pid, 0)
            child.returncode = os.waitstatus_to_exitcode(status)
            if child.returncode:
                raise subprocess.CalledProcessError(child.returncode, command, stderr=child.stderr.read())
            scale = 1024**2 if platform.system() == "Darwin" else 1024
            return usage.ru_maxrss / scale
    return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=Path("target/release/inbox"))
    parser.add_argument("--sizes", default="1000,10000,100000")
    parser.add_argument("--samples", type=int, default=50)
    parser.add_argument("--output", type=Path, default=Path("docs/BENCHMARK.md"))
    args = parser.parse_args()
    if args.samples < 1:
        parser.error("--samples must be positive")
    binary = args.binary.resolve()
    lines = ["# 性能测量", "", f"测量日期：{dt.datetime.now().astimezone().isoformat(timespec='seconds')}", "",
             f"- 平台：{platform.system()} {platform.release()} / {platform.machine()}",
             f"- Rust：{subprocess.check_output(['rustc', '--version'], text=True).strip()}",
             f"- 应用：{subprocess.check_output([str(binary), '--version'], text=True).strip()}",
             f"- 二进制：`{binary.name}`，{binary.stat().st_size:,} bytes（{binary.stat().st_size / 1024**2:.2f} MiB）。",
             f"- 每组预热 5 次，再采样 {args.samples} 次；每次启动新进程。",
             "- 包含 Python 发起子进程的开销、启动、操作、退出；输出重定向，不包含终端绘制。",
             "- 热文件缓存测试，未清空 OS 缓存；不代表冷启动、慢盘或云盘性能。",
             "- 每条正文 500 UTF-8 字节，每 15 分钟一条；16 个 topic 标签，20% 记录各有 3 次浏览。",
             "- 新增和浏览计数启用 fsync；每组使用独立临时目录。新增样本会追加到测试库。", "",
             "| 初始记录数 | 操作 | 中位数 ms | P95 ms | 峰值 RSS MiB |",
             "|---:|---|---:|---:|---:|"]
    for size in map(int, args.sizes.split(",")):
        with tempfile.TemporaryDirectory(prefix="inbox-bench-") as tmp:
            root = Path(tmp)
            note_id = dataset(root, size)
            base = [str(binary), "--dir", str(root)]
            commands = [
                ("新增一条", ["-m", "benchmark append", "-t", "idea"]),
                ("最近 20 条", ["list"]),
                ("标签筛选 20 条", ["list", "-t", "topic3"]),
                ("全文搜索 20 条", ["search", "benchmark"]),
                ("全文搜索无结果", ["search", "__missing__"]),
                ("优先级 Top-20", ["list", "--sort", "priority"]),
                ("查看单条并计数", ["show", note_id]),
            ]
            for label, cmd in commands:
                median, p95 = measure(base + cmd, args.samples)
                memory = rss(base + cmd)
                memory_text = f"{memory:.2f}" if memory is not None else "未测量"
                line = f"| {size:,} | {label} | {median:.2f} | {p95:.2f} | {memory_text} |"
                lines.append(line)
                print(line, flush=True)
            run(base + ["doctor"])
    lines += ["", "复现：", "", "```bash", "cargo build --release --locked", "python3 scripts/benchmark.py", "```", "",
              "无结果或不足 limit 的全文搜索、show 和优先级排序执行全库扫描，较大数据集延迟随条数增长；记录路径不扫描历史。"]
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()
