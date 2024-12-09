use crate::common::packets::error::PacketRun;
use crate::common::packets::gs_2_ls::BlowFish;
use crate::common::traits::handlers::PacketHandler;
use crate::crypt::generate_blowfish_key;
use crate::crypt::login::Encryption;
use crate::crypt::rsa::RSAPublicKey;
use crate::{
    common::packets::{common::HandleablePacket, gs_2_ls::RequestAuthGS, ls_2_gs::InitLS},
    game_server::handlers::LoginHandler,
};
use async_trait::async_trait;
use num_traits::ToBytes;

#[async_trait]
impl HandleablePacket for InitLS {
    type HandlerType = LoginHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let controller = gs.get_controller();
        let config = controller.get_cfg();
        let ra = RequestAuthGS::builder()
            .desired_id(config.server_id)
            .accept_alternative_id(config.accept_alternative_id)
            .host_reserved(config.host_reserved)
            .port(config.listeners.clients.connection.port)
            .max_players(config.max_players)
            .hex_id(config.hex_id.to_be_bytes())
            .hosts(config.get_hosts())
            .build()?;
        let p_key = RSAPublicKey::from_modulus(&self.public_key)?;
        let new_key = generate_blowfish_key(Some(40));
        let encrypted_data = p_key.encrypt(&new_key).expect("Can not encrypt rsa key");
        let bf = BlowFish::new(encrypted_data);
        gs.send_packet(Box::new(bf)).await?;
        gs.set_blowfish(Encryption::new(&new_key));
        gs.send_packet(Box::new(ra)).await?;
        Ok(())
    }
}
