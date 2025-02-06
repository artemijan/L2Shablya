use super::data::LoginController;
use l2_core::shared_packets::common::{PacketType, PlayerLoginFailReasons};
use l2_core::shared_packets::ls_2_gs::{KickPlayer, RequestChars};
use crate::dto::player;
use crate::dto::player::GSCharsInfo;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use tracing::info;

impl LoginController {
    pub async fn on_player_login(
        &self,
        mut player_info: player::Info,
    ) -> anyhow::Result<(), PlayerLoginFailReasons> {
        let account_name = player_info.account_name.clone();
        self.check_player_in_game(&account_name).await?;
        let mut task_results = self.message_broker
            .send_message_to_all(&account_name, || Box::new(RequestChars::new(&account_name)))
            .await
            .into_iter();

        while let Some(Ok(resp)) = task_results.next() {
            if let Some((gs_id, PacketType::ReplyChars(p))) = resp {
                player_info.chars_on_servers.insert(
                    gs_id,
                    GSCharsInfo {
                        char_deletion_timestamps: p.char_deletion_timestamps,
                        chars_to_delete: p.delete_chars_len,
                        total_chars: p.chars,
                    },
                );
            }
            // ignore all the tasks that are timed out
        }
        self.players.insert(account_name, player_info); // if player exists it will drop
        Ok(())
    }

    async fn check_player_in_game(
        &self,
        account_name: &str,
    ) -> anyhow::Result<(), PlayerLoginFailReasons> {
        if let Some(player_in_game) = self.players.remove(account_name) {
            if let Some(gs) = player_in_game.1.game_server {
                let _ = self.message_broker
                    .notify(gs, Box::new(KickPlayer::new(account_name)))
                    .await;
            } else {
                let _ = self.message_broker
                    .notify_all(|| Box::new(KickPlayer::new(account_name)))
                    .await;
            }
            return Err(PlayerLoginFailReasons::ReasonAccountInUse);
        }
        Ok(())
    }

    pub fn on_player_logout(&self, account_name: &str) {
        info!("Player logged out: {account_name}");
        self.remove_player(account_name);
    }

    pub fn get_player(&self, account_name: &str) -> Option<player::Info> {
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
    pub fn remove_player(&self, account_name: &str) {
        self.players.remove(account_name);
    }

    pub fn remove_all_gs_players(&self, gs_id: u8) {
        let accounts_to_remove: Vec<_> = self
            .players
            .iter()
            .filter(|entry| entry.value().game_server == Some(gs_id))
            .map(|entry| entry.key().clone())
            .collect();
        for acc in accounts_to_remove {
            self.remove_player(&acc);
        }
    }

    pub fn on_players_in_game(&self, gs_id: u8, account_name: &[String]) {
        for acc_name in account_name {
            let op_success = self.with_player(acc_name, |pl| {
                pl.game_server = Some(gs_id);
                pl.is_joined_gs = true;
                true
            });
            if !op_success {
                let pl = player::Info {
                    game_server: Some(gs_id),
                    account_name: acc_name.clone(),
                    is_joined_gs: true,
                    is_authed: true,
                    ..player::Info::default()
                };
                self.players.insert(acc_name.clone(), pl);
            }
        }
    }

    pub fn generate_session_id<T>() -> T
    where
        Standard: Distribution<T>,
    {
        let mut rng = rand::thread_rng();
        rng.gen()
    }
}
