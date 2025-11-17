use crate::dto::player::GSCharsInfo;
use crate::login_client::LoginClient;
use l2_core::shared_packets::common::{LoginServerOpcodes, ServerData, ServerStatus};
use l2_core::shared_packets::write::SendablePacketBuffer;
use std::collections::HashMap;
use macro_common::SendablePacket;

#[derive(Debug, Clone, SendablePacket)]
pub struct ServerList {
    pub buffer: SendablePacketBuffer,
    servers: Vec<ServerData>,
    last_server: i32,
    chars_on_server: Option<HashMap<u8, GSCharsInfo>>,
}

impl ServerList {
    pub fn new(login_client: &mut LoginClient, username: &str) -> ServerList {
        let servers = login_client.controller.get_server_list(login_client.addr);
        let player_option = login_client.controller.get_player(username);
        let mut chars_on_server = None;
        if let Some(player) = player_option {
            chars_on_server = Some(player.chars_on_servers);
        }
        let mut sl = Self {
            buffer: SendablePacketBuffer::new(),
            servers,
            last_server: 0, //todo: implement me
            chars_on_server,
        };
        let _ = sl.write_all();
        sl
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        {
            self.buffer.write(LoginServerOpcodes::ServerList as u8)?;
            self.buffer.write(self.servers.len() as u8)?;
            self.buffer.write(self.last_server as u8)?;
        }
        for server in &self.servers {
            self.buffer.write(server.server_id as u8)?;
            let ip_octets = server.get_ip_octets();
            #[allow(clippy::cast_possible_wrap)]
            {
                self.buffer.write(ip_octets[0])?;
                self.buffer.write(ip_octets[1])?;
                self.buffer.write(ip_octets[2])?;
                self.buffer.write(ip_octets[3])?;
            }

            self.buffer.write_i32(server.port)?;
            self.buffer.write(server.age_limit as u8)?; // Age Limit 0, 15, 18
            if server.pvp {
                self.buffer.write(0x01)?;
            } else {
                self.buffer.write(0x00)?;
            }
            self.buffer.write_i16(server.current_players as i16)?;
            self.buffer.write_i16(server.max_players as i16)?;
            if let Some(status) = server.status {
                self.buffer
                    .write_bool(!matches!(status, ServerStatus::Down))?;
            } else {
                self.buffer.write_bool(false)?;
            }
            self.buffer.write_i32(1024)?; // 1: Normal, 2: Relax, 4: Public Test, 8: No Label, 16: Character Creation Restricted, 32: Event, 64: Free
            self.buffer.write_bool(server.brackets)?;
        }
        self.buffer.write_i16(0xA4i16)?; //unknown
        if let Some(ref servers) = self.chars_on_server {
            for (server_id, info) in servers {
                self.buffer.write(*server_id)?;
                self.buffer.write(info.total_chars)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::controller::LoginController;
    use crate::dto::game_server::GSInfo;
    use crate::login_client::LoginClient;
    use crate::packet::to_client::ServerList;
    use l2_core::config::login::LoginServerConfig;
    use l2_core::traits::ServerConfig;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::{get_test_db, test_hex_id};

    #[tokio::test]
    async fn test_play_ok() {
        let db_pool = get_test_db().await;
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        lc.register_gs(
            GSInfo::new(
                1,
                true,
                false,
                9106,
                true,
                1,
                false,
                1,
                0,
                false,
                5000,
                test_hex_id(),
                &["192.168.0.100/8".to_string(), "192.168.0.0".to_string()],
            )
            .unwrap(),
        )
        .unwrap();
        let ip = Ipv4Addr::LOCALHOST;
        let mut login_client = LoginClient::new(ip, lc, db_pool);
        let packet = ServerList::new(&mut login_client, "admin");
        assert_eq!(
            [
                28, 0, 4, 1, 0, 1, 127, 0, 0, 1, 146, 35, 0, 0, 0, 0, 0, 0, 136, 19, 1, 0, 4, 0, 0,
                0, 164, 0
            ],
            packet.buffer.take().as_ref()
        );
    }
}
