use crate::common::packets::common::{
    LoginServerOpcodes, SendablePacket, ServerData, ServerStatus,
};
use crate::common::packets::write::SendablePacketBuffer;
use crate::common::traits::handlers::PacketHandler;
use crate::login_server::client_thread::ClientHandler;
use crate::login_server::dto::player::GSCharsInfo;
use std::collections::HashMap;

#[derive(Debug, Clone)]
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
        let mut player_option = lc.get_player(username);
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
                // todo: here should be real count of chars on server
                self.buffer.write(info.chars)?;
            }
        }
        Ok(())
    }
}

impl SendablePacket for ServerList {
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}
