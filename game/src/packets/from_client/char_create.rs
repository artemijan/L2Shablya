use crate::client_thread::{ClientHandler, ClientStatus};
use crate::data::char_template::CharTemplate;
use crate::data::classes::mapping::Class;
use crate::packets::enums::CharNameResponseVariant;
use crate::packets::to_client::{CreateCharFailed, CreateCharOk};
use crate::packets::utils::validate_can_create_char;
use crate::packets::HandleablePacket;
use anyhow::bail;
use async_trait::async_trait;
use entities::dao::char_info::CharacterInfo;
use entities::entities::character;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::traits::handlers::{PacketHandler, PacketSender};
use sea_orm::DbErr;
use std::sync::Arc;
use tracing::error;

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
    fn read(data: &[u8]) -> anyhow::Result<Self> {
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

#[async_trait]
impl HandleablePacket for CreateCharRequest {
    type HandlerType = ClientHandler;
    async fn handle(&self, handler: &mut Self::HandlerType) -> anyhow::Result<()> {
        if &ClientStatus::Authenticated != handler.get_status() || handler.get_user().is_none() {
            bail!("Not authenticated.");
        }
        let controller = handler.get_controller().clone();
        let template = controller.class_templates.try_get_template(self.class_id)?;
        self.validate(template)?;
        let db_pool = handler.get_db_pool();
        let response = validate_can_create_char(db_pool, &self.name).await?;
        if response == CharNameResponseVariant::Ok {
            let mut char = character::Model {
                name: self.name.clone(),
                level: 1,
                face: self.face,
                hair_style: self.hair_style,
                hair_color: self.hair_color,
                is_female: self.is_female,
                delete_at: None,
                user_id: handler.try_get_user()?.id,
                ..Default::default()
            };
            template.initialize_character(&mut char, &controller.base_stats_table)?;
            match character::Model::create_char(db_pool, char).await {
                Ok(inst) => {
                    handler.add_character(CharacterInfo::new(inst, vec![])?)?;
                    handler.send_packet(Box::new(CreateCharOk::new()?)).await
                }
                Err(DbErr::RecordNotInserted) => {
                    handler
                        .send_packet(Box::new(CreateCharFailed::new(
                            CharNameResponseVariant::AlreadyExists,
                        )?))
                        .await
                }
                e => {
                    error!(?e, "Failed to create char");
                    handler
                        .send_packet(Box::new(CreateCharFailed::new(
                            CharNameResponseVariant::CharCreationFailed,
                        )?))
                        .await
                }
            }
        } else {
            handler
                .send_packet(Box::new(CreateCharFailed::new(response)?))
                .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::Controller;
    use entities::test_factories::factories::user_factory;
    use l2_core::config::gs::GSServer;
    use l2_core::traits::ServerConfig;
    use ntest::timeout;
    use std::net::Ipv4Addr;
    use test_utils::utils::get_test_db;
    use tokio::io::{split, AsyncWriteExt};
    fn build_packet() -> CreateCharRequest {
        let mut data = vec![];
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
        CreateCharRequest::read(&data).unwrap()
    }
    #[tokio::test]
    #[timeout(3000)]
    async fn test_read_and_handle_fail_no_auth() {
        let pool = get_test_db().await;
        let pack = build_packet();
        let (mut client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = Arc::new(GSServer::from_string(include_str!(
            "../../../../test_data/game.yaml"
        )));
        let controller = Arc::new(Controller::new(cfg));
        controller.add_online_account(String::from("test"));
        let mut ch = ClientHandler::new(r, w, Ipv4Addr::LOCALHOST, pool.clone(), controller);
        let res = pack.handle(&mut ch).await;
        assert!(res.is_err());
    }
    #[tokio::test]
    #[timeout(3000)]
    async fn test_read_and_handle_fail_no_user() {
        let pool = get_test_db().await;
        let pack = build_packet();
        let (mut client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = Arc::new(GSServer::from_string(include_str!(
            "../../../../test_data/game.yaml"
        )));
        let controller = Arc::new(Controller::new(cfg));
        controller.add_online_account(String::from("test"));
        let mut ch = ClientHandler::new(r, w, Ipv4Addr::LOCALHOST, pool.clone(), controller);
        ch.set_status(ClientStatus::Authenticated);
        let res = pack.handle(&mut ch).await;
        assert!(res.is_err());
    }
    #[tokio::test]
    #[timeout(3000)]
    async fn test_read_and_handle_fail_no_chars() {
        let pool = get_test_db().await;
        let pack = build_packet();
        let (mut client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = Arc::new(GSServer::from_string(include_str!(
            "../../../../test_data/game.yaml"
        )));
        let controller = Arc::new(Controller::new(cfg));
        controller.add_online_account(String::from("test"));
        let mut ch = ClientHandler::new(r, w, Ipv4Addr::LOCALHOST, pool.clone(), controller);
        ch.set_status(ClientStatus::Authenticated);
        let user = user_factory(&pool, |mut u| {
            u.username = String::from("test");
            u
        })
        .await;
        ch.set_user(user);
        let res = pack.handle(&mut ch).await;
        assert!(res.is_err());
    }
    #[tokio::test]
    #[timeout(3000)]
    async fn test_read_and_handle_ok() {
        let pool = get_test_db().await;
        let pack = build_packet();
        let (mut client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = Arc::new(GSServer::from_string(include_str!(
            "../../../../test_data/game.yaml"
        )));
        let controller = Arc::new(Controller::new(cfg));
        controller.add_online_account(String::from("test"));
        let mut ch = ClientHandler::new(r, w, Ipv4Addr::LOCALHOST, pool.clone(), controller);
        ch.set_status(ClientStatus::Authenticated);
        let user = user_factory(&pool, |mut u| {
            u.username = String::from("test");
            u
        })
        .await;
        ch.set_user(user);
        ch.set_account_chars(vec![]);
        let res = pack.handle(&mut ch).await;
        assert!(res.is_ok());
        assert_eq!(ch.get_account_chars().unwrap().len(), 1);
        let res = pack.handle(&mut ch).await;
        println!("{:?}", res.is_ok()); // there shouldn't be error
        assert_eq!(ch.get_account_chars().unwrap().len(), 1); // but char not created because it's duplicate
        client.shutdown().await.unwrap();
    }
}
