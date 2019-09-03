use reqwest;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct TelegramSource {
    pub id: u32,
    pub is_bot: bool,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub language_code: String,
}

#[derive(Deserialize, Debug)]
pub struct TelegramChat {
    pub id: u32,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub r#type: String,
}

#[derive(Deserialize, Debug)]
pub struct TelegramMessage {
    pub message_id: u32,
    pub from: TelegramSource,
    pub chat: TelegramChat,
    pub date: u64,
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct TelegramBotUpdate {
    pub update_id: u32,
    pub message: TelegramMessage,
}

#[derive(Deserialize, Debug)]
pub struct TelegramResult {
    pub r#ok: bool,
    pub result: Vec<TelegramBotUpdate>,
}

pub struct TelegramBot {
    bot_token: String,
    chat_id: String,
    last_read_upd: u32,
    from_usr_id: Option<u32>,
}

impl TelegramBot {
    pub fn new<T: Into<String>>(bot_token: T, chat_id: T, from_usr_id: Option<u32>) -> TelegramBot {
        let last_read_upd = 0;
        TelegramBot {
            bot_token: bot_token.into(),
            chat_id: chat_id.into(),
            last_read_upd,
            from_usr_id,
        }
    }

    pub fn send_message(&self, msg: String, notify: bool) -> Result<(), reqwest::Error> {
        let req = format!(
            "https://api.telegram.org/bot{}/sendMessage?chat_id={}&disable_notification={}&parse_mode=Markdown&text={}",
            self.bot_token,
            self.chat_id,
            !notify,
            msg.to_owned()
        );
        let _ = reqwest::get(&req)?;
        Ok(())
    }

    pub fn remove_message(&self, msg_id: u32) -> Result<(), reqwest::Error> {
        let req = format!(
            "https://api.telegram.org/bot{}/deleteMessage?chat_id={}&message_id={}",
            self.bot_token, self.chat_id, msg_id
        );
        let _ = reqwest::get(&req)?;
        Ok(())
    }

    pub fn get_unread_updates(&mut self) -> Result<TelegramResult, reqwest::Error> {
        let req = format!(
            "https://api.telegram.org/bot{}/getUpdates?offset={}",
            self.bot_token, self.last_read_upd
        );
        let mut resp: TelegramResult = reqwest::get(&req)?.json()?;
        resp.result = if self.from_usr_id.is_some() {
            resp.result
                .into_iter()
                .filter(|x| x.message.from.id == self.from_usr_id.unwrap())
                .collect::<Vec<TelegramBotUpdate>>()
        } else {
            resp.result
        };
        self.last_read_upd = match resp.result.last() {
            Some(upd) => upd.update_id + 1,
            None => self.last_read_upd,
        };
        Ok(resp)
    }
}
