use crate::packets::to_client::extended::{
    AutoSoulShots, BasicActionList, BookmarkInfo, EquippedItems, InventoryAdenaInfo,
    InventoryWeight, PledgeWaitingListAlarm, QuestItemList, Rotation, SetCompasZoneCode,
    SubclassInfo, SubclassInfoType, UISettings, UnreadMailCount, VitalityInfo,
};
use crate::packets::to_client::{
    AbnormalStatusUpdate, AcquireSkillList, CharEtcStatusUpdate, FriendList, HennaInfo, ItemList,
    MacroList, MoveTo, QuestList, ShortcutsInit, SkillCoolTime, SkillList, SystemMessage,
    SystemMessageType, UserInfo,
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
        self.stop_movement();
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
        self.send_packet(UserInfo::new(&player, UserInfoType::all(), &self.controller).await?)
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
        // if config.rates.enable_vitality
        self.send_packet(VitalityInfo::new(&player, &config)?)
            .await?;
        self.send_packet(UISettings::new(&player)?).await?;
        let macros_packets = MacroList::list_macros(&player)?;
        for m in macros_packets {
            self.send_packet(m).await?;
        }

        self.send_packet(BookmarkInfo::new(&player)?).await?;

        self.send_packet(ItemList::new(&player, false)?).await?;
        self.send_packet(QuestItemList::new(&player)?).await?;
        self.send_packet(ShortcutsInit::new(&player)?).await?;

        self.send_packet(BasicActionList::new(&self.controller.action_list)?)
            .await?;
        self.send_packet(SkillList::empty()?).await?;
        //todo: AuthGG check?

        self.send_packet(HennaInfo::new(&player)?).await?;

        Self::do_later(
            ctx.actor_ref().clone(),
            DoLater {
                delay: Duration::from_millis(500),
                callback: Box::new(move |actor: &mut PlayerClient| {
                    Box::pin(async move {
                        let player = actor.try_get_selected_char_mut()?;
                        let acquire_sl = AcquireSkillList::new(player)?;
                        let packet = SkillList::new(player)?;
                        actor.send_packet(packet).await?;
                        actor.send_packet(acquire_sl).await
                    })
                }),
            },
        );
        self.send_packet(CharEtcStatusUpdate::new(&player)?).await?;
        //todo: again send clan packets (please check why we need to send it twice!!!)
        if player.char_model.clan_id.is_some() {
            todo!("Clan packets");
        } else {
            self.send_packet(PledgeWaitingListAlarm::new()?).await?;
        }
        self.send_packet(SubclassInfo::new(&player, SubclassInfoType::NoChanges)?)
            .await?;
        self.send_packet(InventoryWeight::new(&player)?).await?;
        self.send_packet(InventoryAdenaInfo::new(&player)?).await?;
        self.send_packet(EquippedItems::new(&player, true)?).await?;
        let unread_mails = player.mailbox.iter().map(|m| m.is_unread).len();
        if unread_mails > 0 {
            self.send_packet(UnreadMailCount::new(u32::try_from(unread_mails)?)?)
                .await?;
        }
        //todo: trigger hook on player enter for quests
        self.send_packet(QuestList::new(&player)?).await?;
        //todo: check spawn protection and set it if any
        //todo: spawn player
        self.send_packet(Rotation::new(&player)?).await?;
        //todo: check isCursedWeaponEquipped
        //todo: check if PC points enabled and send update packet
        //todo: send expand storage packet (if there is a skill for that) with a delay of 300ms
        self.send_packet(FriendList::new(&player)?).await?;
        //todo: send friend list logged in to all friends (broadcast)
        //todo: send packet welcome to the L2 world
        //todo: show Announcements
        //todo: send message if auto restart is enabled
        //todo: show clan notice if enabled.
        //todo: show server news if enabled
        //todo: check petitions if enabled
        //todo: if it's dead then send Die packet
        //todo: on_player_enter hook
        self.send_packet(SkillCoolTime::new(&player)?).await?;
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

        self.controller
            .add_player_to_world(&player, ctx.actor_ref())
            .await?;
        let p = UserInfo::new(&player, UserInfoType::all(), &self.controller).await?;
        self.send_packet(p).await?;

        //todo: send ExBeautyItemList
        //todo: send ExWorldChatCnt if ENABLE_WORLD_CHAT is enabled
        //todo: send ExConnectedTimeAndGettableReward
        //todo: send ExOneDayReceiveRewardList
        self.send_packet(SetCompasZoneCode::new(0x0C)?).await?;
        self.send_packet(MoveTo::new(&player, player.get_location())?)
            .await?;
        self.send_packet(AutoSoulShots::new(0, true, 0)?).await?;
        self.send_packet(AutoSoulShots::new(0, true, 1)?).await?;
        self.send_packet(AutoSoulShots::new(0, true, 2)?).await?;
        self.send_packet(AutoSoulShots::new(0, true, 3)?).await?;
        //todo: update abnormal visual effects
        self.send_packet(AbnormalStatusUpdate::new(&player)?)
            .await?;
        self.send_packet(SystemMessage::new(
            SystemMessageType::WelcomeToTheWorldOfLineage2,
        )?)
        .await?;
        //todo: if attendance enabled, send in async packets
        //todo: if HWID enabled, then check it
        //todo: show chat banned icon if player can't speak
        //todo: finish entering the world
        //todo: run PCafe points program
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::GameController;
    use crate::ls_client::LoginServerClient;
    use crate::pl_client::ClientStatus;
    use crate::test_utils::test::{
        get_gs_config, spawn_custom_ls_client_actor, spawn_custom_player_client_actor,
    };
    use entities::entities::user;
    use entities::test_factories::factories::{char_factory, user_factory};
    use l2_core::game_objects::player::Player;
    use std::collections::{HashMap, VecDeque};
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::{get_test_db, DBPool};
    use tokio::io::{split, AsyncReadExt, DuplexStream};
    use tokio::time::{sleep, timeout};

    async fn prepare_pl() -> PlayerClient {
        let pool = get_test_db().await;
        let cfg = Arc::new(get_gs_config());
        let controller = Arc::new(GameController::from_config(cfg).await);
        PlayerClient::new(Ipv4Addr::LOCALHOST, controller.clone(), pool.clone())
    }

    async fn create_user(pool: &DBPool) -> user::Model {
        user_factory(pool, |mut u| {
            u.username = String::from("test");
            u
        })
        .await
    }

    // create a simple character for tests
    async fn create_char(pl: &PlayerClient, user_id: i32) -> Player {
        let char_model = char_factory(&pl.db_pool, |mut c| {
            c.user_id = user_id;
            c.name = "TestChar".to_owned();
            c
        })
        .await;
        let tmpl = pl
            .controller
            .class_templates
            .try_get_template(char_model.class_id)
            .unwrap()
            .clone();
        Player::new(char_model, vec![], tmpl)
    }

    fn enter_world_with_ip(ip: [u8; 4]) -> EnterWorld {
        EnterWorld {
            tracert: [ip, [1, 1, 1, 1], [2, 2, 2, 2], [3, 3, 3, 3], [4, 4, 4, 4]],
        }
    }
    /// Read a single framed packet from the client stream: [u16_le len][body]
    async fn read_one_packet(stream: &mut DuplexStream) -> anyhow::Result<Vec<u8>> {
        let mut len_buf = [0u8; 2];
        stream.read_exact(&mut len_buf).await?; // will error on EOF
        let body_len = u16::from_le_bytes(len_buf) as usize;
        anyhow::ensure!(body_len >= 2, "Invalid frame size: {}", body_len);
        let mut body = vec![0u8; body_len - 2];
        stream.read_exact(&mut body).await?;
        Ok(body)
    }

    /// Collect packets until there is a quiet period (idle_timeout) after a grace wait
    async fn collect_packets_with_idle_timeout(
        client_rx: &mut DuplexStream,
        initial_grace: Duration,
        idle_timeout: Duration,
    ) -> anyhow::Result<Vec<Vec<u8>>> {
        // Give EnterWorld time to schedule and send the delayed packets
        if !initial_grace.is_zero() {
            sleep(initial_grace).await;
        }

        let mut packets = Vec::new();
        loop {
            // Try to read the next packet; if no packet arrives for idle_timeout, stop
            match timeout(idle_timeout, read_one_packet(client_rx)).await {
                Ok(Ok(pkt)) => packets.push(pkt),
                Ok(Err(e)) => {
                    // Stream closed – finish
                    return Err(e);
                }
                Err(_) => break, // idle timeout elapsed, assume no more packets for now
            }
        }
        Ok(packets)
    }

    /// Extract L2 packet ID for assertion. Normal packets start with 1 byte.
    /// Extended packets start with 0xFE then a u16_le subtype.
    fn packet_id(body: &[u8]) -> String {
        if body.is_empty() {
            return "<empty>".into();
        }
        if body[0] == 0xFE {
            if body.len() >= 3 {
                let sub = u16::from_le_bytes([body[1], body[2]]);
                format!("0x{sub:04X}")
            } else {
                "FE:<unknown>".into()
            }
        } else {
            format!("0x{:02X}", body[0])
        }
    }

    #[tokio::test]
    async fn test_error_when_not_entering_state() {
        let pack = enter_world_with_ip([127, 0, 0, 1]);
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let mut pl_client = prepare_pl().await;
        pl_client.set_status(ClientStatus::Authenticated);
        let player_actor = spawn_custom_player_client_actor(
            pl_client.controller.clone(),
            pl_client.db_pool.clone(),
            r,
            w,
            Some(pl_client),
        )
        .await;
        let res = player_actor.ask(pack).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_error_missing_user() {
        let pack = enter_world_with_ip([127, 0, 0, 1]);
        let (_client, server) = tokio::io::duplex(1024);
        let (_ls_client_stream, ls_server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let (ls_r, ls_w) = split(ls_server);
        let mut pl_client = prepare_pl().await;
        pl_client.set_status(ClientStatus::Entering);
        // spawn LS actor so we pass try_get_ls_actor and fail on missing user
        let ls_client = LoginServerClient::new(
            Ipv4Addr::LOCALHOST,
            pl_client.controller.clone(),
            pl_client.db_pool.clone(),
        );
        let ls_actor = spawn_custom_ls_client_actor(
            pl_client.controller.clone(),
            pl_client.db_pool.clone(),
            ls_r,
            ls_w,
            Some(ls_client),
        )
        .await;
        pl_client.controller.set_ls_actor(ls_actor).await;

        let player_actor = spawn_custom_player_client_actor(
            pl_client.controller.clone(),
            pl_client.db_pool.clone(),
            r,
            w,
            Some(pl_client),
        )
        .await;
        let res = player_actor.ask(pack).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_error_missing_selected_char() {
        let pack = enter_world_with_ip([127, 0, 0, 1]);
        let (_client, server) = tokio::io::duplex(1024);
        let (_ls_client_stream, ls_server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let (ls_r, ls_w) = split(ls_server);
        let mut pl_client = prepare_pl().await;
        pl_client.set_status(ClientStatus::Entering);
        let user = create_user(&pl_client.db_pool).await;
        pl_client.set_user(user);
        // Spawn LS actor so we proceed to selected char check
        let ls_client = LoginServerClient::new(
            Ipv4Addr::LOCALHOST,
            pl_client.controller.clone(),
            pl_client.db_pool.clone(),
        );
        let ls_actor = spawn_custom_ls_client_actor(
            pl_client.controller.clone(),
            pl_client.db_pool.clone(),
            ls_r,
            ls_w,
            Some(ls_client),
        )
        .await;
        pl_client.controller.set_ls_actor(ls_actor).await;
        // Have characters but do not select, to trigger try_get_selected_char error
        pl_client.set_account_chars(vec![]);

        let player_actor = spawn_custom_player_client_actor(
            pl_client.controller.clone(),
            pl_client.db_pool.clone(),
            r,
            w,
            Some(pl_client),
        )
        .await;
        let res = player_actor.ask(pack).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn test_handle_ok_status_and_tracert() {
        let pack = enter_world_with_ip([127, 0, 0, 1]);
        let (mut client_rx, server) = tokio::io::duplex(4096);
        let (_ls_client_stream, ls_server) = tokio::io::duplex(4096);
        let (r, w) = split(server);
        let (ls_r, ls_w) = split(ls_server);
        let mut pl_client = prepare_pl().await;
        pl_client.set_status(ClientStatus::Entering);
        let user = create_user(&pl_client.db_pool).await;
        let player = create_char(&pl_client, user.id).await;
        pl_client.set_user(user);
        pl_client.set_account_chars(vec![player]);
        pl_client.select_char(0);

        // Login server actor required for tracert tell
        let controller = pl_client.controller.clone();
        let pool = controller.get_db_pool().clone();
        let ls_client = LoginServerClient::new(Ipv4Addr::LOCALHOST, controller.clone(), pool);
        let ls_actor = spawn_custom_ls_client_actor(
            controller.clone(),
            controller.get_db_pool().clone(),
            ls_r,
            ls_w,
            Some(ls_client),
        )
        .await;
        controller.set_ls_actor(ls_actor).await;

        let player_actor = spawn_custom_player_client_actor(
            pl_client.controller.clone(),
            pl_client.db_pool.clone(),
            r,
            w,
            Some(pl_client),
        )
        .await;
        let res = player_actor.ask(pack).await;
        assert!(res.is_ok());
        // Collect packets: wait long enough to include DoLater(500ms) packets
        let frames = collect_packets_with_idle_timeout(
            &mut client_rx,
            Duration::from_millis(650), // initial grace
            Duration::from_millis(200), // idle timeout
        )
        .await
        .expect("collect packets");

        let ids: Vec<String> = frames.iter().map(|b| packet_id(b)).collect();
        let mut positions = HashMap::new();
        for (index, item) in ids.iter().enumerate() {
            positions
                .entry(item)
                .or_insert(VecDeque::new())
                .push_back(index);
        }
        let mut pos = |tag: &str| {
            let poss = positions.get_mut(&tag.to_string()).unwrap();
            poss.pop_front().unwrap()
        };

        // Replace these with the actual IDs in your protocol
        // For example, if UserInfo is 0x32, write "32"; if extended, use "FE:xxxx"
        let user_info = pos(&packet_id(&UserInfo::PACKET_ID.to_le_bytes()));
        let vitality = pos(&packet_id(
            &VitalityInfo::PACKET_ID
                .to_le_bytes()
                .into_iter()
                .chain(VitalityInfo::EX_PACKET_ID.to_le_bytes().into_iter())
                .collect::<Vec<_>>(),
        ));
        let ui_settings = pos(&packet_id(
            &UISettings::PACKET_ID
                .to_le_bytes()
                .into_iter()
                .chain(UISettings::EX_PACKET_ID.to_le_bytes().into_iter())
                .collect::<Vec<_>>(),
        ));
        let macro_list = pos(&packet_id(&MacroList::PACKET_ID.to_le_bytes()));
        let bookmark = pos(&packet_id(
            &BookmarkInfo::PACKET_ID
                .to_le_bytes()
                .into_iter()
                .chain(BookmarkInfo::EX_PACKET_ID.to_le_bytes().into_iter())
                .collect::<Vec<_>>(),
        ));
        let item_list = pos(&packet_id(&ItemList::PACKET_ID.to_le_bytes()));
        let quest_items = pos(&packet_id(
            &QuestItemList::PACKET_ID
                .to_le_bytes()
                .into_iter()
                .chain(QuestItemList::EX_PACKET_ID.to_le_bytes().into_iter())
                .collect::<Vec<_>>(),
        ));
        let shortcuts_init = pos(&packet_id(&ShortcutsInit::PACKET_ID.to_le_bytes()));
        let basic_action_list = pos(&packet_id(
            &BasicActionList::PACKET_ID
                .to_le_bytes()
                .into_iter()
                .chain(BasicActionList::EX_PACKET_ID.to_le_bytes().into_iter())
                .collect::<Vec<_>>(),
        ));

        let char_etc_status_update = pos(&packet_id(&CharEtcStatusUpdate::PACKET_ID.to_le_bytes()));
        let pledge_waiting_list_alarm = pos(&packet_id(
            &PledgeWaitingListAlarm::PACKET_ID
                .to_le_bytes()
                .into_iter()
                .chain(
                    PledgeWaitingListAlarm::EX_PACKET_ID
                        .to_le_bytes()
                        .into_iter(),
                )
                .collect::<Vec<_>>(),
        ));
        let subclass_info = pos(&packet_id(
            &SubclassInfo::PACKET_ID
                .to_le_bytes()
                .into_iter()
                .chain(SubclassInfo::EX_PACKET_ID.to_le_bytes().into_iter())
                .collect::<Vec<_>>(),
        ));
        let inventory_weight = pos(&packet_id(
            &InventoryWeight::PACKET_ID
                .to_le_bytes()
                .into_iter()
                .chain(InventoryWeight::EX_PACKET_ID.to_le_bytes().into_iter())
                .collect::<Vec<_>>(),
        ));
        let inventory_adena_info = pos(&packet_id(
            &InventoryAdenaInfo::PACKET_ID
                .to_le_bytes()
                .into_iter()
                .chain(InventoryAdenaInfo::EX_PACKET_ID.to_le_bytes().into_iter())
                .collect::<Vec<_>>(),
        ));
        let equipped_items = pos(&packet_id(
            &EquippedItems::PACKET_ID
                .to_le_bytes()
                .into_iter()
                .chain(EquippedItems::EX_PACKET_ID.to_le_bytes().into_iter())
                .collect::<Vec<_>>(),
        ));
        let quest_list = pos(&packet_id(&QuestList::PACKET_ID.to_le_bytes()));
        let rotation = pos(&packet_id(
            &Rotation::PACKET_ID
                .to_le_bytes()
                .into_iter()
                .chain(Rotation::EX_PACKET_ID.to_le_bytes().into_iter())
                .collect::<Vec<_>>(),
        ));

        let friend_list = pos(&packet_id(&FriendList::PACKET_ID.to_le_bytes()));
        let skill_cool_time = pos(&packet_id(&SkillCoolTime::PACKET_ID.to_le_bytes()));
        let user_info_2 = pos(&packet_id(&UserInfo::PACKET_ID.to_le_bytes()));
        let set_compas_zone_code = pos(&packet_id(
            &SetCompasZoneCode::PACKET_ID
                .to_le_bytes()
                .into_iter()
                .chain(SetCompasZoneCode::EX_PACKET_ID.to_le_bytes().into_iter())
                .collect::<Vec<_>>(),
        ));
        let sh_id = packet_id(
            &AutoSoulShots::PACKET_ID
                .to_le_bytes()
                .into_iter()
                .chain(AutoSoulShots::EX_PACKET_ID.to_le_bytes().into_iter())
                .collect::<Vec<_>>(),
        );
        let move_to = pos(&packet_id(&MoveTo::PACKET_ID.to_le_bytes()));
        let auto_soul_shots_1 = pos(&sh_id);
        let auto_soul_shots_2 = pos(&sh_id);
        let auto_soul_shots_3 = pos(&sh_id);
        let auto_soul_shots_4 = pos(&sh_id);
        let abnormal_status_update =
            pos(&packet_id(&AbnormalStatusUpdate::PACKET_ID.to_le_bytes()));
        let system_message = pos(&packet_id(&SystemMessage::PACKET_ID.to_le_bytes()));
        let skill_list_2 = pos(&packet_id(&SkillList::PACKET_ID.to_le_bytes()));
        let aq_skill_list = pos(&packet_id(&AcquireSkillList::PACKET_ID.to_le_bytes()));

        assert!(user_info < vitality);
        assert!(vitality < ui_settings);
        assert!(ui_settings < macro_list);
        assert!(macro_list < bookmark);
        assert!(bookmark < item_list);
        assert!(item_list < quest_items);
        assert!(quest_items < shortcuts_init);
        assert!(shortcuts_init < basic_action_list);
        assert!(basic_action_list < char_etc_status_update);
        assert!(char_etc_status_update < pledge_waiting_list_alarm);
        assert!(pledge_waiting_list_alarm < subclass_info);
        assert!(subclass_info < inventory_weight);
        assert!(inventory_weight < inventory_adena_info);
        assert!(inventory_adena_info < equipped_items);
        assert!(equipped_items < quest_list);
        assert!(quest_list < rotation);
        assert!(rotation < friend_list);
        assert!(friend_list < skill_cool_time);
        assert!(skill_cool_time < user_info_2);
        assert!(user_info_2 < set_compas_zone_code);
        assert!(set_compas_zone_code < move_to);
        assert!(move_to < auto_soul_shots_1);
        assert!(auto_soul_shots_1 < auto_soul_shots_2);
        assert!(auto_soul_shots_2 < auto_soul_shots_3);
        assert!(auto_soul_shots_3 < auto_soul_shots_4);
        assert!(auto_soul_shots_4 < abnormal_status_update);
        assert!(abnormal_status_update < system_message);
        assert!(system_message < skill_list_2);
        assert!(skill_list_2 < aq_skill_list);
    }
}
