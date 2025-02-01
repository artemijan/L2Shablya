use super::{gs_2_ls::ReplyChars, read::ReadablePacketBuffer};
use crate as l2_core;
use crate::shared_packets::ls_2_gs::PlayerAuthResponse;
use crate::shared_packets::write::SendablePacketBuffer;
use anyhow::bail;
use macro_common::SendablePacketImpl;
use num_enum::TryFromPrimitive;
use std::str::FromStr;
use std::{fmt::Debug, net::Ipv4Addr};

pub trait SendablePacket: Debug + Send + Sync {
    fn get_bytes(&mut self, with_padding: bool) -> &mut [u8];
}

pub trait ReadablePacket: Debug + Send + Sync {
    const PACKET_ID: u8;
    const EX_PACKET_ID: Option<u16>;
    fn read(data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized + ReadablePacket;
}
#[repr(u8)]
#[allow(unused)]
#[derive(Clone, Debug)]
pub enum LoginServerOpcodes {
    Init = 0x00,
    LoginOk = 0x03,
    ServerList = 0x04,
    GgAuth = 0x0b,
    LoginFail = 0x01,
    AccountKicked = 0x02,
    PlayFail = 0x06,
    PlayOk = 0x07,
    PiAgreementCheck = 0x11,
    PiAgreementAck = 0x12,
    LoginOptFail = 0x0D,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum ServerStatus {
    Auto = 0x00,
    Good = 0x01,
    Normal = 0x02,
    Full = 0x03,
    Down = 0x04,
    GmOnly = 0x05,
}

#[derive(Debug, Clone)]
pub struct ServerData {
    pub ip: Ipv4Addr,
    pub port: i32,
    pub age_limit: i32,
    pub pvp: bool,
    pub current_players: i32,
    pub max_players: i32,
    pub brackets: bool,
    pub clock: bool,
    pub status: Option<ServerStatus>,
    pub server_id: i32,
    pub server_type: Option<ServerType>,
}

impl ServerData {
    #[must_use]
    pub fn get_ip_octets(&self) -> [u8; 4] {
        self.ip.octets()
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(i32)]
pub enum ServerType {
    Normal = 0x01,
    Relax = 0x02,
    Test = 0x04,
    Nolabel = 0x08,
    CreationRestricted = 0x10,
    Event = 0x20,
    Free = 0x40,
}

impl FromStr for ServerType {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "normal" => Ok(ServerType::Normal),
            "relax" => Ok(ServerType::Relax),
            "test" => Ok(ServerType::Test),
            "nolabel" => Ok(ServerType::Nolabel),
            "creationrestricted" => Ok(ServerType::CreationRestricted),
            "event" => Ok(ServerType::Event),
            "free" => Ok(ServerType::Free),
            _ => Err(format!("Invalid server type: {input}")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PacketType {
    ReplyChars(ReplyChars),
    PlayerAuthResp(PlayerAuthResponse),
}

#[repr(u8)]
#[allow(unused)]
#[derive(Debug, Clone)]
pub enum GSLoginFailReasons {
    None = 0x00,
    IpBanned = 0x01,
    IpRreserved = 0x02,
    WrongHexId = 0x03,
    IdReserved = 0x04,
    NoFreeID = 0x05,
    NotAuthed = 0x06,
    AlreadyRegistered = 0x07,
}

impl GSLoginFailReasons {
    #[allow(clippy::missing_errors_doc)]
    pub fn from_u8(reason: u8) -> anyhow::Result<Self> {
        match reason {
            0x00 => Ok(Self::None),
            0x01 => Ok(Self::IpBanned),
            0x02 => Ok(Self::IpRreserved),
            0x03 => Ok(Self::WrongHexId),
            0x04 => Ok(Self::IdReserved),
            0x05 => Ok(Self::NoFreeID),
            0x06 => Ok(Self::NotAuthed),
            0x07 => Ok(Self::AlreadyRegistered),
            _ => bail!("Unknown reason"),
        }
    }
}

#[repr(u8)]
#[allow(unused)]
#[derive(Debug, Clone)]
pub enum PlayerLoginFailReasons {
    ReasonNoMessage = 0x00,
    ReasonSystemErrorLoginLater = 0x01,
    /// this will close client, so user has to restart game
    ReasonUserOrPassWrong = 0x02,
    ReasonAccessFailedTryAgainLater = 0x04,
    ReasonAccountInfoIncorrectContactSupport = 0x05,
    /// maybe this is good for N tries and after N use 0x02
    ReasonNotAuthed = 0x06,
    ReasonAccountInUse = 0x07,
    ReasonUnder18YearsKr = 0x0C,
    ReasonServerOverloaded = 0x0F,
    ReasonServerMaintenance = 0x10,
    ReasonTempPassExpired = 0x11,
    ReasonGameTimeExpired = 0x12,
    ReasonNoTimeLeft = 0x13,
    ReasonSystemError = 0x14,
    ReasonAccessFailed = 0x15,
    ReasonRestrictedIp = 0x16,
    ReasonWeekUsageFinished = 0x1E,
    ReasonSecurityCardNumberInvalid = 0x1F,
    ReasonAgeNotVerifiedCantLogBeetween10pm6am = 0x20,
    ReasonServerCannotBeAccessedByYourCoupon = 0x21,
    ReasonDualBox = 0x23,
    ReasonInactive = 0x24,
    ReasonUserAgreementRejectedOnWebsite = 0x25,
    ReasonGuardianConsentRequired = 0x26,
    ReasonUserAgreementDeclinedOrWithdrawlRequest = 0x27,
    ReasonAccountSuspendedCall = 0x28,
    ReasonChangePasswordAndQuizOnWebsite = 0x29,
    ReasonAlreadyLoggedInto10Accounts = 0x2A,
    ReasonMasterAccountRestricted = 0x2B,
    ReasonCertificationFailed = 0x2E,
    ReasonTelephoneCertificationUnavailable = 0x2F,
    ReasonTelephoneSignalsDelayed = 0x30,
    ReasonCertificationFailedLineBusy = 0x31,
    ReasonCertificationServiceNumberExpiredOrIncorrect = 0x32,
    ReasonCertificationServiceCurrentlyBeingChecked = 0x33,
    ReasonCertificationServiceCantBeUsedHeavyVolume = 0x34,
    ReasonCertificationServiceExpiredGameplayBlocked = 0x35,
    ReasonCertificationFailed3TimesGameplayBlocked30Min = 0x36,
    ReasonCertificationDailyUseExceeded = 0x37,
    ReasonCertificationUnderwayTryAgainLater = 0x38,
}

#[derive(Debug, Clone, SendablePacketImpl)]
pub struct GSLoginFail {
    pub buffer: SendablePacketBuffer,
    pub reason: GSLoginFailReasons,
}
impl ReadablePacket for GSLoginFail {
    const PACKET_ID: u8 = 0x01;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: &[u8]) -> anyhow::Result<Self>
    where
        Self: Sized + ReadablePacket,
    {
        let mut buffer = ReadablePacketBuffer::new(data);
        buffer.read_byte()?; //packet id
        let reason = buffer.read_byte()?;
        Ok(Self {
            buffer: SendablePacketBuffer::empty(),
            reason: GSLoginFailReasons::from_u8(reason)?,
        })
    }
}

impl GSLoginFail {
    #[allow(clippy::missing_errors_doc)]
    pub fn new(reason: GSLoginFailReasons) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
            reason,
        };
        inst.write_all()?;
        Ok(inst)
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write(GSLoginFailReasons::NotAuthed as u8)?;
        self.buffer.write(self.reason.clone() as u8)?;
        Ok(())
    }
}

#[derive(Debug, Clone, SendablePacketImpl)]
pub struct PlayerLoginFail {
    pub buffer: SendablePacketBuffer,
    pub reason: PlayerLoginFailReasons,
}

impl PlayerLoginFail {
    #[allow(clippy::missing_errors_doc)]
    pub fn new(reason: PlayerLoginFailReasons) -> anyhow::Result<Self> {
        let mut login_ok = Self {
            buffer: SendablePacketBuffer::new(),
            reason,
        };
        login_ok.write_all()?;
        Ok(login_ok)
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write(LoginServerOpcodes::LoginFail as u8)?;
        self.buffer.write(self.reason.clone() as u8)?;
        Ok(())
    }
}

#[repr(i32)]
#[derive(Clone, Debug, Default, Copy)]
pub enum GSStatus {
    Auto = 0x00,
    Good = 0x01,
    Normal = 0x02,
    Full = 0x03,
    #[default]
    Down = 0x04,
    GmOnly = 0x05,
}

impl GSStatus {
    #[must_use]
    pub fn from_opcode(opcode: i32) -> Option<Self> {
        match opcode {
            0x00 => Some(Self::Auto),
            0x01 => Some(Self::Good),
            0x02 => Some(Self::Normal),
            0x03 => Some(Self::Full),
            0x04 => Some(Self::Down),
            0x05 => Some(Self::GmOnly),
            _ => None,
        }
    }
}
