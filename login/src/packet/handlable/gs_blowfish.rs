use l2_core::traits::handlers::PacketSender;
use async_trait::async_trait;
use l2_core::shared_packets::{
    common::{PlayerLoginFail, PlayerLoginFailReasons},
    error::PacketRun,
    gs_2_ls::BlowFish,
};
use crate::{
    gs_thread::{enums, GSHandler},
};
use crate::packet::HandleablePacket;

#[async_trait]
impl HandleablePacket for BlowFish {
    type HandlerType = GSHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let mut key = self.encrypted_key.clone();
        if let Ok(mut decrypted) = gs.decrypt_rsa(&mut key) {
            // there are nulls before the key we must remove them
            if let Some(index) = decrypted.iter().position(|&x| x != 0) {
                decrypted.drain(..index);
            }
            gs.set_connection_state(&enums::GS::BfConnected).await?;
            gs.set_blowfish_key(&decrypted);
        } else {
            gs.send_packet(Box::new(PlayerLoginFail::new(
                PlayerLoginFailReasons::ReasonNotAuthed,
            )))
            .await?;
            return Err(PacketRun {
                msg: Some("Unable to decrypt GS blowfish key".to_string()),
            });
        }
        Ok(())
    }
}
