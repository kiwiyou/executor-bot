use std::env;

use command::{init_parser, Args, CommandMatcher, ExecutorCommand};
use log::error;
use once_cell::sync::Lazy;
use teloxide::prelude::*;

mod command;
pub mod language;
use tokio_stream::wrappers::UnboundedReceiverStream;

static MATCHER: Lazy<CommandMatcher<ExecutorCommand>> = Lazy::new(init_parser);

async fn on_text(cx: UpdateWithCx<AutoSend<Bot>, Message>) {
    let from = if let Some(from) = cx.update.from() {
        from
    } else {
        return;
    };
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
    // first character of cmd_line is '/'
    let mut args = Args::wrap(&cmd_line[1..]);
    if let Some(mut label) = args.next() {
        if let Some(username_pos) = label.rfind('@') {
            label = &label[..username_pos];
        }
        if let Some(command) = MATCHER.find(label) {
            let result = match command {
                ExecutorCommand::Help if !is_replied => command::handler::help(cx).await,
                ExecutorCommand::Run => {
                    command::handler::run(&cx, from, args, text, is_replied).await
                }
                _ => Ok(()),
            };
            if let Err(e) = result {
                error!("an error occurred processing update: {}", e);
            }
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
