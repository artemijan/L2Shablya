pub mod from_client;
pub mod common;
pub mod error;
pub mod ls_factory;
pub mod from_gs;
pub mod gs_factory;
pub mod to_gs;
pub mod to_client;
pub mod login_fail;

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

#[repr(u8)]
#[allow(unused)]
#[derive(Debug, Clone)]
pub enum LoginFailReasons {
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
