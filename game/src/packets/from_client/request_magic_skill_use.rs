use crate::movement::{calculate_distance, calculate_nearest_hit_point};
use crate::packets::to_client;
use crate::packets::to_client::ActionFailed;
use crate::pl_client::{ApplyDamage, FullStats, GetStats, PlayerClient, PlayerTasks};
use bytes::BytesMut;
use kameo::message::{Context, Message};
use l2_core::data::skills::TargetType;
use l2_core::errors::KameoAnyhowExt;
use l2_core::game_objects::stats::Formulas;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::shared_packets::write::SendablePacketBuffer;
use tracing::{error, instrument};

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
        if self.is_casting() {
            self.send_packet(ActionFailed::normal()?).await?;
            return Ok(());
        }
        self.stop_movement();
        let player = self.try_get_selected_char()?;
        let (attacker_id, attacker_name, attacker_stats, (x, y, z), level) = {
            let level = player.get_skill_level(msg.skill_id).unwrap_or(1);
            if player.is_skill_disabled(msg.skill_id) {
                let mut sm = to_client::SystemMessage::new(
                    to_client::SystemMessageType::S1IsNotAvailableAtThisTimeBeingPreparedForReuse,
                )?;
                sm.add_param(to_client::SystemMessageParam::SkillName {
                    id: msg.skill_id,
                    level,
                    sub_level: 0,
                })?;
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
        let skill_data = self
            .controller
            .skills
            .get_skill(msg.skill_id as u32, level as u8)
            .expect("Skill doesnt exist");
        let selected_target = {
            match (&skill_data.target_type, &self.selected_target) {
                (Some(tt), Some(val)) if *tt == TargetType::SELF => {
                    Some((player.get_object_id(), _ctx.actor_ref().clone()))
                }
                (_, Some(target)) => Some(target.clone()),
                _ => None,
            }
        };
        if let Some((target_id, target_actor)) = selected_target {
            let target_stats = if target_id == player.get_object_id() {
                FullStats {
                    stats: player.stats.cached_stats.clone(),
                    x: player.get_x(),
                    y: player.get_y(),
                    z: player.get_z(),
                }
            } else {
                target_actor.ask(GetStats).await.anyhow()?
            };
            let (target_x, target_y, target_z) = (target_stats.x, target_stats.y, target_stats.z);

            let level_u8 = level as u8;
            let skill_id_u32 = msg.skill_id as u32;

            let dist = calculate_distance(x, y, z, target_x, target_y, target_z).unwrap_or(0.0);

            if !self.check_visibility(target_x, target_y, target_z).await? {
                return Ok(());
            }

            let skill_data = self.controller.skills.get_skill(skill_id_u32, level_u8);

            let mut cast_range = 40;
            if let Some(skill) = skill_data {
                if let Some(cr) = skill.cast_range.as_ref().and_then(|v| v.text) {
                    cast_range = cr;
                }
            }

            if dist > (cast_range + 40) as f64 {
                // Start movement towards the target
                let pl_actor = _ctx.actor_ref().clone();
                let (hit_x, hit_y, hit_z) = calculate_nearest_hit_point(
                    (x, y, z),
                    (target_x, target_y, target_z),
                    dist,
                    cast_range,
                );
                self.start_movement(hit_x, hit_y, hit_z, pl_actor)?;
                let self_actor = _ctx.actor_ref().clone();
                self.schedule_triggered_task(PlayerTasks::ActionIntent, async move {
                    // Arrived, try to hit again
                    let _ = self_actor.tell(msg).await;
                });
                return Ok(());
            }

            let mut hit_time = 1000;
            let mut reuse_delay = 0;
            let mut reuse_group = -1;
            let skill_power = 50.0;
            if let Some(skill) = skill_data {
                if let Some(ht) = skill.hit_time.as_ref().and_then(|v| v.text) {
                    let m_atk_spd = attacker_stats
                        .get(&l2_core::game_objects::stats::stat_enum::Stat::MAtkSpd)
                        .cloned()
                        .unwrap_or(333.0);
                    hit_time = (ht as f64 * 333.0 / m_atk_spd) as i32;
                }

                if let Some(rd) = skill.reuse_delay.as_ref().and_then(|v| v.text) {
                    reuse_delay = rd as i64;
                }

                if let Some(rg) = skill.reuse_delay_group.as_ref().and_then(|v| v.text) {
                    reuse_group = rg;
                }
            }

            let client_reuse_group = if reuse_group > 0 {
                reuse_group
            } else {
                msg.skill_id
            };

            {
                let player = self.try_get_selected_char_mut()?;
                player.add_skill_reuse(msg.skill_id, level as i32, reuse_delay, reuse_group);
            }
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
            self.controller.broadcast_packet(magic_use_packet);
            let magic_launched_packet = to_client::MagicSkillLaunched::new(
                attacker_id,
                msg.skill_id,
                level as i32,
                0, // casting_type
                &[target_id],
            )?;
            let controller = self.controller.clone();
            let magic_skill_use_task = tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(hit_time as u64)).await;
                controller.broadcast_packet(magic_launched_packet);
                let damage = Formulas::calc_magic_dam(
                    &attacker_stats,
                    &target_stats.stats,
                    skill_power,
                    false,
                    false,
                );
                if let Err(err) = target_actor
                    .tell(ApplyDamage {
                        damage,
                        attacker_id,
                        attacker_name,
                    })
                    .await
                    .anyhow()
                {
                    error!("Failed to apply damage from {attacker_id} to {target_id}: {err}");
                }
            });
            self.schedule_task(PlayerTasks::CauseDamage, magic_skill_use_task);
        }

        Ok(())
    }
}
