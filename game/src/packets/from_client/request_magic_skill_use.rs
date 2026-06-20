use crate::packets::to_client;
use crate::pl_client::{ApplyDamage, GetStats, PlayerClient};
use bytes::BytesMut;
use kameo::message::{Context, Message};
use l2_core::game_objects::stats::Formulas;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::shared_packets::write::SendablePacketBuffer;
use tracing::{info, instrument};

#[derive(Debug, Clone)]
pub struct RequestMagicSkillUse {
    pub buffer: SendablePacketBuffer,
    pub skill_id: i32,
    pub ctrl_pressed: bool,
    pub shift_pressed: bool,
}

impl ReadablePacket for RequestMagicSkillUse {
    const PACKET_ID: u8 = 0x39;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        let skill_id = buffer.read_i32()?;
        let ctrl_pressed = buffer.read_i32()? != 0;
        let shift_pressed = buffer.read_byte()? != 0;
        Ok(Self {
            buffer: SendablePacketBuffer::empty(),
            skill_id,
            ctrl_pressed,
            shift_pressed,
        })
    }
}

impl Message<RequestMagicSkillUse> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: RequestMagicSkillUse,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        info!("Handling RequestMagicSkillUse: skill_id={}, ctrl={}, shift={}", msg.skill_id, msg.ctrl_pressed, msg.shift_pressed);
        
        let (attacker_id, attacker_name, attacker_stats, (x, y, z), level) = {
            let player = self.try_get_selected_char()?;
            let level = player.get_skill_level(msg.skill_id).unwrap_or(1);

            if player.is_skill_disabled(msg.skill_id) {
                info!("Skill {} is on cooldown", msg.skill_id);
                let mut sm = to_client::SystemMessage::new(to_client::SystemMessageType::S1IsNotAvailableAtThisTimeBeingPreparedForReuse)?;
                sm.add_param(to_client::SystemMessageParam::SkillName { id: msg.skill_id, level: level as i16, sub_level: 0 })?;
                self.send_packet(sm).await?;
                return Ok(());
            }

            (
                player.get_object_id(),
                player.get_visible_name().to_string(),
                player.stats.cached_stats.clone(),
                (player.get_x(), player.get_y(), player.get_z()),
                level,
            )
        };

        if let Some((target_id, target_actor)) = self.selected_target.clone() {
            let target_stats = target_actor.ask(GetStats).await?;
            let (target_x, target_y, target_z) = (target_stats.x, target_stats.y, target_stats.z);

            let level_u8 = level as u8;
            let skill_id_u32 = msg.skill_id as u32;
            let skill_data = self.controller.skills.get_skill(skill_id_u32, level_u8);

            let mut hit_time = 1000;
            let mut reuse_delay = 0;
            let mut reuse_group = -1;
            let skill_power = 50.0;
            if let Some(skill) = skill_data {
                if let Some(ht) = skill.hit_time.as_ref().and_then(|v| v.text) {
                    let m_atk_spd = attacker_stats.get(&l2_core::game_objects::stats::stat_enum::Stat::MAtkSpd).cloned().unwrap_or(333.0);
                    hit_time = (ht as f64 * 333.0 / m_atk_spd) as i32;
                }
                
                if let Some(rd) = skill.reuse_delay.as_ref().and_then(|v| v.text) {
                    reuse_delay = rd as i64;
                }

                if let Some(rg) = skill.reuse_delay_group.as_ref().and_then(|v| v.text) {
                    reuse_group = rg;
                }
            }

            let client_reuse_group = if reuse_group > 0 { reuse_group } else { msg.skill_id };

            {
                let player = self.try_get_selected_char_mut()?;
                player.add_skill_reuse(msg.skill_id, level as i32, reuse_delay, reuse_group);
            }
            info!("Skill reuse added: skill_id={}, reuse_delay={}, reuse_group={}, client_group={}", msg.skill_id, reuse_delay, reuse_group, client_reuse_group);
            let magic_use_packet = to_client::MagicSkillUse::new(
                attacker_id,
                target_id,
                msg.skill_id,
                level as i32,
                hit_time,
                reuse_delay as i32, // reuse
                client_reuse_group,
                x,
                y,
                z,
                target_x,
                target_y,
                target_z,
            )?;
            println!("Reuse: {reuse_delay}");
            self.controller.broadcast_packet(magic_use_packet);

            // Calculate damage and schedule hit
            let damage = Formulas::calc_magic_dam(&attacker_stats, &target_stats.stats, skill_power, false, false);
            let controller = self.controller.clone();

            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(hit_time as u64)).await;

                let magic_launched_packet = to_client::MagicSkillLaunched::new(
                    attacker_id,
                    msg.skill_id,
                    level as i32,
                    0, // casting_type
                    &[target_id],
                ).unwrap();

                controller.broadcast_packet(magic_launched_packet);

                // Apply damage only when hit
                let _ = target_actor.tell(ApplyDamage {
                    damage,
                    attacker_id,
                    attacker_name,
                }).await;
            });
        }

        Ok(())
    }
}
