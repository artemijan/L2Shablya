use crate::database::user::User;
use crate::login_server::gs_thread::GSHandler;
use crate::common::traits::handlers::PacketHandler;
use crate::common::packet::read::ReadablePacketBuffer;
use crate::login_server::packet::common::GSHandle;
use crate::common::packet::error;
use async_trait::async_trait;
use crate::common::packet::{ReadablePacket, SendablePacket};

#[derive(Clone, Debug)]
pub struct ChangeAL {
    pub account: String,
    pub level: i32,
}

impl ReadablePacket for ChangeAL {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        Some(Self {
            level: buffer.read_i32(),
            account: buffer.read_string(),
        })
    }
}
#[async_trait]
impl GSHandle for ChangeAL {
    async fn handle(
        &self,
        gs: &mut GSHandler,
    ) -> Result<Option<Box<dyn SendablePacket>>, error::PacketRun> {
        let db_pool = gs.get_db_pool_mut();
        //ignore error updating an account
        match User::change_access_level(db_pool, self.level, &self.account).await {
            Ok(user) => {
                println!("[change access level] OK {:?}", user.id);
            }
            Err(e) => {
                println!("[change access level] err {e:?}");
            }
        };
        Ok(None)
    }
}
