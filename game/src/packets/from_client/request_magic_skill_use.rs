use crate::movement::{calculate_distance, calculate_nearest_hit_point};
use crate::packets::to_client;
use crate::packets::to_client::ActionFailed;
use crate::pl_client::{
    ApplyBuff, ApplyDamage, ApplyHeal, FullStats, GetStats, PlayerClient, PlayerTasks,
};
use crate::skills::{AffectedTarget, SkillAction, classify_effects, gather_affected_targets};
use bytes::BytesMut;
use kameo::actor::ActorRef;
use kameo::message::{Context, Message};
use l2_core::data::skills::TargetType;
use l2_core::errors::KameoAnyhowExt;
use l2_core::game_objects::stats::Formulas;
use l2_core::game_objects::stats::stat_enum::Stat;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::shared_packets::write::SendablePacketBuffer;
use std::sync::Arc;
use tracing::{error, instrument, warn};

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

impl PlayerClient {
    async fn send_invalid_target(&mut self) -> anyhow::Result<()> {
        self.send_packet(to_client::SystemMessage::new(
            to_client::SystemMessageType::InvalidTarget,
        )?)
        .await?;
        self.send_packet(ActionFailed::normal()?).await
    }
}

/// Applies the resolved skill actions on every affected target.
/// Runs inside the launch task, after the cast time has passed.
#[allow(clippy::too_many_arguments)]
async fn apply_skill_actions(
    actions: &[SkillAction],
    attacker_id: i32,
    attacker_name: &str,
    attacker_stats: &std::collections::HashMap<Stat, f64>,
    magic_crit_rate: f64,
    abnormal: (Option<String>, i32, i32), // (type, level, time secs)
    skill: (i32, i32),                    // (id, level)
    targets: &[AffectedTarget],
) {
    let (skill_id, skill_level) = skill;
    for target in targets {
        for action in actions {
            let result = match action {
                SkillAction::MagicDamage { power } => {
                    let mcrit = Formulas::calc_magic_crit(attacker_stats, magic_crit_rate);
                    let damage = Formulas::calc_magic_dam(
                        attacker_stats,
                        &target.stats.stats,
                        *power,
                        false,
                        false,
                        mcrit,
                    );
                    target
                        .actor
                        .tell(ApplyDamage {
                            damage,
                            attacker_id,
                            attacker_name: attacker_name.to_string(),
                        })
                        .await
                        .anyhow()
                }
                SkillAction::PhysDamage { power, crit_chance } => {
                    let crit = Formulas::calc_phys_skill_crit(*crit_chance);
                    let damage = Formulas::calc_phys_skill_dam(
                        attacker_stats,
                        &target.stats.stats,
                        *power,
                        false,
                        crit,
                    );
                    target
                        .actor
                        .tell(ApplyDamage {
                            damage,
                            attacker_id,
                            attacker_name: attacker_name.to_string(),
                        })
                        .await
                        .anyhow()
                }
                SkillAction::Heal { power } => {
                    let (amount, _crit) =
                        Formulas::calc_heal(attacker_stats, *power, magic_crit_rate, false, false);
                    target
                        .actor
                        .tell(ApplyHeal {
                            amount,
                            is_percent: false,
                            is_mp: false,
                            healer_id: attacker_id,
                            healer_name: attacker_name.to_string(),
                        })
                        .await
                        .anyhow()
                }
                SkillAction::HealPercent { percent } => target
                    .actor
                    .tell(ApplyHeal {
                        amount: *percent,
                        is_percent: true,
                        is_mp: false,
                        healer_id: attacker_id,
                        healer_name: attacker_name.to_string(),
                    })
                    .await
                    .anyhow(),
                SkillAction::ManaHeal { power } => target
                    .actor
                    .tell(ApplyHeal {
                        amount: *power,
                        is_percent: false,
                        is_mp: true,
                        healer_id: attacker_id,
                        healer_name: attacker_name.to_string(),
                    })
                    .await
                    .anyhow(),
                SkillAction::ManaHealPercent { percent } => target
                    .actor
                    .tell(ApplyHeal {
                        amount: *percent,
                        is_percent: true,
                        is_mp: true,
                        healer_id: attacker_id,
                        healer_name: attacker_name.to_string(),
                    })
                    .await
                    .anyhow(),
                SkillAction::Buff { mods } => target
                    .actor
                    .tell(ApplyBuff {
                        skill_id,
                        skill_level,
                        caster_id: attacker_id,
                        abnormal_type: abnormal.0.clone(),
                        abnormal_level: abnormal.1,
                        abnormal_time_secs: abnormal.2,
                        mods: mods.clone(),
                    })
                    .await
                    .anyhow(),
            };
            if let Err(err) = result {
                error!(
                    "Failed to apply skill {skill_id} effect from {attacker_id} to {}: {err}",
                    target.id
                );
            }
        }
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
        let (attacker_id, attacker_name, attacker_stats, (x, y, z), level, current_mp) = {
            let player = self.try_get_selected_char()?;
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
                player.stats.current_mp,
            )
        };
        let level_u8 = u8::try_from(level.max(1)).unwrap_or(1);
        let Some(skill_data) = self
            .controller
            .skills
            .get_skill(msg.skill_id as u32, level_u8)
            .cloned()
        else {
            warn!("Skill {} (level {level}) not found in data", msg.skill_id);
            self.send_packet(ActionFailed::normal()?).await?;
            return Ok(());
        };

        let target_type = skill_data.target_type_at(level_u8);
        let is_bad = skill_data.is_bad(level_u8);

        // --- Main target resolution (docs/skills/02-target-selection.md) ---
        let self_actor = _ctx.actor_ref().clone();
        let (target_id, target_actor): (i32, ActorRef<PlayerClient>) = match target_type {
            TargetType::SELF | TargetType::MY_PARTY | TargetType::NONE | TargetType::GROUND => {
                (attacker_id, self_actor.clone())
            }
            TargetType::ENEMY | TargetType::ENEMY_ONLY | TargetType::OTHERS => {
                match &self.selected_target {
                    Some((tid, actor)) if *tid != attacker_id => (*tid, actor.clone()),
                    _ => {
                        // You cannot attack yourself, and an offensive skill needs a target.
                        self.send_invalid_target().await?;
                        return Ok(());
                    }
                }
            }
            // TARGET/ENEMY_NOT and anything else: currently selected target, or yourself
            // for friendly skills when nothing is selected.
            _ => match &self.selected_target {
                Some((tid, actor)) => (*tid, actor.clone()),
                None if !is_bad => (attacker_id, self_actor.clone()),
                None => {
                    self.send_invalid_target().await?;
                    return Ok(());
                }
            },
        };

        // Force-attack rule: bad skills on a friendly (non flagged) player need Ctrl.
        if is_bad && target_id != attacker_id && !msg.ctrl_pressed {
            let auto_attackable = {
                let attacker = self.try_get_selected_char()?;
                if let Ok(target_player) = target_actor.ask(crate::pl_client::GetCharInfo).await {
                    target_player.is_auto_attackable(attacker)
                } else {
                    false
                }
            };
            // ENEMY allows force attack with Ctrl; ENEMY_ONLY would require a real enemy.
            if !auto_attackable {
                self.send_invalid_target().await?;
                return Ok(());
            }
        }

        let target_stats = if target_id == attacker_id {
            let player = self.try_get_selected_char()?;
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

        // --- Range & visibility checks ---
        let cast_range = skill_data.cast_range_at(level_u8);
        if target_id != attacker_id {
            if !self.check_visibility(target_x, target_y, target_z).await? {
                return Ok(());
            }
            let dist = calculate_distance(x, y, z, target_x, target_y, target_z).unwrap_or(0.0);
            if cast_range > 0 && dist > f64::from(cast_range + 40) {
                if msg.shift_pressed {
                    // Shift = don't move: cancel instead of walking into range.
                    self.send_packet(to_client::SystemMessage::new(
                        to_client::SystemMessageType::TheDistanceIsTooFarAndSoTheCastingHasBeenCancelled,
                    )?)
                    .await?;
                    self.send_packet(ActionFailed::normal()?).await?;
                    return Ok(());
                }
                // Walk into cast range and retry the same request upon arrival.
                let (hit_x, hit_y, hit_z) = calculate_nearest_hit_point(
                    (x, y, z),
                    (target_x, target_y, target_z),
                    dist,
                    cast_range,
                );
                self.start_movement(hit_x, hit_y, hit_z, self_actor.clone())?;
                let retry_actor = self_actor.clone();
                self.schedule_triggered_task(PlayerTasks::ActionIntent, async move {
                    let _ = retry_actor.tell(msg).await;
                });
                return Ok(());
            }
        }
        self.stop_movement();

        // --- MP consumption (initial + regular, docs/skills/01-skill-model-and-casting.md) ---
        let mp_cost = f64::from(
            skill_data.mp_initial_consume_at(level_u8) + skill_data.mp_consume_at(level_u8),
        );
        if mp_cost > 0.0 {
            if current_mp < mp_cost {
                self.send_packet(to_client::SystemMessage::new(
                    to_client::SystemMessageType::NotEnoughMp,
                )?)
                .await?;
                self.send_packet(ActionFailed::normal()?).await?;
                return Ok(());
            }
            let new_mp = {
                let player = self.try_get_selected_char_mut()?;
                player.stats.current_mp -= mp_cost;
                player.sync_vitals_to_model();
                player.stats.current_mp
            };
            let mut su = to_client::StatusUpdate::new(attacker_id)?;
            su.add_update(to_client::StatusUpdateType::CurMp, new_mp as i32)?;
            self.send_packet(su).await?;
        }

        // --- Cast timing: magic scales with casting speed, physical with attack speed ---
        let base_hit_time = skill_data.hit_time_at(level_u8);
        let hit_time = if base_hit_time > 0 {
            if skill_data.is_magic() {
                let m_atk_spd = attacker_stats.get(&Stat::MAtkSpd).copied().unwrap_or(333.0);
                (f64::from(base_hit_time) * 333.0 / m_atk_spd) as i32
            } else {
                let p_atk_spd = attacker_stats.get(&Stat::PAtkSpd).copied().unwrap_or(300.0);
                (f64::from(base_hit_time) * 300.0 / p_atk_spd) as i32
            }
        } else {
            0
        };

        // --- Reuse (cooldown) starts at cast start, like in retail ---
        let reuse_delay = i64::from(skill_data.reuse_delay_at(level_u8));
        let reuse_group = skill_data.reuse_delay_group_at(level_u8);
        let client_reuse_group = if reuse_group > 0 {
            reuse_group
        } else {
            msg.skill_id
        };
        {
            let player = self.try_get_selected_char_mut()?;
            player.add_skill_reuse(msg.skill_id, i32::from(level), reuse_delay, reuse_group);
        }

        // --- Broadcast cast start ---
        let magic_use_packet = to_client::MagicSkillUse::new(
            attacker_id,
            target_id,
            msg.skill_id,
            i32::from(level),
            hit_time,
            reuse_delay as i32,
            client_reuse_group,
            x,
            y,
            z,
            target_x,
            target_y,
            target_z,
        )?;
        self.controller.broadcast_packet(magic_use_packet);
        let mut use_msg = to_client::SystemMessage::new(to_client::SystemMessageType::YouUseS1)?;
        use_msg.add_param(to_client::SystemMessageParam::SkillName {
            id: msg.skill_id,
            level,
            sub_level: 0,
        })?;
        self.send_packet(use_msg).await?;

        // --- Launch task: gather affected targets and apply effects after the cast time ---
        let actions = classify_effects(&skill_data, level_u8);
        let is_continuous = skill_data.is_continuous();
        let abnormal = (
            skill_data
                .abnormal_type
                .as_ref()
                .and_then(|w| w.get(level_u8))
                .cloned(),
            skill_data
                .abnormal_level
                .as_ref()
                .and_then(|w| w.get(level_u8))
                .copied()
                .unwrap_or(i32::from(level)),
            skill_data.abnormal_time_at(level_u8),
        );
        // Buff actions without a continuous operate type make no sense; drop them.
        let actions: Vec<SkillAction> = actions
            .into_iter()
            .filter(|a| is_continuous || !matches!(a, SkillAction::Buff { .. }))
            .collect();

        let scope = skill_data.affect_scope_at(level_u8);
        let affect_range = skill_data.affect_range_at(level_u8);
        let affect_limit = skill_data.affect_limit_at(level_u8);
        let magic_crit_rate = f64::from(skill_data.magic_critical_rate_at(level_u8).max(0));
        let controller: Arc<_> = self.controller.clone();
        let skill_id = msg.skill_id;
        let skill_level = i32::from(level);

        let launch_task = tokio::spawn(async move {
            if hit_time > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(hit_time as u64)).await;
            }
            let main_target = AffectedTarget {
                id: target_id,
                actor: target_actor,
                stats: target_stats,
            };
            let targets = gather_affected_targets(
                &controller,
                scope,
                affect_range,
                affect_limit,
                attacker_id,
                (x, y, z),
                main_target,
            )
            .await;

            let target_ids: Vec<i32> = targets.iter().map(|t| t.id).collect();
            match to_client::MagicSkillLaunched::new(
                attacker_id,
                skill_id,
                skill_level,
                0, // casting_type
                &target_ids,
            ) {
                Ok(packet) => controller.broadcast_packet(packet),
                Err(err) => error!("Failed to build MagicSkillLaunched: {err}"),
            }

            apply_skill_actions(
                &actions,
                attacker_id,
                &attacker_name,
                &attacker_stats,
                magic_crit_rate,
                abnormal,
                (skill_id, skill_level),
                &targets,
            )
            .await;
        });
        self.schedule_task(PlayerTasks::CauseDamage, launch_task);

        Ok(())
    }
}
