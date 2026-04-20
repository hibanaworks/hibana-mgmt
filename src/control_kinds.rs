use hibana::substrate::{
    Lane, SessionId,
    cap::advanced::{
        CAP_HANDLE_LEN, CapError, CapsMask, ControlHandling, ControlMint, ControlScopeKind,
        RouteDecisionHandle, RouteDecisionKind, ScopeId, SessionScopedKind,
    },
    cap::{CapShot, ControlResourceKind, ResourceKind},
};

const LABEL_MGMT_LOAD_BEGIN: u8 = 40;
const LABEL_MGMT_LOAD_COMMIT: u8 = 43;
const LABEL_MGMT_ROUTE_LOAD: u8 = 64;
const LABEL_MGMT_ROUTE_ACTIVATE: u8 = 65;
const LABEL_MGMT_ROUTE_REVERT: u8 = 66;
const LABEL_MGMT_ROUTE_STATS: u8 = 67;
const LABEL_MGMT_ROUTE_LOAD_FAMILY: u8 = 68;
const LABEL_MGMT_ROUTE_LOAD_AND_ACTIVATE: u8 = 69;
const LABEL_MGMT_ROUTE_REPLY_ERROR: u8 = 70;
const LABEL_MGMT_ROUTE_REPLY_LOADED: u8 = 71;
const LABEL_MGMT_ROUTE_REPLY_ACTIVATED: u8 = 72;
const LABEL_MGMT_ROUTE_REPLY_REVERTED: u8 = 73;
const LABEL_MGMT_ROUTE_REPLY_STATS: u8 = 74;
const LABEL_MGMT_ROUTE_COMMAND_FAMILY: u8 = 75;
const LABEL_MGMT_ROUTE_COMMAND_TAIL: u8 = 76;
const LABEL_MGMT_ROUTE_REPLY_SUCCESS_FAMILY: u8 = 78;
const LABEL_MGMT_ROUTE_REPLY_SUCCESS_TAIL: u8 = 79;
const LABEL_MGMT_ROUTE_REPLY_SUCCESS_FINAL: u8 = 80;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MgmtRouteKind<const LABEL: u8, const ARM: u8>;

impl<const LABEL: u8, const ARM: u8> ResourceKind for MgmtRouteKind<LABEL, ARM> {
    type Handle = RouteDecisionHandle;
    const TAG: u8 = <RouteDecisionKind as ResourceKind>::TAG;
    const NAME: &'static str = "MgmtRoute";
    const AUTO_MINT_EXTERNAL: bool = false;

    fn encode_handle(handle: &Self::Handle) -> [u8; CAP_HANDLE_LEN] {
        handle.encode()
    }

    fn decode_handle(data: [u8; CAP_HANDLE_LEN]) -> Result<Self::Handle, CapError> {
        RouteDecisionHandle::decode(data)
    }

    fn zeroize(handle: &mut Self::Handle) {
        *handle = RouteDecisionHandle::default();
    }

    fn caps_mask(_handle: &Self::Handle) -> CapsMask {
        CapsMask::empty()
    }

    fn scope_id(handle: &Self::Handle) -> Option<ScopeId> {
        Some(handle.scope)
    }
}

impl<const LABEL: u8, const ARM: u8> SessionScopedKind for MgmtRouteKind<LABEL, ARM> {
    fn handle_for_session(_sid: SessionId, _lane: Lane) -> Self::Handle {
        RouteDecisionHandle::default()
    }

    fn shot() -> CapShot {
        CapShot::One
    }
}

impl<const LABEL: u8, const ARM: u8> ControlMint for MgmtRouteKind<LABEL, ARM> {
    fn mint_handle(_sid: SessionId, _lane: Lane, scope: ScopeId) -> Self::Handle {
        RouteDecisionHandle { scope, arm: ARM }
    }
}

impl<const LABEL: u8, const ARM: u8> ControlResourceKind for MgmtRouteKind<LABEL, ARM> {
    const LABEL: u8 = LABEL;
    const SCOPE: ControlScopeKind = ControlScopeKind::Route;
    const TAP_ID: u16 = <RouteDecisionKind as ControlResourceKind>::TAP_ID;
    const SHOT: CapShot = CapShot::One;
    const HANDLING: ControlHandling = ControlHandling::Canonical;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoadBeginKind;

impl ResourceKind for LoadBeginKind {
    type Handle = (u8, u64);
    const TAG: u8 = 0x50;
    const NAME: &'static str = "LoadBegin";
    const AUTO_MINT_EXTERNAL: bool = false;

    fn encode_handle(handle: &Self::Handle) -> [u8; CAP_HANDLE_LEN] {
        let mut buf = [0u8; CAP_HANDLE_LEN];
        buf[0] = handle.0;
        buf[1..6].copy_from_slice(&handle.1.to_le_bytes()[0..5]);
        buf
    }

    fn decode_handle(data: [u8; CAP_HANDLE_LEN]) -> Result<Self::Handle, CapError> {
        let slot = data[0];
        let mut hash_bytes = [0u8; 8];
        hash_bytes[0..5].copy_from_slice(&data[1..6]);
        Ok((slot, u64::from_le_bytes(hash_bytes)))
    }

    fn zeroize(_handle: &mut Self::Handle) {}

    fn caps_mask(_handle: &Self::Handle) -> CapsMask {
        CapsMask::empty()
    }

    fn scope_id(_handle: &Self::Handle) -> Option<ScopeId> {
        None
    }
}

impl ControlResourceKind for LoadBeginKind {
    const LABEL: u8 = LABEL_MGMT_LOAD_BEGIN;
    const SCOPE: ControlScopeKind = ControlScopeKind::Policy;
    const TAP_ID: u16 = 0;
    const SHOT: CapShot = CapShot::One;
    const HANDLING: ControlHandling = ControlHandling::External;
}

impl ControlMint for LoadBeginKind {
    fn mint_handle(_sid: SessionId, _lane: Lane, _scope: ScopeId) -> Self::Handle {
        (0, 0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoadCommitKind;

impl ResourceKind for LoadCommitKind {
    type Handle = u8;
    const TAG: u8 = 0x51;
    const NAME: &'static str = "LoadCommit";
    const AUTO_MINT_EXTERNAL: bool = false;

    fn encode_handle(handle: &Self::Handle) -> [u8; CAP_HANDLE_LEN] {
        let mut buf = [0u8; CAP_HANDLE_LEN];
        buf[0] = *handle;
        buf
    }

    fn decode_handle(data: [u8; CAP_HANDLE_LEN]) -> Result<Self::Handle, CapError> {
        Ok(data[0])
    }

    fn zeroize(_handle: &mut Self::Handle) {}

    fn caps_mask(_handle: &Self::Handle) -> CapsMask {
        CapsMask::empty()
    }

    fn scope_id(_handle: &Self::Handle) -> Option<ScopeId> {
        None
    }
}

impl ControlResourceKind for LoadCommitKind {
    const LABEL: u8 = LABEL_MGMT_LOAD_COMMIT;
    const SCOPE: ControlScopeKind = ControlScopeKind::Policy;
    const TAP_ID: u16 = 0;
    const SHOT: CapShot = CapShot::One;
    const HANDLING: ControlHandling = ControlHandling::External;
}

impl ControlMint for LoadCommitKind {
    fn mint_handle(_sid: SessionId, _lane: Lane, _scope: ScopeId) -> Self::Handle {
        0
    }
}

pub type MgmtRouteLoadKind = MgmtRouteKind<LABEL_MGMT_ROUTE_LOAD, 0>;
pub type MgmtRouteActivateKind = MgmtRouteKind<LABEL_MGMT_ROUTE_ACTIVATE, 0>;
pub type MgmtRouteRevertKind = MgmtRouteKind<LABEL_MGMT_ROUTE_REVERT, 0>;
pub type MgmtRouteStatsKind = MgmtRouteKind<LABEL_MGMT_ROUTE_STATS, 1>;
pub type MgmtRouteLoadFamilyKind = MgmtRouteKind<LABEL_MGMT_ROUTE_LOAD_FAMILY, 0>;
pub type MgmtRouteLoadAndActivateKind = MgmtRouteKind<LABEL_MGMT_ROUTE_LOAD_AND_ACTIVATE, 1>;
pub type MgmtRouteReplyErrorKind = MgmtRouteKind<LABEL_MGMT_ROUTE_REPLY_ERROR, 0>;
pub type MgmtRouteReplyLoadedKind = MgmtRouteKind<LABEL_MGMT_ROUTE_REPLY_LOADED, 0>;
pub type MgmtRouteReplyActivatedKind = MgmtRouteKind<LABEL_MGMT_ROUTE_REPLY_ACTIVATED, 0>;
pub type MgmtRouteReplyRevertedKind = MgmtRouteKind<LABEL_MGMT_ROUTE_REPLY_REVERTED, 0>;
pub type MgmtRouteReplyStatsKind = MgmtRouteKind<LABEL_MGMT_ROUTE_REPLY_STATS, 1>;
pub type MgmtRouteCommandFamilyKind = MgmtRouteKind<LABEL_MGMT_ROUTE_COMMAND_FAMILY, 1>;
pub type MgmtRouteCommandTailKind = MgmtRouteKind<LABEL_MGMT_ROUTE_COMMAND_TAIL, 1>;
pub type MgmtRouteReplySuccessFamilyKind = MgmtRouteKind<LABEL_MGMT_ROUTE_REPLY_SUCCESS_FAMILY, 1>;
pub type MgmtRouteReplySuccessTailKind = MgmtRouteKind<LABEL_MGMT_ROUTE_REPLY_SUCCESS_TAIL, 1>;
pub type MgmtRouteReplySuccessFinalKind = MgmtRouteKind<LABEL_MGMT_ROUTE_REPLY_SUCCESS_FINAL, 1>;
