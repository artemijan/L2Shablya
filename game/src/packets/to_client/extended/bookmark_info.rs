use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct BookmarkInfo {
    pub(crate) buffer: SendablePacketBuffer
}

impl BookmarkInfo {
    const PACKET_ID: u8 = 0xFE;
    const EX_PACKET_ID: u16 = 0x85;

    pub fn new(player: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(Self::EX_PACKET_ID)?;
        inst.buffer.write_i32(0)?;
        inst.buffer.write_i32(player.get_bookmark_slot())?;
        let bookmarks = player.get_teleport_bookmarks();
        inst.buffer.write_i32(i32::try_from(bookmarks.len())?)?;
        for bookmark in bookmarks {
            inst.buffer.write_i32(bookmark.id)?;
            inst.buffer.write_i32(bookmark.x)?;
            inst.buffer.write_i32(bookmark.y)?;
            inst.buffer.write_i32(bookmark.z)?;
            inst.buffer.write_sized_c_utf16le_string(Some(&bookmark.name))?;
            inst.buffer.write_i32(bookmark.icon)?;
            inst.buffer.write_sized_c_utf16le_string(Some(&bookmark.tag))?;
        }
        Ok(inst)
    }
}
