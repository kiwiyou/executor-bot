use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, Instant};

use crate::language::{Language, LANGUAGES};

use super::Args;
use log::info;
use teloxide::prelude::*;
use teloxide::types::{ParseMode, User};
use teloxide::utils::markdown::code_block;
use teloxide::Bot;
use tempfile::tempdir;
use tokio::fs::{create_dir_all, write};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub async fn help(cx: UpdateWithCx<AutoSend<Bot>, Message>) -> eyre::Result<()> {
    cx.reply_to(
        "사용할 수 있는 명령어 목록입니다.\n\
            /help - 도움말을 봅니다.\n\
            /eval - 스크립트를 실행합니다.",
    )
    .await?;
    Ok(())
}

pub async fn run(
    cx: &UpdateWithCx<AutoSend<Bot>, Message>,
    from: &User,
    mut args: Args<'_>,
    text: &str,
    is_replied: bool,
) -> eyre::Result<()> {
    let lang_code = if let Some(code) = args.next() {
        code
    } else {
        if !is_replied {
            cx.reply_to(format!(
                "사용법이 잘못되었습니다.\n\
                    /eval <언어> <코드>\n\
                    <언어>로 <코드>를 실행합니다.\n\
                    {}",
                available_languages()
            ))
            .await?;
        }
        return Ok(());
    };
    let lang = if let Some(lang) = LANGUAGES.iter().find(|lang| lang.code == lang_code) {
        lang
    } else {
        if !is_replied {
            cx.reply_to(format!(
                "{}은(는) 사용할 수 없는 언어입니다.\n\
                    {}",
                lang_code,
                available_languages()
            ))
            .await?;
        }
        return Ok(());
    };
    let code = args.as_str();
    let script_begin = Instant::now();
    let run = if is_replied {
        run_script(lang, &code, text)
    } else {
        run_script(lang, &code, "")
    }
    .await;
    let script_end = Instant::now();
    info!(
        "user {} invoked {} code in {:#?}",
        from.id,
        lang.code,
        script_end - script_begin
    );
    save_code(from, lang, code).await?;
    let response = match run {
        Ok(stdout) => {
            if stdout.is_empty() {
                "(출력 없음)".into()
            } else {
                stdout
            }
        }
        Err(error) => {
            format!("오류가 발생했습니다.\n{}", error.to_string())
        }
    };
    let trunc: String = response.chars().take(1500).collect();
    cx.reply_to(code_block(&trunc))
        .parse_mode(ParseMode::MarkdownV2)
        .await?;
    Ok(())
}

fn available_languages() -> String {
    let mut out = "사용 가능한 언어:\n".to_string();
    let langs: String = LANGUAGES
        .iter()
        .map(|l| l.code.to_string())
        .reduce(|mut prev, s| {
            prev.push_str(", ");
            prev.push_str(&s);
            prev
        })
        .expect("LANGAUGES must have one or more elements");
    out.push_str(&langs);
    out
}

async fn run_script(lang: &Language, code: &str, input: &str) -> eyre::Result<String> {
    let dir = tempdir()?;
    let mut path = dir.path().join("main");
    path.set_extension(lang.ext);
    std::fs::write(path, code)?;
    for compile in lang.compile {
        let output = Command::new("/bin/sh")
            .current_dir(dir.path())
            .args(&["-c", compile])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .await?;
        if !output.status.success() {
            let msg = String::from_utf8_lossy(&output.stderr);
            eyre::bail!(msg.to_string());
        }
    }
    let mut child = Command::new("firejail")
        .args(&[
            "--quiet",
            "--net=none",
            "--private-cwd",
            "--private-opt=none",
            "--private-etc=none",
        ])
        .arg(format!("--private={}", dir.path().display()))
        .args(&["/bin/bash", "-c"])
        .arg(format!("ulimit -v 2000000 -f 100 && {}", lang.run))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;
    let stdin = child.stdin.as_mut().expect("Stdin must be piped");
    stdin.write_all(input.as_bytes()).await?;
    let wait = child.wait_with_output();
    let output = tokio::time::timeout(Duration::from_secs(5), wait).await??;
    if !output.status.success() {
        let msg = String::from_utf8_lossy(&output.stderr) + String::from_utf8_lossy(&output.stdout);
        Err(eyre::eyre!(msg.to_string()))
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }
}

async fn save_code(from: &User, lang: &Language, code: &str) -> eyre::Result<()> {
    let now = chrono::Local::now();
    let filename = format!("{}_{}", now.format("%Y%m%d_%H%M%S"), from.id);
    let mut dir = PathBuf::from("codes");
    create_dir_all(&dir).await?;
    dir.push(filename);
    dir.set_extension(lang.ext);
    write(&dir, code).await?;
    Ok(())
}
