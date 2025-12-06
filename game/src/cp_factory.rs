use crate::packets::from_client::action::Action;
use crate::packets::from_client::auth::AuthLogin;
use crate::packets::from_client::char_create::CreateCharRequest;
use crate::packets::from_client::char_restore::RestoreChar;
use crate::packets::from_client::char_select::SelectChar;
use crate::packets::from_client::delete_char::DeleteChar;
use crate::packets::from_client::enter_world::EnterWorld;
use crate::packets::from_client::extended::{
    CheckCharName, GoLobby, RequestKeyMapping, RequestManorList, RequestUserBanInfo,
    SelectedQuestZoneId, SendClientIni,
};
use crate::packets::from_client::logout::Logout;
use crate::packets::from_client::move_to_location::RequestMoveToLocation;
use crate::packets::from_client::new_char_request::NewCharacterRequest;
use crate::packets::from_client::noop::NoOp;
use crate::packets::from_client::protocol::ProtocolVersion;
use crate::packets::from_client::req_skill_cooltime::ReqSkillCoolTime;
use crate::packets::from_client::restart::RequestRestart;
use crate::packets::from_client::stop_move::StopMove;
use crate::packets::from_client::validate_position::ValidatePosition;
use anyhow::bail;
use bytes::BytesMut;
use l2_core::shared_packets::common::ReadablePacket;
use macro_common::PacketEnum;
use strum::Display;
use tracing::{error, info};

#[derive(Clone, Debug, Display, PacketEnum)]
pub enum PlayerPackets {
    ProtocolVersion(ProtocolVersion),
    AuthLogin(AuthLogin),
    NewCharacterRequest(NewCharacterRequest),
    CreateCharRequest(CreateCharRequest),
    Logout(Logout),
    DeleteChar(DeleteChar),
    RestoreChar(RestoreChar),
    SelectChar(SelectChar),
    EnterWorld(EnterWorld),
    GoLobby(GoLobby),
    CheckCharName(CheckCharName),
    SendClientIni(SendClientIni),
    RequestUserBanInfo(RequestUserBanInfo),
    RequestManorList(RequestManorList),
    RequestKeyMapping(RequestKeyMapping),
    NoOp(NoOp),
    MoveToLocation(RequestMoveToLocation),
    ReqRestart(RequestRestart),
    SelectedQuestZoneId(SelectedQuestZoneId),
    ReqSkillCoolTime(ReqSkillCoolTime),
    ValidatePosition(ValidatePosition),
    StopMove(StopMove),
    Action(Action),
}

pub fn build_client_packet(mut data: BytesMut) -> anyhow::Result<PlayerPackets> {
    if data.is_empty() {
        bail!("Not enough data to build packet: {data:?}");
    }
    let packet_id = data.split_to(1); // skip 1st byte, because it's packet id
    info!("Player packet ID: 0x{:02X}", packet_id[0]);

    match packet_id[0] {
        ProtocolVersion::PACKET_ID => {
            Ok(PlayerPackets::ProtocolVersion(ProtocolVersion::read(data)?))
        }
        AuthLogin::PACKET_ID => Ok(PlayerPackets::AuthLogin(AuthLogin::read(data)?)),
        NewCharacterRequest::PACKET_ID => Ok(PlayerPackets::NewCharacterRequest(
            NewCharacterRequest::read(data)?,
        )),
        CreateCharRequest::PACKET_ID => Ok(PlayerPackets::CreateCharRequest(
            CreateCharRequest::read(data)?,
        )),
        RequestMoveToLocation::PACKET_ID => Ok(PlayerPackets::MoveToLocation(
            RequestMoveToLocation::read(data)?,
        )),
        RequestRestart::PACKET_ID => Ok(PlayerPackets::ReqRestart(RequestRestart::read(data)?)),
        Logout::PACKET_ID => Ok(PlayerPackets::Logout(Logout::read(data)?)),
        DeleteChar::PACKET_ID => Ok(PlayerPackets::DeleteChar(DeleteChar::read(data)?)),
        RestoreChar::PACKET_ID => Ok(PlayerPackets::RestoreChar(RestoreChar::read(data)?)),
        SelectChar::PACKET_ID => Ok(PlayerPackets::SelectChar(SelectChar::read(data)?)),
        EnterWorld::PACKET_ID => Ok(PlayerPackets::EnterWorld(EnterWorld::read(data)?)),
        ValidatePosition::PACKET_ID => Ok(PlayerPackets::ValidatePosition(ValidatePosition::read(
            data,
        )?)),
        ReqSkillCoolTime::PACKET_ID => Ok(PlayerPackets::ReqSkillCoolTime(ReqSkillCoolTime::read(
            data,
        )?)),
        Action::PACKET_ID => Ok(PlayerPackets::Action(Action::read(data)?)),
        0xD0 => build_ex_client_packet(data),
        _ => {
            error!("Unknown Player packet ID: 0x{:02X}", packet_id[0]);
            Ok(PlayerPackets::NoOp(NoOp::read(data)?))
        }
    }
}

pub fn build_ex_client_packet(mut data: BytesMut) -> anyhow::Result<PlayerPackets> {
    if data.len() < 2 {
        bail!("Empty extended packet {data:?}");
    }
    let packet_id_bytes = data.split_to(2);
    let packet_id = u16::from_le_bytes([packet_id_bytes[0], packet_id_bytes[1]]);
    match Some(packet_id) {
        GoLobby::EX_PACKET_ID => Ok(PlayerPackets::GoLobby(GoLobby::read(data)?)),
        CheckCharName::EX_PACKET_ID => Ok(PlayerPackets::CheckCharName(CheckCharName::read(data)?)),
        SendClientIni::EX_PACKET_ID => Ok(PlayerPackets::SendClientIni(SendClientIni::read(data)?)),
        StopMove::EX_PACKET_ID => Ok(PlayerPackets::StopMove(StopMove::read(data)?)),
        RequestUserBanInfo::EX_PACKET_ID => Ok(PlayerPackets::RequestUserBanInfo(
            RequestUserBanInfo::read(data)?,
        )),
        RequestManorList::EX_PACKET_ID => Ok(PlayerPackets::RequestManorList(
            RequestManorList::read(data)?,
        )),
        SelectedQuestZoneId::EX_PACKET_ID => Ok(PlayerPackets::SelectedQuestZoneId(
            SelectedQuestZoneId::read(data)?,
        )),
        RequestKeyMapping::EX_PACKET_ID => Ok(PlayerPackets::RequestKeyMapping(
            RequestKeyMapping::read(data)?,
        )),
        _ => {
            error!("Unknown extended client packet ID: 0x{:x}", packet_id);
            Ok(PlayerPackets::NoOp(NoOp::read(data)?))
        }
    }
}
