use crate::Result;
use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

impl FromStr for Shell {
    type Err = Box<dyn std::error::Error>;

    fn from_str(value: &str) -> Result<Self> {
        match value.to_ascii_lowercase().as_str() {
            "bash" => Ok(Self::Bash),
            "zsh" => Ok(Self::Zsh),
            "fish" => Ok(Self::Fish),
            "powershell" | "pwsh" => Ok(Self::PowerShell),
            _ => Err(crate::i18n::text(
                "shell 只支持 bash、zsh、fish 或 powershell",
                "Shell must be bash, zsh, fish, or powershell",
            )
            .into()),
        }
    }
}

impl fmt::Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Bash => BASH,
            Self::Zsh => ZSH,
            Self::Fish => FISH,
            Self::PowerShell => POWERSHELL,
        })
    }
}

#[cfg(test)]
const COMMANDS: &str =
    "add edit list search review show delete trash restore tags doctor completions";

const BASH: &str = r#"_inbox() {
    local cur
    cur="${COMP_WORDS[COMP_CWORD]}"
    if [[ $COMP_CWORD -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "add edit list search review show delete trash restore tags doctor completions" -- "$cur") )
    elif [[ ${COMP_WORDS[1]} == delete && $COMP_CWORD -eq 2 ]]; then
        COMPREPLY=( $(compgen -W "today range all" -- "$cur") )
    elif [[ ${COMP_WORDS[1]} == trash && $COMP_CWORD -eq 2 ]]; then
        COMPREPLY=( $(compgen -W "empty" -- "$cur") )
    elif [[ ${COMP_WORDS[1]} == completions && $COMP_CWORD -eq 2 ]]; then
        COMPREPLY=( $(compgen -W "bash zsh fish powershell" -- "$cur") )
    fi
}
complete -F _inbox inbox
"#;

const ZSH: &str = r#"#compdef inbox
_inbox() {
  local -a commands
  commands=(
    'add:add an idea' 'edit:edit an idea' 'list:list ideas'
    'search:search ideas' 'review:review priority candidates' 'show:show an idea'
    'delete:move ideas to trash' 'trash:list or empty trash' 'restore:restore an idea'
    'tags:list tags' 'doctor:check stored data' 'completions:generate shell completion'
  )
  if (( CURRENT == 2 )); then
    _describe 'command' commands
    return
  fi
  case $words[2] in
    delete) (( CURRENT == 3 )) && _values 'target' today range all ;;
    trash) (( CURRENT == 3 )) && _values 'action' empty ;;
    completions) (( CURRENT == 3 )) && _values 'shell' bash zsh fish powershell ;;
  esac
}
compdef _inbox inbox
"#;

const FISH: &str = r#"complete -c inbox -f
complete -c inbox -n '__fish_use_subcommand' -a 'add edit list search review show delete trash restore tags doctor completions'
complete -c inbox -n '__fish_seen_subcommand_from delete' -a 'today range all'
complete -c inbox -n '__fish_seen_subcommand_from trash' -a 'empty'
complete -c inbox -n '__fish_seen_subcommand_from completions' -a 'bash zsh fish powershell'
"#;

const POWERSHELL: &str = r#"Register-ArgumentCompleter -Native -CommandName inbox -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    $commands = 'add','edit','list','search','review','show','delete','trash','restore','tags','doctor','completions'
    $elements = @($commandAst.CommandElements)
    if ($elements.Count -le 2) {
        $commands | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_completion_contains_all_top_level_commands() {
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
            let script = shell.to_string();
            for command in COMMANDS.split_whitespace() {
                assert!(script.contains(command), "{shell:?} lacks {command}");
            }
            assert!(script.ends_with('\n'));
        }
    }
}
