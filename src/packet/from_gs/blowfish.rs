use crate::login_server::gs_handler::{GSConnectionState, GSHandler};
use crate::packet::common::read::ReadablePacketBuffer;
use crate::packet::common::GSHandle;
use crate::packet::common::{ReadablePacket, SendablePacket};
use crate::packet::error::PacketRun;
use crate::packet::login_fail::PlayerLogin;
use crate::packet::PlayerLoginFailReasons;
use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct BlowFish {
    pub encrypted_key: Vec<u8>,
}

impl ReadablePacket for BlowFish {
    #[allow(clippy::cast_sign_loss)]
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        let size = buffer.read_i32();
        Some(BlowFish {
            encrypted_key: buffer.read_bytes(size as usize),
        })
    }
}

#[async_trait]
impl GSHandle for BlowFish {
    async fn handle(&self, gs: &mut GSHandler) -> Result<Option<Box<dyn SendablePacket>>, PacketRun> {
        let mut key = self.encrypted_key.clone();
        if let Ok(mut decrypted) = gs.decrypt_rsa(&mut key) {
            // there are nulls before the key we must remove them
            if let Some(index) = decrypted.iter().position(|&x| x != 0) {
                decrypted.drain(..index);
            }
            gs.connection_state.transition_to(&GSConnectionState::BfConnected)?;
            gs.set_blowfish_key(&decrypted);
        } else {
            return Err(PacketRun {
                msg: Some("Unable to decrypt GS blowfish key".to_string()),
                response: Some(Box::new(PlayerLogin::new(PlayerLoginFailReasons::ReasonNotAuthed))),
            });
        }
        Ok(None)
    }
}
