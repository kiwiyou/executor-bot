#![feature(str_split_whitespace_as_str)]
#![feature(iter_intersperse)]

use std::env;
use std::process::Stdio;

use futures::StreamExt;
use regex::Regex;
use telegram_bot::ParseMode::MarkdownV2;
use telegram_bot::{Api, CanReplySendMessage, MessageText, UpdateKind};
use tempfile::tempdir;
use tokio::process::Command;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let token = env::var("BOT_TOKEN").expect("BOT_TOKEN not found");
    let api = Api::new(token);

    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        let update = update?;
        if let UpdateKind::Message(message) = update.kind {
            if let Some(ref text) = message.text() {
                if text.starts_with("/run") {
                    let mut parts = text.split_ascii_whitespace();
                    parts.next();
                    if let Some(raw_lang) = parts.next() {
                        let lang = LANGUAGES.iter().find(|lang| lang.code == raw_lang);

                        if let Some(lang) = lang {
                            let code = parts.as_str();
                            let dir = tempdir()?;
                            let mut path = dir.path().join("main");
                            path.set_extension(lang.ext);
                            std::fs::write(path, code)?;
                            let regex = Regex::new(r"[_\*\[\]\(\)~`>#\+-=\|\{\}\.!]")?;
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
                                    let trunc: String = msg.chars().take(1500).collect();
                                    let escape = regex.replace_all(&trunc, r"\$0");
                                    let text = format!("Failed to compile:```\n{}\n```", escape);
                                    api.spawn(message.text_reply(text).parse_mode(MarkdownV2));
                                    continue;
                                }
                            }
                            let output = Command::new("/bin/sh")
                                .current_dir(dir.path())
                                .args(&["-c", lang.run])
                                .stdin(Stdio::null())
                                .stdout(Stdio::piped())
                                .stderr(Stdio::null())
                                .output()
                                .await?;
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            api.spawn(message.text_reply(stdout));
                        } else {
                            let req = message.text_reply(format!(
                                "Invalid language.\n{}",
                                available_languages()
                            ));
                            api.spawn(req);
                        }
                    } else {
                        let req = message.text_reply(format!(
                            "No language specified.\n{}",
                            available_languages()
                        ));
                        api.spawn(req);
                    }
                }
            }
        }
    }

    Ok(())
}

fn available_languages() -> String {
    let mut out = "Available languages are:\n".to_string();
    let langs: String = LANGUAGES.iter().map(|l| l.code).intersperse(", ").collect();
    out.push_str(&langs);
    out
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
];
