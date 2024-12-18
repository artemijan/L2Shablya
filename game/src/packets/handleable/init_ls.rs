use async_trait::async_trait;
use l2_core::crypt::generate_blowfish_key;
use l2_core::crypt::login::Encryption;
use l2_core::crypt::rsa::RSAPublicKey;
use l2_core::packets::error::PacketRun;
use l2_core::packets::gs_2_ls::BlowFish;
use l2_core::packets::{gs_2_ls::RequestAuthGS, ls_2_gs::InitLS};
use l2_core::traits::handlers::PacketHandler;
use crate::ls_thread::LoginHandler;
use crate::packets::HandleablePacket;

#[async_trait]
impl HandleablePacket for InitLS {
    type HandlerType = LoginHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let controller = gs.get_controller();
        let config = controller.get_cfg();
        let ra = RequestAuthGS::new(&config)?;
        let p_key = RSAPublicKey::from_modulus(&self.public_key)?;
        let new_key = generate_blowfish_key(Some(40));
        let encrypted_data = p_key.encrypt(&new_key)?;
        let bf = BlowFish::new(encrypted_data);
        gs.send_packet(Box::new(bf)).await?;
        gs.set_blowfish(Encryption::new(&new_key));
        gs.send_packet(Box::new(ra)).await?;
        Ok(())
    }
}
