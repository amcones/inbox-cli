"""Exercise real ZLE Tab completion in an isolated pseudoterminal."""
import os
import pathlib
import pty
import select
import signal
import subprocess
import tempfile
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]


def read_until(fd, marker):
    output = b""
    deadline = time.monotonic() + 8
    while marker not in output:
        assert time.monotonic() < deadline, output.decode(errors="replace")
        if select.select([fd], [], [], 0.1)[0]:
            output += os.read(fd, 65536)
    return output


with tempfile.TemporaryDirectory(prefix="inbox-zle-") as directory:
    root = pathlib.Path(directory)
    binary = ROOT / "target/debug/inbox"
    env = dict(os.environ, INBOX_DIR=str(root / "data"), ZDOTDIR=directory,
               PATH=str(binary.parent) + os.pathsep + os.environ["PATH"], TERM="xterm")
    def inbox(*args):
        return subprocess.check_output([str(binary), *args], env=env, text=True).strip()

    active = [inbox("add", text, "-t", "work") for text in ("active one", "active two")]
    trash = [inbox("add", text) for text in ("trash one", "trash two")]
    for note in trash:
        inbox("delete", note)
    # Capture candidates inside the actual completion widget, preserving all
    # matching and positional processing performed by Zsh's completion system.
    rc = f"""
autoload -Uz compinit
compinit -D
source {ROOT}/scripts/completion.zsh
PROMPT='READY> '
compadd() {{
  local -a captured
  builtin compadd -O captured "$@"
  if (( $#captured )); then
    print -rl -- "${{captured[@]}}" >> {root}/matches
  fi
  builtin compadd "$@"
}}
_test_tab() {{ zle complete-word; print -rn -- 'TABDONE' }}
zle -N _test_tab
bindkey '^I' _test_tab
_test_reset() {{ BUFFER=''; CURSOR=0; print -rn -- 'TESTDONE'; zle reset-prompt }}
zle -N _test_reset
bindkey '^G' _test_reset
"""
    (root / ".zshrc").write_text(rc)
    pid, master = pty.fork()
    if pid == 0:
        os.execvpe("zsh", ["zsh", "-i"], env)
    try:
        read_until(master, b"READY> ")
        cases = [
            ("he", ["help"], ["--help"]),
            ("inf", ["info"], ["--help"]),
            ("restore ", trash, active + ["--dir"]),
            ("show ", active, trash + ["--no-track", "--dir"]),
            ("edit ", active, ["--clear-tags"]),
            ("delete ", active + ["today", "range", "all"], ["--yes"]),
            ("show -", ["--no-track", "--dir"], ["--help", "--tag", "--yes", "--limit"]),
            ("restore -", ["--from", "--dir"], ["--help", "--no-track", "--tag"]),
            ("backup -", ["--dir"], ["--from", "--yes", "--tag"]),
            ("backup ", ["verify"], ["--from", "--yes"]),
            ("list -t ", ["work"], ["--help"]),
            ("delete today -", ["--yes"], ["--tag", "--no-track"]),
            ("trash -", ["--dir"], ["--help", "--yes", "--tag"]),
        ]
        for line, wanted, forbidden in cases:
            (root / "matches").write_text("")
            os.write(master, ("inbox " + line + "\t").encode())
            read_until(master, b"TABDONE")
            os.write(master, b"\x07")
            read_until(master, b"TESTDONE")
            matches = (root / "matches").read_text().splitlines()
            assert all(x in matches for x in wanted), (line, wanted, matches)
            assert not any(x in matches for x in forbidden), (line, forbidden, matches)
            print(f"PASS inbox {line}<Tab>")
    finally:
        os.kill(pid, signal.SIGKILL)
        os.waitpid(pid, 0)
        os.close(master)
