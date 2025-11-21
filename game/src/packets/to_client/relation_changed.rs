use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;
#[derive(Debug, Clone)]
pub struct Relation {
    pub obj_id: i32,
    pub rel: u32,
    pub auto_attackable: bool,
    pub reputation: u32,
    pub pvp_flag: bool,
}

#[derive(Debug, Clone, SendablePacket)]
pub struct RelationChanged {
    pub buffer: SendablePacketBuffer,
    pub multi: Option<Vec<Relation>>,
    pub mask: u8,
}

impl RelationChanged {
    const PACKET_ID: u8 = 0xCE;
    const EX_PACKET_ID: Option<u16> = None;
    const SEND_DEFAULT: u8 = 0x01;
    const SEND_ONE: u8 = 0x02;
    const SEND_MULTI: u8 = 0x04;
    pub fn builder() -> RelationChangedBuilder {
        RelationChangedBuilder::new()
    }
}
#[derive(Debug, Clone)]
pub struct RelationChangedBuilder {
    multi: Option<Vec<Relation>>,
    mask: u8,
    signed: Option<Relation>,
}
impl RelationChangedBuilder {
    pub fn new() -> Self {
        Self {
            multi: None,
            signed: None,
            mask: RelationChanged::SEND_MULTI,
        }
    }

    pub fn add_relation(mut self, player: &Player, relation: u32, auto_attackable: bool) -> Self {
        if !player.is_invisible() {
            let relation = Relation {
                obj_id: player.char_model.id,
                rel: relation,
                auto_attackable,
                reputation: player.char_model.reputation,
                pvp_flag: player.get_pvp_flag(),
            };
            if let Some(multi) = &mut self.multi {
                multi.push(relation);
            } else {
                self.multi = Some(vec![relation]);

            }
        }
        self
    }
    pub fn finish(self) -> anyhow::Result<RelationChanged> {
        let mut buffer = SendablePacketBuffer::new();
        buffer.write_u8(RelationChanged::PACKET_ID)?;
        buffer.write(self.mask)?;
        if let Some(ref multi) = self.multi {
            buffer.write_u16(u16::try_from(multi.len())?)?;
            for r in multi {
                self.write_relation(r, &mut buffer)?;
            }
        } else if let Some(ref singled) = self.signed {
            self.write_relation(singled, &mut buffer)?;
        } else {
            return Err(anyhow::anyhow!("No relations to send"));
        }
        Ok(RelationChanged {
            buffer,
            multi: self.multi,
            mask: self.mask,
        })
    }
    fn write_relation(
        &self,
        r: &Relation,
        buffer: &mut SendablePacketBuffer,
    ) -> anyhow::Result<()> {
        buffer.write_i32(r.obj_id)?;
        if (self.mask & RelationChanged::SEND_DEFAULT) != RelationChanged::SEND_DEFAULT {
            buffer.write_u32(r.rel)?;
            buffer.write(r.auto_attackable)?;
            buffer.write_u32(r.reputation)?;
            buffer.write(r.pvp_flag)?;
        }
        Ok(())
    }
}
