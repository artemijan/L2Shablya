use crate::packets::enums::CharNameResponseVariant;
use crate::packets::to_client::{CreateCharFailed, CreateCharOk};
use crate::packets::utils::validate_can_create_char;
use crate::pl_client::{ClientStatus, PlayerClient};
use anyhow::bail;
use bytes::BytesMut;
use entities::entities::character;
use kameo::message::Context;
use kameo::prelude::Message;
use l2_core::data::char_template::CharTemplate;
use l2_core::data::classes::mapping::Class;
use l2_core::game_objects::player::Player;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use sea_orm::DbErr;
use tracing::{error, instrument};

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct CreateCharRequest {
    name: String,
    is_female: bool,
    class_id: Class,
    hair_style: u8,
    hair_color: u8,
    int: u8,
    str: u8,
    con: u8,
    men: u8,
    dex: u8,
    wit: u8,
    face: u8,
    race: u8,
}
impl CreateCharRequest {
    pub fn validate(&self, _: &CharTemplate) -> anyhow::Result<()> {
        if !(0..=2).contains(&self.face) {
            bail!("Invalid face value: {}.", self.face);
        }
        if (!self.is_female && (self.hair_style > 4)) || (self.is_female && (self.hair_style > 6)) {
            bail!(
                "Invalid hair style value: {}. For is_female({})",
                self.hair_style,
                self.is_female
            );
        }
        if self.hair_color > 3 {
            bail!("Invalid hair color value: {}.", self.hair_color);
        }
        Ok(())
    }
}

impl ReadablePacket for CreateCharRequest {
    const PACKET_ID: u8 = 0x0C;
    const EX_PACKET_ID: Option<u16> = None;

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);

        let inst = Self {
            name: buffer.read_c_utf16le_string()?,
            race: buffer.read_i32()? as u8, //ignored
            is_female: buffer.read_u32()? != 0,
            class_id: Class::try_from(u8::try_from(buffer.read_i32()?)?)?,
            int: buffer.read_u32()? as u8, //ignored
            str: buffer.read_u32()? as u8, //ignored
            con: buffer.read_u32()? as u8, //ignored
            men: buffer.read_u32()? as u8, //ignored
            dex: buffer.read_u32()? as u8, //ignored
            wit: buffer.read_u32()? as u8, //ignored
            hair_style: u8::try_from(buffer.read_u32()?)?,
            hair_color: u8::try_from(buffer.read_u32()?)?,
            face: u8::try_from(buffer.read_u32()?)?,
        };
        Ok(inst)
    }
}
impl Message<CreateCharRequest> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, msg, _ctx))]
    async fn handle(
        &mut self,
        msg: CreateCharRequest,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        if &ClientStatus::Authenticated != self.get_status() || self.get_user().is_none() {
            bail!("Not authenticated.");
        }
        let template = self
            .controller
            .class_templates
            .try_get_template(msg.class_id)?;
        msg.validate(template)?;
        let response = validate_can_create_char(&self.db_pool, &msg.name).await?;
        if response == CharNameResponseVariant::Ok {
            let mut char = character::Model {
                name: msg.name.clone(),
                level: 1, //todo take from config
                face: msg.face,
                hair_style: msg.hair_style,
                hair_color: msg.hair_color,
                is_female: msg.is_female,
                delete_at: None,
                user_id: self.try_get_user()?.id,
                ..Default::default()
            };
            template.initialize_character(&mut char, &self.controller.base_stats_table)?;
            match character::Model::create_char(&self.db_pool, char).await {
                Ok(inst) => {
                    self.add_character(Player::new(inst, vec![], template.clone()))?;
                    self.send_packet(CreateCharOk::new()?).await
                }
                Err(DbErr::RecordNotInserted) => {
                    self.send_packet(CreateCharFailed::new(
                        CharNameResponseVariant::AlreadyExists,
                    )?)
                    .await
                }
                e => {
                    error!(?e, "Failed to create char");
                    self.send_packet(CreateCharFailed::new(
                        CharNameResponseVariant::CharCreationFailed,
                    )?)
                    .await
                }
            }
        } else {
            self.send_packet(CreateCharFailed::new(response)?).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::GameController;
    use crate::test_utils::test::{spawn_custom_player_client_actor, spawn_player_client_actor};
    use entities::test_factories::factories::user_factory;
    use l2_core::config::gs::GSServerConfig;
    use l2_core::traits::ServerConfig;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::split;

    fn build_packet() -> CreateCharRequest {
        let mut data = BytesMut::new();
        data.extend_from_slice(&[116, 0, 101, 0, 115, 0, 116, 0, 0, 0]);
        data.extend_from_slice(&i32::to_le_bytes(1));
        data.extend_from_slice(&i32::to_le_bytes(1));
        data.extend_from_slice(&i32::to_le_bytes(1));
        data.extend_from_slice(&i32::to_le_bytes(1));
        data.extend_from_slice(&i32::to_le_bytes(1));
        data.extend_from_slice(&i32::to_le_bytes(1));
        data.extend_from_slice(&i32::to_le_bytes(1));
        data.extend_from_slice(&i32::to_le_bytes(1));
        data.extend_from_slice(&i32::to_le_bytes(1));
        data.extend_from_slice(&i32::to_le_bytes(1));
        data.extend_from_slice(&i32::to_le_bytes(1));
        data.extend_from_slice(&i32::to_le_bytes(1));
        CreateCharRequest::read(data).unwrap()
    }
    #[tokio::test]
    async fn test_read_and_handle_fail_no_auth() {
        let pool = get_test_db().await;
        let pack = build_packet();
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = Arc::new(GSServerConfig::from_string(include_str!(
            "../../../../config/game.yaml"
        )));
        let controller = Arc::new(GameController::from_config(cfg).await);
        controller.add_online_account("test", None);
        let player_actor = spawn_player_client_actor(controller, pool, r, w).await;
        let res = player_actor.ask(pack).await;
        assert!(res.is_err());
    }
    #[tokio::test]
    async fn test_read_and_handle_fail_no_user() {
        let pool = get_test_db().await;
        let pack = build_packet();
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = Arc::new(GSServerConfig::from_string(include_str!(
            "../../../../config/game.yaml"
        )));
        let controller = Arc::new(GameController::from_config(cfg).await);
        controller.add_online_account("test", None);
        let mut player_client =
            PlayerClient::new(Ipv4Addr::LOCALHOST, controller.clone(), pool.clone());
        player_client.set_status(ClientStatus::Authenticated);
        let player_actor =
            spawn_custom_player_client_actor(controller, pool, r, w, Some(player_client)).await;
        let res = player_actor.ask(pack).await;
        assert!(res.is_err());
    }
    #[tokio::test]
    async fn test_read_and_handle_fail_no_chars() {
        let pool = get_test_db().await;
        let pack = build_packet();
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = Arc::new(GSServerConfig::from_string(include_str!(
            "../../../../config/game.yaml"
        )));
        let controller = Arc::new(GameController::from_config(cfg).await);
        controller.add_online_account("test", None);
        let mut player_client =
            PlayerClient::new(Ipv4Addr::LOCALHOST, controller.clone(), pool.clone());
        player_client.set_status(ClientStatus::Authenticated);

        let user = user_factory(&pool, |mut u| {
            u.username = String::from("test");
            u
        })
        .await;
        player_client.set_user(user);
        let player_actor =
            spawn_custom_player_client_actor(controller, pool, r, w, Some(player_client)).await;
        let res = player_actor.ask(pack).await;
        assert!(res.is_err());
    }
    #[tokio::test]
    async fn test_read_and_handle_ok() {
        let pool = get_test_db().await;
        let pack = build_packet();
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = Arc::new(GSServerConfig::from_string(include_str!(
            "../../../../config/game.yaml"
        )));
        let controller = Arc::new(GameController::from_config(cfg).await);
        controller.add_online_account("test", None);
        let mut player_client =
            PlayerClient::new(Ipv4Addr::LOCALHOST, controller.clone(), pool.clone());
        player_client.set_status(ClientStatus::Authenticated);
        let user = user_factory(&pool, |mut u| {
            u.username = String::from("test");
            u
        })
        .await;
        player_client.set_user(user);
        player_client.set_account_chars(vec![]);
        let player_actor =
            spawn_custom_player_client_actor(controller, pool.clone(), r, w, Some(player_client))
                .await;
        let res = player_actor.ask(pack.clone()).await;
        assert!(res.is_ok());
        let chars = character::Model::find_by_username(&pool, "test")
            .await
            .expect("Char must be created");
        assert_eq!(chars.len(), 1);
        let res = player_actor.ask(pack).await;
        assert!(res.is_ok());
        let chars = character::Model::find_by_username(&pool, "test")
            .await
            .expect("Char must be created");
        assert_eq!(chars.len(), 1);
    }
}
