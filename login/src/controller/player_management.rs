use super::data::LoginController;
use crate::dto::player;
use crate::dto::player::GSCharsInfo;
use crate::gs_client::GSMessages;
use l2_core::shared_packets::common::PlayerLoginFailReasons;
use l2_core::shared_packets::ls_2_gs::{KickPlayer, RequestChars};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::time::Duration;
use tracing::{error, info};

impl LoginController {
    pub async fn on_player_login(
        &self,
        mut player_info: player::Info,
    ) -> anyhow::Result<(), PlayerLoginFailReasons> {
        let account_name = player_info.account_name.clone();
        self.check_player_in_game(&account_name).await?;
        let packet = RequestChars::new(&account_name);
        let timeout =
            Duration::from_secs(self.config.listeners.game_servers.messages.timeout.into());
        for entry in &self.gs_actors {
            let gs_actor = entry.value().clone();
            let gs_id = *entry.key();
            if let Ok(resp_fut) = gs_actor.ask(packet.clone()).reply_timeout(timeout).await {
                if let Ok(GSMessages::ReplyChars(rc)) = resp_fut.await {
                    player_info.chars_on_servers.insert(
                        gs_id,
                        GSCharsInfo {
                            char_deletion_timestamps: rc.char_deletion_timestamps,
                            chars_to_delete: rc.delete_chars_len,
                            total_chars: rc.chars,
                        },
                    );
                }
            }
        }
        self.players.insert(account_name, player_info); // if player exists it will drop
        Ok(())
    }

    async fn check_player_in_game(
        &self,
        account_name: &str,
    ) -> anyhow::Result<(), PlayerLoginFailReasons> {
        if let Some(player_in_game) = self.players.remove(account_name) {
            let packet = KickPlayer::new(account_name);
            if let Some(gs) = player_in_game.1.game_server {
                let gs_actor = self
                    .gs_actors
                    .get(&gs)
                    .ok_or(PlayerLoginFailReasons::ReasonSystemErrorLoginLater)?;
                let _ = gs_actor.tell(packet.clone()).await;
            } else {
                for entry in self.gs_actors.iter() {
                    let _ = entry.value().tell(packet.clone()).await;
                }
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
        if let Some((acc, pl)) = self.players.remove(account_name) {
            if let Some(pl_actor) = pl.player_actor {
                //process in the background, no need make caller wait
                if pl_actor.is_alive() {
                    tokio::spawn(async move {
                        if pl_actor.is_alive() && let Err(e) = pl_actor.stop_gracefully().await {
                            error!("Failed to stop player {acc}, actor: {e:?}");
                        }
                    });
                }
            }
        }
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
        rng.r#gen()
    }
}
