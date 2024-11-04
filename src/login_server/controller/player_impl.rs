use crate::common::dto::player;
use crate::common::dto::player::GSCharsInfo;
use super::data::Login;
use crate::packet::{error, PlayerLoginFailReasons};
use crate::packet::common::PacketType;
use crate::packet::login_fail::PlayerLogin;
use crate::packet::to_gs::{KickPlayer, RequestChars};

impl Login {
    pub async fn on_player_login(
        &self,
        mut player_info: player::Info,
    ) -> anyhow::Result<(), error::PacketRun> {
        let account_name = player_info.account_name.clone();
        if let Some(player_in_game) = self.players.remove(&account_name) {
            if let Some(gs) = player_in_game.1.game_server {
                let _ = self.notify_gs(gs, Box::new(KickPlayer::new(&account_name))).await;
            } else {
                let _ = self.notify_all_gs(|| Box::new(KickPlayer::new(&account_name))).await;
            }
            return Err(error::PacketRun {
                msg: Some(format!("Account in use: {account_name}, IP {:?}", player_in_game.1.ip_address)),
                response: Some(Box::new(PlayerLogin::new(PlayerLoginFailReasons::ReasonAccountInUse))),
            });
        }
        let mut task_results = self.send_message_to_all_gs(
            &account_name, || Box::new(RequestChars::new(&account_name)),
        ).await.into_iter();

        while let Some(Ok(resp)) = task_results.next() {
            if let Some((gs_id, PacketType::ReplyChars(p))) = resp {
                player_info.chars_on_servers.insert(
                    gs_id,
                    GSCharsInfo {
                        char_list: p.char_list,
                        chars_to_delete: p.chars_to_delete,
                        chars: p.chars,
                    },
                );
            }
            // ignore all the tasks that are timed out
        }
        self.players.insert(account_name, player_info); // if player exists it will drop
        Ok(())
    }

    pub fn on_player_logout(&self, account_name: &str) {
        println!("Player logged out: {account_name}");
        self.players.remove(account_name);
    }

    pub async fn get_player(&self, account_name: &str) -> Option<player::Info> {
        self.players.get(account_name).map(|i| i.clone())
    }
    pub fn with_player<F>(&self, account_name: &str, f: F) -> bool
    where
        F: FnOnce(&mut player::Info) -> bool,
    {
        if let Some(mut player) = self.players.get_mut(account_name) {
            f(&mut player)
        } else {
            false
        }
    }
    pub async fn remove_player(&self, account_name: &str) {
        self.players.remove(account_name);
    }
}