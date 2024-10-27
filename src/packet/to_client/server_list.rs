use std::collections::HashMap;
use crate::common::dto::player::GSCharsInfo;
use crate::packet::common::write::SendablePacketBuffer;
use crate::packet::common::{SendablePacket, ServerData, ServerStatus};
use crate::packet::LoginServerOpcodes;

#[derive(Debug, Clone)]
pub struct ServerList {
    pub buffer: SendablePacketBuffer,
    servers: Vec<ServerData>,
    last_server: i32,
    chars_on_server: Option<HashMap<u8, GSCharsInfo>>,
}

impl ServerList {
    pub fn new(servers: Vec<ServerData>, last_server: i32, chars_on_server: Option<HashMap<u8, GSCharsInfo>>) -> ServerList {
        let mut sl = ServerList {
            buffer: SendablePacketBuffer::new(),
            servers,
            last_server,
            chars_on_server,
        };
        sl.write_all().unwrap();
        sl
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        #[allow(clippy::cast_possible_truncation)]
        {
            self.buffer.write(LoginServerOpcodes::ServerList as u8)?;
            self.buffer.write(self.servers.len() as u8)?;
            self.buffer.write(self.last_server as u8)?;
        }
        for server in &self.servers {
            #[allow(clippy::cast_possible_truncation)]
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
            #[allow(clippy::cast_possible_truncation)]
            self.buffer.write(server.age_limit as u8)?; // Age Limit 0, 15, 18
            if server.pvp {
                self.buffer.write(0x01)?;
            } else {
                self.buffer.write(0x00)?;
            }
            #[allow(clippy::cast_possible_truncation)]
            self.buffer.write_i16(server.current_players as i16)?;
            #[allow(clippy::cast_possible_truncation)]
            self.buffer.write_i16(server.max_players as i16)?;
            if let Some(status) = server.status {
                self.buffer.write_i8_from_bool(!matches!(status, ServerStatus::Down))?;
            } else {
                self.buffer.write_i8_from_bool(false)?;
            }
            self.buffer.write_i32(1024)?; // 1: Normal, 2: Relax, 4: Public Test, 8: No Label, 16: Character Creation Restricted, 32: Event, 64: Free
            self.buffer.write_i8_from_bool(server.brackets)?;
        }
        self.buffer.write_i16(0xA4)?; //unknown
        if let Some(ref servers) = self.chars_on_server {
            for (server_id, info) in servers {
                #[allow(clippy::cast_possible_truncation)]
                self.buffer.write(*server_id)?;
                // todo: here should be real count of chars on server
                #[allow(clippy::cast_possible_truncation)]
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
