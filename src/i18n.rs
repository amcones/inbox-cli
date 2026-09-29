use crate::Result;
use std::{cell::Cell, ffi::OsString};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Zh,
    En,
}

thread_local! {
    // Each CLI process chooses this once, before parsing or opening any files.
    static CURRENT: Cell<Language> = const { Cell::new(Language::En) };
}

pub fn language() -> Language {
    CURRENT.get()
}
pub fn text(zh: &'static str, en: &'static str) -> &'static str {
    match language() {
        Language::Zh => zh,
        Language::En => en,
    }
}

#[macro_export]
macro_rules! message {
    ($zh:literal, $en:literal $(, $arg:expr)* $(,)?) => {
        match $crate::i18n::language() {
            $crate::i18n::Language::Zh => format!($zh $(, $arg)*),
            $crate::i18n::Language::En => format!($en $(, $arg)*),
        }
    };
}

pub fn locale(value: &str) -> Language {
    let first = value
        .trim()
        .split(['_', '-', '.', '@'])
        .next()
        .unwrap_or("");
    if first.eq_ignore_ascii_case("zh") {
        Language::Zh
    } else {
        Language::En
    }
}

fn system_language() -> Language {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(value) = std::env::var(key)
            && !value.trim().is_empty()
        {
            return locale(&value);
        }
    }
    // Normal terminals supply a locale. Only consult macOS preferences when
    // none is set, avoiding a subprocess on the usual recording path.
    #[cfg(target_os = "macos")]
    if let Ok(output) = std::process::Command::new("/usr/bin/defaults")
        .args(["read", "-g", "AppleLanguages"])
        .output()
        && output.status.success()
        && let Some(first) = std::str::from_utf8(&output.stdout).ok().and_then(|s| {
            s.split(|c: char| c.is_whitespace() || matches!(c, '(' | ')' | '"' | ','))
                .find(|s| !s.is_empty())
        })
    {
        return locale(first);
    }
    Language::En
}

/// A lexical prepass makes `--help --lang en` and argument errors use the
/// requested language, without interpreting message/tag values as options.
pub fn configure(args: &[OsString]) -> Result<()> {
    use lexopt::{
        Arg::{Long, Short},
        ValueExt,
    };
    CURRENT.set(system_language());
    let mut parser = lexopt::Parser::from_args(args.iter().cloned());
    let mut selected = None;
    while let Some(arg) = parser.next()? {
        match arg {
            Long("lang") => {
                selected = Some(
                    parser
                        .value()
                        .map_err(|_| text("--lang 缺少语言参数", "--lang requires a language"))?
                        .string()?,
                );
            }
            // Consume values; the full parser will diagnose missing ones.
            Short('m' | 't' | 'n') | Long("message" | "tag" | "limit" | "sort" | "dir") => {
                let missing = parser.value().is_err();
                if missing {
                    break;
                }
            }
            _ => (),
        }
    }
    let selected = selected.or_else(|| std::env::var("INBOX_LANG").ok());
    match selected.as_deref().unwrap_or("auto") {
        "auto" => (),
        "zh" => CURRENT.set(Language::Zh),
        "en" => CURRENT.set(Language::En),
        _ => {
            return Err(text(
                "语言只支持 auto、zh 或 en",
                "Language must be auto, zh, or en",
            )
            .into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn locale_variants_and_fallback() {
        for tag in ["zh_CN.UTF-8", "zh-TW", "zh-Hant-HK", "ZH", "zh_SG"] {
            assert_eq!(locale(tag), Language::Zh);
        }
        for tag in ["en_US.UTF-8", "C", "POSIX", "fr_FR", "", "zho"] {
            assert_eq!(locale(tag), Language::En);
        }
    }
}
