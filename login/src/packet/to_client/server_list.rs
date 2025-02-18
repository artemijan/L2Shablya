use crate::client_thread::ClientHandler;
use crate::dto::player::GSCharsInfo;
use l2_core::shared_packets::common::{LoginServerOpcodes, ServerData, ServerStatus};
use l2_core::shared_packets::write::SendablePacketBuffer;
use l2_core::traits::handlers::PacketHandler;
use macro_common::SendablePacketImpl;
use std::collections::HashMap;

#[derive(Debug, Clone, SendablePacketImpl)]
pub struct ServerList {
    pub buffer: SendablePacketBuffer,
    servers: Vec<ServerData>,
    last_server: i32,
    chars_on_server: Option<HashMap<u8, GSCharsInfo>>,
}

impl ServerList {
    pub fn new(ch: &ClientHandler, username: &str) -> ServerList {
        let lc = ch.get_controller();
        let servers = lc.get_server_list(ch.ip);
        let player_option = lc.get_player(username);
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
        self.buffer.write_i16(0xA4)?; //unknown
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
    use crate::client_thread::ClientHandler;
    use crate::controller::LoginController;
    use crate::dto::game_server::GSInfo;
    use crate::packet::to_client::ServerList;
    use l2_core::config::login::LoginServer;
    use l2_core::shared_packets::common::SendablePacket;
    use l2_core::traits::handlers::PacketHandler;
    use l2_core::traits::ServerConfig;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::{get_test_db, test_hex_id};
    use tokio::io::split;

    #[tokio::test]
    async fn test_play_ok() {
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServer::from_string(include_str!("../../../../config/login.yaml"));
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
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let ch = ClientHandler::new(r, w, ip, db_pool, cloned_lc);
        let mut packet = ServerList::new(&ch, "admin");
        assert_eq!(
            [
                28, 0, 4, 1, 0, 1, 127, 0, 0, 1, 146, 35, 0, 0, 0, 0, 0, 0, 136, 19, 1, 0, 4, 0, 0,
                0, 164, 0
            ],
            packet.get_bytes(false)
        );
    }
}
