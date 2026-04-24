use hibana::substrate::{
    Lane, SessionId,
    cap::advanced::{
        CAP_HANDLE_LEN, CapError, ControlOp, ControlPath, ControlScopeKind, RouteDecisionKind,
        ScopeId,
    },
    cap::{CapShot, ControlResourceKind, ResourceKind},
};

const LABEL_MGMT_LOAD_BEGIN: u8 = 110;
const LABEL_MGMT_LOAD_COMMIT: u8 = 111;
const LABEL_MGMT_ROUTE_LOAD: u8 = 112;
const LABEL_MGMT_ROUTE_ACTIVATE: u8 = 113;
const LABEL_MGMT_ROUTE_REVERT: u8 = 114;
const LABEL_MGMT_ROUTE_STATS: u8 = 115;
const LABEL_MGMT_ROUTE_LOAD_FAMILY: u8 = 116;
const LABEL_MGMT_ROUTE_LOAD_AND_ACTIVATE: u8 = 117;
const LABEL_MGMT_ROUTE_REPLY_ERROR: u8 = 118;
const LABEL_MGMT_ROUTE_REPLY_LOADED: u8 = 119;
const LABEL_MGMT_ROUTE_REPLY_ACTIVATED: u8 = 120;
const LABEL_MGMT_ROUTE_REPLY_REVERTED: u8 = 121;
const LABEL_MGMT_ROUTE_REPLY_STATS: u8 = 122;
const LABEL_MGMT_ROUTE_COMMAND_FAMILY: u8 = 123;
const LABEL_MGMT_ROUTE_COMMAND_TAIL: u8 = 124;
const LABEL_MGMT_ROUTE_REPLY_SUCCESS_FAMILY: u8 = 125;
const LABEL_MGMT_ROUTE_REPLY_SUCCESS_TAIL: u8 = 126;
const LABEL_MGMT_ROUTE_REPLY_SUCCESS_FINAL: u8 = 127;
const TAP_MGMT_LOAD_BEGIN: u16 = 0x0300 + LABEL_MGMT_LOAD_BEGIN as u16;
const TAP_MGMT_LOAD_COMMIT: u16 = 0x0300 + LABEL_MGMT_LOAD_COMMIT as u16;

type MgmtRouteHandle = (u8, u64);

fn encode_route_handle(handle: MgmtRouteHandle) -> [u8; CAP_HANDLE_LEN] {
    let mut buf = [0u8; CAP_HANDLE_LEN];
    buf[0] = handle.0;
    buf[1..9].copy_from_slice(&handle.1.to_le_bytes());
    buf
}

fn decode_route_handle(data: [u8; CAP_HANDLE_LEN]) -> MgmtRouteHandle {
    let mut scope_bytes = [0u8; 8];
    scope_bytes.copy_from_slice(&data[1..9]);
    (data[0], u64::from_le_bytes(scope_bytes))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MgmtRouteKind<const LABEL: u8, const ARM: u8>;

impl<const LABEL: u8, const ARM: u8> ResourceKind for MgmtRouteKind<LABEL, ARM> {
    type Handle = MgmtRouteHandle;
    const TAG: u8 = <RouteDecisionKind as ResourceKind>::TAG;
    const NAME: &'static str = "MgmtRoute";

    fn encode_handle(handle: &Self::Handle) -> [u8; CAP_HANDLE_LEN] {
        encode_route_handle(*handle)
    }

    fn decode_handle(data: [u8; CAP_HANDLE_LEN]) -> Result<Self::Handle, CapError> {
        Ok(decode_route_handle(data))
    }

    fn zeroize(handle: &mut Self::Handle) {
        *handle = (0, 0);
    }
}

impl<const LABEL: u8, const ARM: u8> ControlResourceKind for MgmtRouteKind<LABEL, ARM> {
    const LABEL: u8 = LABEL;
    const SCOPE: ControlScopeKind = ControlScopeKind::Route;
    const PATH: ControlPath = ControlPath::Local;
    const TAP_ID: u16 = <RouteDecisionKind as ControlResourceKind>::TAP_ID;
    const SHOT: CapShot = CapShot::One;
    const OP: ControlOp = ControlOp::RouteDecision;
    const AUTO_MINT_WIRE: bool = false;

    fn mint_handle(_session: SessionId, _lane: Lane, scope: ScopeId) -> Self::Handle {
        (ARM, scope.raw())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoadBeginKind;

impl ResourceKind for LoadBeginKind {
    type Handle = ();
    const TAG: u8 = 0x50;
    const NAME: &'static str = "LoadBegin";

    fn encode_handle(handle: &Self::Handle) -> [u8; CAP_HANDLE_LEN] {
        let _ = handle;
        [0u8; CAP_HANDLE_LEN]
    }

    fn decode_handle(data: [u8; CAP_HANDLE_LEN]) -> Result<Self::Handle, CapError> {
        let _ = data;
        Ok(())
    }

    fn zeroize(_handle: &mut Self::Handle) {}
}

impl ControlResourceKind for LoadBeginKind {
    const LABEL: u8 = LABEL_MGMT_LOAD_BEGIN;
    const SCOPE: ControlScopeKind = ControlScopeKind::Policy;
    const PATH: ControlPath = ControlPath::Wire;
    const TAP_ID: u16 = TAP_MGMT_LOAD_BEGIN;
    const SHOT: CapShot = CapShot::One;
    const OP: ControlOp = ControlOp::Fence;
    const AUTO_MINT_WIRE: bool = true;

    fn mint_handle(_session: SessionId, _lane: Lane, _scope: ScopeId) -> Self::Handle {
        ()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoadCommitKind;

impl ResourceKind for LoadCommitKind {
    type Handle = (u32, u16);
    const TAG: u8 = 0x51;
    const NAME: &'static str = "LoadCommit";

    fn encode_handle(handle: &Self::Handle) -> [u8; CAP_HANDLE_LEN] {
        let mut buf = [0u8; CAP_HANDLE_LEN];
        buf[0..4].copy_from_slice(&handle.0.to_le_bytes());
        buf[4..6].copy_from_slice(&handle.1.to_le_bytes());
        buf
    }

    fn decode_handle(data: [u8; CAP_HANDLE_LEN]) -> Result<Self::Handle, CapError> {
        Ok((
            u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            u16::from_le_bytes([data[4], data[5]]),
        ))
    }

    fn zeroize(_handle: &mut Self::Handle) {}
}

impl ControlResourceKind for LoadCommitKind {
    const LABEL: u8 = LABEL_MGMT_LOAD_COMMIT;
    const SCOPE: ControlScopeKind = ControlScopeKind::Policy;
    const PATH: ControlPath = ControlPath::Wire;
    const TAP_ID: u16 = TAP_MGMT_LOAD_COMMIT;
    const SHOT: CapShot = CapShot::One;
    const OP: ControlOp = ControlOp::TxCommit;
    const AUTO_MINT_WIRE: bool = true;

    fn mint_handle(session: SessionId, lane: Lane, _scope: ScopeId) -> Self::Handle {
        (session.raw(), lane.raw() as u16)
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
