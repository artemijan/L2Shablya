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
        if let Some(player_in_game) = self.players.write().await.remove(&account_name) {
            if let Some(gs) = player_in_game.game_server {
                let _ = self.notify_gs(gs, Box::new(KickPlayer::new(&account_name))).await;
            } else {
                let _ = self.notify_all_gs(|| Box::new(KickPlayer::new(&account_name))).await;
            }
            return Err(error::PacketRun {
                msg: Some(format!("Account in use: {account_name}, IP {:?}", player_in_game.ip_address)),
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
        self.players.write().await.insert(account_name, player_info); // if player exists it will drop
        Ok(())
    }

    pub async fn get_player(&self, account_name: &str) -> Option<player::Info> {
        self.players.read().await.get(account_name).cloned()
    }
    pub async fn with_player<F,R>(&self, account_name: &str, f:F) -> bool
    where
        F: FnOnce(&mut player::Info) -> R,
    {
        let mut lock = self.players.write().await;
        if let Some(player) = lock.get_mut(account_name){
            f(player);
            true
        }else{
            false
        }
    }
    pub async fn remove_player(&self, account_name: &str) {
        self.players.write().await.remove(account_name);
    }
}