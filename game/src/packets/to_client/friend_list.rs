use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct FriendList {
    pub(crate) buffer: SendablePacketBuffer,
}

impl FriendList {
    const PACKET_ID: u8 = 0x75;

    pub fn new(_player: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        let friends:Vec<u8> = vec![];
        inst.buffer.write_u32(friends.len() as u32)?;
        //todo: implement me
        Ok(inst)
    }
}
