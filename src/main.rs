#![allow(unused_imports, dead_code)]

use either::Either;
pub use std::{
    collections::hash_map::DefaultHasher,
    error::Error,
    hash::{Hash, Hasher},
};
use teloxide::{
    dispatching::dialogue::InMemStorageError, payloads::GetChat, prelude::*, types::*,
    utils::command::BotCommand,
};
use tokio;

const HASH_PREFIX: [u8; 8] = *b"_muctep_";
const BOT_TOKEN: &str = include_str!(r"..\..\BOT_TOKEN");
const ARTHURS: [i64; 1] = [
    5004341011,
];

fn generate_number(chat: Option<Chat>, user: Option<User>) -> u16 {
    let mut h = DefaultHasher::new();
    (
        HASH_PREFIX,
        chat.map(|Chat { id, .. }| id),
        user.map(|User { id, .. }| id),
    )
        .hash(&mut h);
    (h.finish() & 0xffff) as u16
}

const BAR_SIZE: usize = 10;

fn generate_data(n: u16, is_male: bool, username: String) -> ([&'static str; BAR_SIZE], String) {
    match n {
        n if n & 0x0fff == 0x0fff => (
            ["🌈"; BAR_SIZE],
            ("Твой гендер - самый настоящий гейский гей! (вероятность 0.02%)".to_owned()),
        ),
        n if n & 0x07ff == 0x07ff => (
            ["📖"; BAR_SIZE],
            ("Твой гендер - ботан. Халявно получишь только жопе! (вероятность 0.05%)".to_owned()),
        ),
        n if n & 0x03ff == 0x03ff => (
            ["🐷"; BAR_SIZE],
            ("Твой гендер - свинья вонючая, иди спать! (вероятность 0.1%)".to_owned()),
        ),
        n if n & 0x01ff == 0x01ff => (
            ["🍺"; BAR_SIZE],
            ("Твой гендер - пивозавр! (вероятность 0.2%)".to_owned()),
        ),
        n => {
            let frac = (n & 0xff) as f64 / 255.;

            let mut data = ["♀️"; BAR_SIZE];
            for i in 0..((frac * BAR_SIZE as f64).round() as usize) {
                data[i] = "♂️";
            }

            if is_male {
                (data, format!("@{} пидорас на {:.2}%.", username, frac * 100.))
            } else {
                (data, format!("@{} пидораска на {:.2}%.", username, (1. - frac) * 100.))
            }
        }
    }
}

#[derive(BotCommand)]
#[command(rename = "lowercase")]
enum Command {
    #[command(description = "Показать этот текст.")]
    Help,
    #[command(description = "Угадать твою ориентацию.")]
    Guess,
    #[command(description = "Насколько ты Артурчик")]
    Arthur,
}

async fn answer(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    command: Command,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat = cx.update.sender_chat().cloned();
    let user = cx.update.from();

    let is_male = user
        .map(|user| !matches!(user.first_name.chars().last(), Some('a' | 'а' | 'я')))
        .unwrap_or(true);
    let username = user.and_then(|User { username, .. }| username.clone()).unwrap_or(String::new());

    let user = user.cloned();

    log::info!("{:#?}", chat);
    log::info!("{:#?}", user);

    match command {
        Command::Help => cx.answer(Command::descriptions()).await?,
        Command::Guess => {
            let answer = tokio::task::spawn_blocking(move || {
                let n = generate_number(chat, user, );
                let data = generate_data(n, is_male, username);
                format!(
                    "{}\n{}\n\n/guess",
                    data.1,
                    data.0.into_iter().collect::<String>()
                )
            })
            .await?;
            cx.answer(answer).await?
        },
        Command::Arthur => {
            if let Some(user) = user.filter(|User { id, .. }| ARTHURS.contains(id)) {
                cx.answer(format!("@{} - артурчик.\n\nВсе ваши данные:\n{:#?}", username,  user)).await?
            } else {
                cx.answer(format!("@{} не артурчик.", username)).await?
            }
        },
    };

    Ok(())
}
async fn run(bot: AutoSend<Bot>, bot_name: String) {
    teloxide::commands_repl(bot, bot_name, answer).await;
}

fn main() {
    teloxide::enable_logging!();
    log::info!("Starting @gomo_or_getero_bot...");

    let bot = Bot::new(BOT_TOKEN).auto_send();
    let bot_name = String::from("Guess your orientation");

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(run(bot, bot_name));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn different_names() {
        let user = User {
            id: 1234567,
            is_bot: false,
            first_name: "Denis".to_owned(),
            last_name: Some("Drohzhin".to_owned()),
            username: Some("muctep".to_owned()),
            language_code: Some("ru".to_owned()),
        };

        println!("{}", generate_number(None, Some(user)));
    }
}
