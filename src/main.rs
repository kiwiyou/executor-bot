use std::env;
use std::process::Stdio;
use std::time::Duration;

use command::{init_parser, Args, CommandMatcher, ExecutorCommand};
use once_cell::sync::Lazy;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::utils::markdown::code_block;
use tempfile::tempdir;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

mod command;
use tokio_stream::wrappers::UnboundedReceiverStream;

static MATCHER: Lazy<CommandMatcher<ExecutorCommand>> = Lazy::new(init_parser);

async fn on_text(cx: UpdateWithCx<AutoSend<Bot>, Message>) {
    let text = if let Some(text) = cx.update.text() {
        text
    } else {
        return;
    };
    let mut is_replied = false;
    let mut cmd_line = text;
    if let Some(text) = cx.update.reply_to_message().and_then(|m| m.text()) {
        cmd_line = text;
        is_replied = true;
    }
    if !cmd_line.starts_with('/') {
        return;
    }
    match MATCHER.find(&cmd_line[1..]) {
        Some(ExecutorCommand::Help) | None => {
            if !is_replied {
                cx.reply_to(
                    "사용할 수 있는 언어 목록입니다.\n\
                    /help - 도움말을 봅니다.\n\
                    /run - 스크립트를 실행합니다.",
                )
                .await
                .expect("Telegram fail");
            }
        }
        Some(ExecutorCommand::Run) => {
            let mut args = Args::wrap(&cmd_line);
            args.next();
            let lang_code = if let Some(code) = args.next() {
                code
            } else {
                if !is_replied {
                    cx.reply_to(format!(
                        "사용법이 잘못되었습니다.\n\
                            /run <언어> <코드>\n\
                            <언어>로 <코드>를 실행합니다.\n\
                            {}",
                        available_languages()
                    ))
                    .await
                    .expect("Telegram fail");
                }
                return;
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
                    .await
                    .expect("Telegram fail");
                }
                return;
            };
            let code = args.as_str();
            let run = if is_replied {
                run_script(lang, &code, text)
            } else {
                run_script(lang, &code, "")
            }
            .await;
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
                .await
                .expect("Telegram fail");
        }
    }
}

#[tokio::main]
async fn main() {
    teloxide::enable_logging!();
    let bot = Bot::from_env().auto_send();

    Dispatcher::new(bot)
        .messages_handler(|rx| UnboundedReceiverStream::new(rx).for_each_concurrent(None, on_text))
        .dispatch()
        .await;
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
    let mut child = Command::new("/bin/sh")
        .current_dir(dir.path())
        .args(&["-c", lang.run])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let stdin = child.stdin.as_mut().expect("Stdin must be piped");
    stdin.write_all(input.as_bytes()).await?;
    let wait = child.wait_with_output();
    let output = tokio::time::timeout(Duration::from_secs(5), wait).await??;
    if !output.status.success() {
        let msg = String::from_utf8_lossy(&output.stderr);
        Err(eyre::eyre!(msg.to_string()))
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }
}

struct Language {
    code: &'static str,
    ext: &'static str,
    compile: &'static [&'static str],
    run: &'static str,
}

const LANGUAGES: &[Language] = &[
    Language {
        code: "rs",
        ext: "rs",
        compile: &["rustc --edition=2018 -O -o main main.rs"],
        run: "./main",
    },
    Language {
        code: "cpp",
        ext: "cc",
        compile: &["g++ -std=c++20 -o main -O3 main.cc"],
        run: "./main",
    },
    Language {
        code: "hs",
        ext: "hs",
        compile: &["ghc -fllvm -dynamic -o main main.hs"],
        run: "./main",
    },
    Language {
        code: "c",
        ext: "c",
        compile: &["gcc -std=c17 -o main -O3 main.c"],
        run: "./main",
    },
    Language {
        code: "py",
        ext: "py",
        compile: &["python3 -c 'import py_compile; py_compile.compile(\"main.py\")'"],
        run: "python3 main.py",
    },
    Language {
        code: "js",
        ext: "js",
        compile: &[],
        run: "node main.js",
    },
    Language {
        code: "sh",
        ext: "sh",
        compile: &["chmod +x main.sh"],
        run: "bash main.sh",
    },
    Language {
        code: "go",
        ext: "go",
        compile: &["go build main.go"],
        run: "./main",
    },
    Language {
        code: "java",
        ext: "java",
        compile: &["mv main.java Main.java", "javac Main.java"],
        run: "java Main",
    },
];
