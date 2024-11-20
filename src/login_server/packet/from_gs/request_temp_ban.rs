use crate::database::user::User;
use crate::login_server::gs_thread::GSHandler;
use crate::common::traits::handlers::PacketHandler;
use crate::common::packet::read::ReadablePacketBuffer;
use crate::login_server::packet::common::GSHandle;
use crate::common::packet::error;
use async_trait::async_trait;
use crate::common::packet::{ReadablePacket, SendablePacket};

#[derive(Clone, Debug)]
pub struct RequestTempBan {
    pub account: String,
    pub ban_duration: i64,
    pub ip: String,
}

impl ReadablePacket for RequestTempBan {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        Some(Self {
            account: buffer.read_string(),
            ip: buffer.read_string(),
            ban_duration: buffer.read_i64(),
        })
    }
}
#[async_trait]
impl GSHandle for RequestTempBan {
    async fn handle(
        &self,
        gs: &mut GSHandler,
    ) -> Result<Option<Box<dyn SendablePacket>>, error::PacketRun> {
        let db_pool = gs.get_db_pool_mut();
        //ignore error updating an account
        match User::req_temp_ban(db_pool, &self.account, self.ban_duration, &self.ip).await {
            Ok(user) => {
                println!("[Account banned] OK {:?}", user.id);
            }
            Err(e) => {
                println!("[Failed to ban account] err {e:?}");
            }
        };
        let lc = gs.get_controller();
        lc.update_ip_ban_list(&self.ip, self.ban_duration);
        lc.remove_player(&self.account);
        Ok(None)
    }
}
