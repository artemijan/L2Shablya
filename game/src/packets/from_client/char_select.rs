use crate::packets::to_client::CharSelected;
use crate::pl_client::{ClientStatus, PlayerClient};
use anyhow::bail;
use bytes::BytesMut;
use kameo::message::Context;
use kameo::prelude::Message;
use tracing::instrument;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;

#[derive(Debug, Clone)]
pub struct SelectChar {
    char_slot: i32,
}

impl ReadablePacket for SelectChar {
    const PACKET_ID: u8 = 0x12;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        Ok(Self {
            char_slot: buffer.read_i32()?,
        })
        //unknown part
        //buffer.read_i16()
        //buffer.read_i32()
        //buffer.read_i32()
    }
}
impl Message<SelectChar> for PlayerClient {
    type Reply = anyhow::Result<()>;
    
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: SelectChar,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        //todo flood protection, ban check, maybe set online status in DB?
        if self.get_selected_char_slot().is_some() {
            bail!("Char already selected")
        }
        self.try_get_session_key()?;
        self.try_get_user()?;
        self.try_get_char_by_slot_id(msg.char_slot)?;
        let game_time = self.controller.get_game_time();
        self.set_status(ClientStatus::Entering);
        self.select_char(msg.char_slot);
        let char_info = self.try_get_char_by_slot_id(msg.char_slot)?;
        self.send_packet(
            CharSelected::new(
                char_info,
                self.try_get_session_key()?.get_play_session_id(),
                game_time,
            )?
            .buffer,
        )
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::Controller;
    use crate::test_utils::test::spawn_custom_player_client_actor;
    use entities::test_factories::factories::{char_factory, user_factory};
    use kameo::actor::ActorRef;
    use l2_core::config::gs::GSServerConfig;
    use l2_core::game_objects::player::Player;
    use l2_core::session::SessionKey;
    use l2_core::traits::ServerConfig;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::{get_test_db, DBPool};
    use tokio::io::split;

    async fn prepare() -> (Arc<Controller>,DBPool, PlayerClient) {
        let pool = get_test_db().await;
        let cfg = Arc::new(GSServerConfig::from_string(include_str!(
            "../../../../config/game.yaml"
        )));
        let controller = Arc::new(Controller::from_config(cfg));
        let pl_client = PlayerClient::new(Ipv4Addr::LOCALHOST, controller.clone(), pool.clone());

        controller.add_online_account(String::from("test"));
        // Run hook_before
        (controller, pool, pl_client)
    }

    #[tokio::test]
    async fn test_handle_no_session_key() {
        let (controller, pool, pl_client) = prepare().await;
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let pl_actor = spawn_custom_player_client_actor(
            controller.clone(),
            pool.clone(),
            r,
            w,
            Some(pl_client),
        )
            .await;
        let res = pl_actor.ask(SelectChar { char_slot: 0 }).await;
        assert!(res.is_err());
    }
    #[tokio::test]
    async fn test_handle_no_user() {
        let (controller, pool, mut pl_client) = prepare().await;
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        pl_client.set_session_key(SessionKey::new());
        let pl_actor = spawn_custom_player_client_actor(
            controller.clone(),
            pool.clone(),
            r,
            w,
            Some(pl_client),
        )
            .await;
        let res = pl_actor.ask(SelectChar { char_slot: 0 }).await;
        assert!(res.is_err());
    }
    
    #[tokio::test]
    async fn test_handle_no_chars() {
        let (controller, pool, mut pl_client) = prepare().await;
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let user = user_factory(&pool, |mut u| {
            u.username = String::from("test");
            u
        })
            .await;
        pl_client.set_session_key(SessionKey::new());
        pl_client.set_user(user);
        let pl_actor = spawn_custom_player_client_actor(
            controller.clone(),
            pool.clone(),
            r,
            w,
            Some(pl_client),
        )
            .await;
        let res = pl_actor.ask(SelectChar { char_slot: 0 }).await;
        assert!(res.is_err());
    }
    #[tokio::test]
    async fn test_handle_ok() {
        let (controller, pool, mut pl_client) = prepare().await;
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let user = user_factory(&pool, |mut u| {
            u.username = String::from("test");
            u
        })
            .await;
        let char_model = char_factory(&pool, |mut u| {
            u.user_id = user.id;
            u.name = "Char".to_owned();
            u
        })
            .await;
        pl_client.set_session_key(SessionKey::new());
        pl_client.set_user(user);
        pl_client.set_account_chars(vec![Player::new(char_model, vec![])]);
        let pl_actor = spawn_custom_player_client_actor(
            controller.clone(),
            pool.clone(),
            r,
            w,
            Some(pl_client),
        )
            .await;
        let res = pl_actor.ask(SelectChar { char_slot: 0 }).await;
        assert!(res.is_ok());
    }
}
