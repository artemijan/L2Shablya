use crate::gs_thread::{enums, GSHandler};
use crate::packet::HandleablePacket;
use anyhow::bail;
use async_trait::async_trait;
use l2_core::shared_packets::{
    common::{PlayerLoginFail, PlayerLoginFailReasons},
    gs_2_ls::BlowFish,
};
use l2_core::traits::handlers::PacketSender;

#[async_trait]
impl HandleablePacket for BlowFish {
    type HandlerType = GSHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> anyhow::Result<()> {
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
            bail!("Unable to decrypt GS blowfish key");
        }
        Ok(())
    }
}
