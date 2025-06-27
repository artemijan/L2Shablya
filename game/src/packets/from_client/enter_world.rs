use crate::packets::to_client::extended::{ActionList, BookmarkInfo, QuestItemList, VitalityInfo};
use crate::packets::to_client::{
    HennaInfo, ItemList, MacroList, ShortcutsInit, SkillList, UserInfo,
};
use crate::pl_client::{ClientStatus, DoLater, PlayerClient};
use anyhow::bail;
use bytes::BytesMut;
use kameo::message::{Context, Message};
use l2_core::game_objects::player::user_info::UserInfoType;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::gs_2_ls::PlayerTracert;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use std::time::Duration;
use tracing::{instrument, warn};

#[derive(Debug, Clone, Default)]
pub struct EnterWorld {
    tracert: [[u8; 4]; 5],
}

impl ReadablePacket for EnterWorld {
    const PACKET_ID: u8 = 0x11;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        let mut inst = Self {
            ..Default::default()
        };
        for i in 0..5 {
            for o in 0..4 {
                let b = buffer.read_byte()?;
                inst.tracert[i][o] = b;
            }
        }
        // buffer.read_i32()?; // Unknown Value
        // buffer.read_i32()?; // Unknown Value
        // buffer.read_i32()?; // Unknown Value
        // buffer.read_i32()?; // Unknown Value
        // buffer.read_bytes(64)?; // Unknown Byte Array
        // buffer.read_i32()?; // Unknown Value
        Ok(inst)
    }
}

impl Message<EnterWorld> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, ctx))]
    async fn handle(
        &mut self,
        msg: EnterWorld,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        if self.get_status() != &ClientStatus::Entering {
            bail!("Not in entering state")
        }
        self.set_status(ClientStatus::InGame);
        let mut addresses = Vec::with_capacity(5);
        for i in 0..5 {
            addresses.push(format!(
                "{}.{}.{}.{}",
                msg.tracert[i][0], msg.tracert[i][1], msg.tracert[i][2], msg.tracert[i][3]
            ));
        }
        let [ip, hop1, hop2, hop3, hop4]: [String; 5] =
            addresses.try_into().expect("Expected 5 tracert addresses");

        if ip != self.ip.to_string() {
            warn!(
                "IP address client sent {ip:?}, doesn't much with what we got from socket {:?}",
                self.ip
            );
        }
        let config = self.controller.get_cfg();
        self.controller
            .try_get_ls_actor()
            .await?
            .tell(PlayerTracert::new(
                self.try_get_user()?.username.clone(),
                ip,
                hop1,
                hop2,
                hop3,
                hop4,
            )?)
            .await?;

        let player = self.try_get_selected_char()?.clone();
        self.send_packet(
            UserInfo::new(&player, UserInfoType::all(), &self.controller)
                .await?
                .buffer,
        )
        .await?;
        if config.restore_player_instance {
            //todo: restore player in the instance
        }
        if player.is_gm() {
            //todo: gm startup process.
        } else {
            //todo: update pvp title
        }
        if player.char_model.clan_id.is_some() {
            //todo: send clan packet
        }
        if config.rates.enable_vitality {
            self.send_packet(VitalityInfo::new(&player, &config)?.buffer)
                .await?;
        }
        let macros_packets = MacroList::list_macros(&player)?;
        for m in macros_packets {
            self.send_packet(m.buffer).await?;
        }

        self.send_packet(BookmarkInfo::new(&player)?.buffer).await?;

        self.send_packet(ItemList::new(&player, false)?.buffer)
            .await?;

        self.send_packet(QuestItemList::new(&player)?.buffer)
            .await?;
        self.send_packet(ShortcutsInit::new(&player)?.buffer)
            .await?;

        self.send_packet(ActionList::new(&self.controller)?.buffer)
            .await?;
        self.send_packet(SkillList::empty()?.buffer).await?;
        //todo: AuthGG check?

        self.send_packet(HennaInfo::new(&player)?.buffer).await?;
        self.do_later(
            ctx.actor_ref(),
            DoLater {
                delay: Duration::from_millis(300),
                callback: Box::new(move |actor: &mut PlayerClient| {
                    //todo: Send real Skill list
                    Box::pin(async move { actor.send_packet(SkillList::empty()?.buffer).await })
                }),
            },
        );
        //todo: send etc status update packet
        //todo: again send clan packets (please check why we need to send it twice!!!)
        //todo: if no clan then send ExPledgeWaitingListAlarm
        //todo: send subclass info packet
        //todo: send inventory info
        //todo: Send Adena / Inventory Count Info
        //todo: Send Equipped Items
        //todo: Send Unread Mail Count if any
        //todo: trigger hook on player enter for quests
        //todo: send quest list again (but why?)
        //todo: check spawn protection and set it if any
        //todo: spawn player
        //todo: send ExRotation packet
        //todo: check isCursedWeaponEquipped
        //todo: check if PC points enabled and send update packet
        //todo: send expand storage packet (if there is a skill for that)
        //todo: send friend list logged in to all friends (broadcast)
        //todo: send packet welcome to the L2 world
        //todo: show Announcements
        //todo: send message if auto restart is enabled
        //todo: show clan notice if enabled.
        //todo: show server news if enabled
        //todo: check petitions if enabled
        //todo: if it's dead then send Die packet
        //todo: on_player_enter hook
        //todo: send skill cool time update
        //todo: send vote system info
        //todo: handle shadow items or items with mana
        //todo: do the same for items in warehouse
        //todo: send a message if recently dismissed from a clan
        //todo: remove combat flag before teleporting from battle ground
        //todo: teleport if needed
        //todo: check over enchanted items and punish if any
        //todo: Remove demonic weapon if character is not cursed weapon equipped.
        //todo: send unread mail again (but why?)
        //todo: send welcome message again (but why?)
        //todo: send message about premium items (maybe premium account or so?)
        //todo: check if offline trade and cancel it
        //todo: broadcast user info
        //todo: send ExBeautyItemList
        //todo: send ExWorldChatCnt if ENABLE_WORLD_CHAT is enabled
        //todo: send ExConnectedTimeAndGettableReward
        //todo: send ExOneDayReceiveRewardList
        //todo: send ExAutoSoulShot 0
        //todo: send ExAutoSoulShot 1
        //todo: send ExAutoSoulShot 2
        //todo: send ExAutoSoulShot 3
        //todo: update abnormal visual effects
        //todo: if attendance enabled, send in async packets
        //todo: if HWID enabled, then check it
        //todo: show chat banned icon if player can't speak
        //todo: finish entering the world
        //todo: run PCafe points program
        Ok(())
    }
}
