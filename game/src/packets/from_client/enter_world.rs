use crate::client_thread::{ClientHandler, ClientStatus};
use crate::ls_thread::LoginHandler;
use crate::packets::to_client::UserInfo;
use crate::packets::HandleablePacket;
use anyhow::bail;
use async_trait::async_trait;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::gs_2_ls::PlayerTracert;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::traits::handlers::PacketSender;
use l2_core::model::user_info::UserInfoType;
use tracing::info;

#[derive(Debug, Clone, Default)]
pub struct EnterWorld {
    tracert: [[u8; 4]; 5],
}

impl ReadablePacket for EnterWorld {
    const PACKET_ID: u8 = 0x11;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(data: &[u8]) -> anyhow::Result<Self> {
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

#[async_trait]
impl HandleablePacket for EnterWorld {
    type HandlerType = ClientHandler;
    async fn handle(&self, handler: &mut Self::HandlerType) -> anyhow::Result<()> {
        if handler.get_status() != &ClientStatus::Entering {
            bail!("Not in entering state")
        }
        handler.set_status(ClientStatus::InGame);
        let mut addresses = Vec::with_capacity(5);
        for i in 0..5 {
            addresses.push(format!(
                "{}.{}.{}.{}",
                self.tracert[i][0], self.tracert[i][1], self.tracert[i][2], self.tracert[i][3]
            ));
        }
        let [ip, hop1, hop2, hop3, hop4]: [String; 5] =
            addresses.try_into().expect("Expected 5 tracert addresses");

        if ip != handler.get_ip().to_string() {
            bail!("IP address client sent, doesn't much with what we got from socket.");
        }
        let controller = handler.get_controller();
        let config = controller.get_cfg();
        controller
            .message_broker
            .notify(
                LoginHandler::HANDLER_ID,
                Box::new(PlayerTracert::new(
                    handler.try_get_user()?.username.clone(),
                    ip,
                    hop1,
                    hop2,
                    hop3,
                    hop4,
                )?),
            )
            .await?;
        let player = handler.try_get_selected_char()?;
        
        handler.send_packet(Box::new(UserInfo::new(player, UserInfoType::all())?)).await?;
        //todo: send user info
        //todo: restore player in the instance
        //todo: send clan packet
        if config.rates.enable_vitality {
            info!("Vitality enabled.");
            //todo: send vitality packet
        }
        //todo: send macro list packet
        //todo: send teleport bookmark list
        //todo: send item list, without showing a window
        //todo: send quest list
        //todo: send shortcuts
        //todo: send action list (static packet, all actions are predefined)
        //todo: send blank skill list (I suppose because we don't need to show them yet)
        //todo: AuthGG check?
        //todo: send a skill list asynchronously without waiting a result
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
