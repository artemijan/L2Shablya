use crate::movement::calculate_distance;
use crate::packets::to_client;
use crate::pl_client::{ApplyDamage, GetStats, PlayerClient};
use bytes::BytesMut;
use kameo::message::{Context, Message};
use l2_core::errors::KameoAnyhowExt;
use l2_core::game_objects::stats::Formulas;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::shared_packets::write::SendablePacketBuffer;
use tracing::{error, info, instrument};

#[derive(Debug, Clone)]
pub struct Attack {
    pub buffer: SendablePacketBuffer,
    pub object_id: i32,
    pub origin_x: i32,
    pub origin_y: i32,
    pub origin_z: i32,
    pub attack_id: u8,
}

impl ReadablePacket for Attack {
    const PACKET_ID: u8 = 0x32;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        let object_id = buffer.read_i32()?;
        let origin_x = buffer.read_i32()?;
        let origin_y = buffer.read_i32()?;
        let origin_z = buffer.read_i32()?;
        let attack_id = buffer.read_byte()?;
        Ok(Self {
            buffer: SendablePacketBuffer::empty(),
            object_id,
            origin_x,
            origin_y,
            origin_z,
            attack_id,
        })
    }
}

impl Message<Attack> for PlayerClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: Attack,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let player = self.try_get_selected_char()?;
        let attacker_id = player.get_object_id();
        let attacker_name = player.get_visible_name().to_string();
        let attacker_stats = player.stats.cached_stats.clone();
        let (x, y, z) = (player.get_x(), player.get_y(), player.get_z());

        if let Some(target_actor) = self.controller.get_player_by_object_id(msg.object_id) {
            let target_stats = target_actor.ask(GetStats).await.anyhow()?;
            let (target_x, target_y, target_z) = (target_stats.x, target_stats.y, target_stats.z);

            let dist = calculate_distance(x, y, z, target_x, target_y, target_z).unwrap_or(0.0);

            if !self.check_visibility(target_x, target_y, target_z).await? {
                return Ok(());
            }

            // Physical attack range is typically short, e.g., 40.
            let attack_range = 40;
            if dist > (attack_range + 40) as f64 {
                info!("Target is too far, moving to target: dist={}, attack_range={}", dist, attack_range);

                // Start movement towards the target
                let pl_actor = _ctx.actor_ref().clone();
                self.start_movement(target_x, target_y, target_z, pl_actor)?;

                let self_actor = _ctx.actor_ref().clone();
                let msg_clone = msg.clone();

                tokio::spawn(async move {
                    // Poll for arrival
                    loop {
                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                        let pos = self_actor.ask(crate::pl_client::GetMovementPosition).await;
                        match pos {
                            Ok(Some((_cx, _cy, _cz, true))) => break, // Arrived
                            Ok(Some(_)) => continue, // Still moving
                            _ => return, // Movement stopped or error
                        }
                    }

                    // Arrived, try to hit again
                    let _ = self_actor.tell(msg_clone).await;
                });

                return Ok(());
            }

            let miss = Formulas::calc_hit_miss(&attacker_stats, &target_stats.stats);

            if miss {
                let mut sys_msg = to_client::SystemMessage::new(to_client::SystemMessageType::C1SAttackWentAstray)?;
                sys_msg.add_param(to_client::SystemMessageParam::PcName(attacker_name))?;
                self.send_packet(sys_msg).await?;
                
                let attack_packet = to_client::Attack::new(
                    attacker_id,
                    msg.object_id,
                    0,
                    1, // miss flag is usually 1 in some versions, 4 in others. Let's try 1 (MISS)
                    x,
                    y,
                    z,
                    target_x,
                    target_y,
                    target_z,
                )?;
                self.controller.broadcast_packet(attack_packet);
                return Ok(());
            }
            let damage = Formulas::calc_phys_dam(&attacker_stats, &target_stats.stats, false, false);
            target_actor
                .tell(ApplyDamage {
                    damage,
                    attacker_id,
                    attacker_name,
                })
                .await
                .anyhow()?;

            let attack_packet = to_client::Attack::new(
                attacker_id,
                msg.object_id,
                damage as i32,
                0,
                x,
                y,
                z,
                target_x,
                target_y,
                target_z,
            )?;

            self.controller.broadcast_packet(attack_packet);
        }
        Ok(())
    }
}
