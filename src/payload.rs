use hibana::{
    substrate::policy::PolicySlot,
    substrate::wire::{CodecError, Payload, WireEncode, WirePayload},
};

use hibana_epf::{host::HostError, loader::LoaderError, verifier::VerifyError};

pub(crate) const LOAD_CHUNK_MAX: usize = 1024;

#[inline]
fn require_exact_len(
    input_len: usize,
    expected: usize,
    trailing: &'static str,
) -> Result<(), CodecError> {
    if input_len < expected {
        return Err(CodecError::Truncated);
    }
    if input_len != expected {
        return Err(CodecError::Invalid(trailing));
    }
    Ok(())
}

/// Errors that can occur during the management session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MgmtError {
    InvalidSlot(u8),
    InvalidTransition,
    ChunkOutOfOrder { expected: u32, got: u32 },
    ChunkTooLarge { remaining: u32, provided: u32 },
    LoaderNotFinalised,
    NoStagedImage,
    NoActiveImage,
    NoPreviousImage,
    CapabilityMismatch,
    ObserveUnavailable,
    HostInstallFailed,
    HostUninstallFailed,
    StreamEnded,
}

impl From<LoaderError> for MgmtError {
    fn from(err: LoaderError) -> Self {
        match err {
            LoaderError::AlreadyLoading => MgmtError::InvalidTransition,
            LoaderError::NotLoading => MgmtError::InvalidTransition,
            LoaderError::CodeTooLarge { declared } => MgmtError::ChunkTooLarge {
                remaining: 0,
                provided: declared as u32,
            },
            LoaderError::UnexpectedOffset { expected, got } => {
                MgmtError::ChunkOutOfOrder { expected, got }
            }
            LoaderError::ChunkTooLarge {
                remaining,
                provided,
            } => MgmtError::ChunkTooLarge {
                remaining,
                provided,
            },
            LoaderError::HashMismatch { .. } => MgmtError::LoaderNotFinalised,
            LoaderError::Verify(err) => MgmtError::from(err),
        }
    }
}

impl From<VerifyError> for MgmtError {
    fn from(_: VerifyError) -> Self {
        MgmtError::LoaderNotFinalised
    }
}

impl From<HostError> for MgmtError {
    fn from(err: HostError) -> Self {
        match err {
            HostError::SlotOccupied => MgmtError::InvalidTransition,
            HostError::SlotEmpty => MgmtError::HostUninstallFailed,
            HostError::InvalidFuel => MgmtError::HostInstallFailed,
            HostError::ScratchTooSmall { .. } => MgmtError::HostInstallFailed,
            HostError::ScratchTooLarge { .. } => MgmtError::HostInstallFailed,
            HostError::Verify(_) => MgmtError::HostInstallFailed,
        }
    }
}

impl WireEncode for MgmtError {
    fn encoded_len(&self) -> Option<usize> {
        Some(match self {
            MgmtError::InvalidSlot(_) => 2,
            MgmtError::ChunkOutOfOrder { .. } => 9,
            MgmtError::ChunkTooLarge { .. } => 9,
            _ => 1,
        })
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        let need = self.encoded_len().unwrap_or(1);
        if out.len() < need {
            return Err(CodecError::Truncated);
        }
        match self {
            MgmtError::InvalidSlot(slot) => {
                out[0] = 0;
                out[1] = *slot;
            }
            MgmtError::InvalidTransition => out[0] = 1,
            MgmtError::ChunkOutOfOrder { expected, got } => {
                out[0] = 2;
                out[1..5].copy_from_slice(&expected.to_be_bytes());
                out[5..9].copy_from_slice(&got.to_be_bytes());
            }
            MgmtError::ChunkTooLarge {
                remaining,
                provided,
            } => {
                out[0] = 3;
                out[1..5].copy_from_slice(&remaining.to_be_bytes());
                out[5..9].copy_from_slice(&provided.to_be_bytes());
            }
            MgmtError::LoaderNotFinalised => out[0] = 4,
            MgmtError::NoStagedImage => out[0] = 5,
            MgmtError::NoActiveImage => out[0] = 6,
            MgmtError::NoPreviousImage => out[0] = 7,
            MgmtError::CapabilityMismatch => out[0] = 8,
            MgmtError::ObserveUnavailable => out[0] = 9,
            MgmtError::HostInstallFailed => out[0] = 10,
            MgmtError::HostUninstallFailed => out[0] = 11,
            MgmtError::StreamEnded => out[0] = 12,
        }
        Ok(need)
    }
}

impl WirePayload for MgmtError {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let input = input.as_bytes();
        if input.is_empty() {
            return Err(CodecError::Truncated);
        }
        let expected = match input[0] {
            0 => 2,
            1 => 1,
            2 | 3 => 9,
            4..=12 => 1,
            _ => return Err(CodecError::Invalid("unknown management error tag")),
        };
        require_exact_len(input.len(), expected, "trailing bytes after MgmtError")?;

        match input[0] {
            0 => Ok(MgmtError::InvalidSlot(input[1])),
            1 => Ok(MgmtError::InvalidTransition),
            2 => Ok(MgmtError::ChunkOutOfOrder {
                expected: u32::from_be_bytes([input[1], input[2], input[3], input[4]]),
                got: u32::from_be_bytes([input[5], input[6], input[7], input[8]]),
            }),
            3 => Ok(MgmtError::ChunkTooLarge {
                remaining: u32::from_be_bytes([input[1], input[2], input[3], input[4]]),
                provided: u32::from_be_bytes([input[5], input[6], input[7], input[8]]),
            }),
            4 => Ok(MgmtError::LoaderNotFinalised),
            5 => Ok(MgmtError::NoStagedImage),
            6 => Ok(MgmtError::NoActiveImage),
            7 => Ok(MgmtError::NoPreviousImage),
            8 => Ok(MgmtError::CapabilityMismatch),
            9 => Ok(MgmtError::ObserveUnavailable),
            10 => Ok(MgmtError::HostInstallFailed),
            11 => Ok(MgmtError::HostUninstallFailed),
            12 => Ok(MgmtError::StreamEnded),
            _ => Err(CodecError::Invalid("unknown management error tag")),
        }
    }
}

/// Typical management protocol replies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reply {
    Loaded(LoadReport),
    ActivationScheduled(TransitionReport),
    Reverted(TransitionReport),
    Stats {
        stats: StatsResp,
        staged_version: Option<u32>,
    },
}

/// One-shot report returned when an image is staged but not activated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoadReport {
    pub staged_version: u32,
}

/// Payload carried by the stats reply route.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StatsReply {
    pub stats: StatsResp,
    pub staged_version: Option<u32>,
}

/// Request payload for code upload branches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoadRequest<'a> {
    pub slot: PolicySlot,
    pub code: &'a [u8],
    pub fuel_max: u16,
    pub mem_len: u16,
}

/// Request payload for slot-scoped command branches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotRequest {
    pub slot: PolicySlot,
}

/// Management requests carried by the request/reply management session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request<'a> {
    Load(LoadRequest<'a>),
    LoadAndActivate(LoadRequest<'a>),
    Activate(SlotRequest),
    Revert(SlotRequest),
    Stats(SlotRequest),
}

/// Slot-level metrics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StatsResp {
    pub traps: u32,
    pub aborts: u32,
    pub fuel_used: u32,
    pub active_version: u32,
}

/// Policy statistics collected during transitions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TransitionReport {
    pub version: u32,
    pub policy_stats: PolicyStats,
}

/// Policy event statistics harvested from tap events.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PolicyStats {
    pub aborts: u32,
    pub traps: u32,
    pub annotations: u32,
    pub effects: u32,
    pub effects_ok: u32,
    pub commits: u32,
    pub reverts: u32,
    pub last_commit: Option<u32>,
    pub last_revert: Option<u32>,
}

/// Payload carried by the `LoadBegin` message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoadBegin {
    pub slot: PolicySlot,
    pub code_len: u32,
    pub fuel_max: u16,
    pub mem_len: u16,
    pub hash: u32,
}

/// Payload for `LoadChunk`; the chunk body is a borrowed wire view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoadChunk<'a> {
    pub offset: u32,
    bytes: &'a [u8],
}

impl<'a> LoadChunk<'a> {
    pub fn new(offset: u32, chunk: &'a [u8]) -> Self {
        assert!(
            chunk.len() <= LOAD_CHUNK_MAX,
            "chunk length exceeds management chunk capacity"
        );
        Self {
            offset,
            bytes: chunk,
        }
    }

    #[inline]
    pub fn len(&self) -> u16 {
        self.bytes.len() as u16
    }

    #[inline]
    pub fn bytes(&self) -> &[u8] {
        self.bytes
    }
}

/// Subscribe request for streaming observe.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SubscribeReq {
    pub flags: u16,
}

impl WireEncode for SubscribeReq {
    fn encoded_len(&self) -> Option<usize> {
        Some(2)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 2 {
            return Err(CodecError::Truncated);
        }
        out[0..2].copy_from_slice(&self.flags.to_be_bytes());
        Ok(2)
    }
}

impl WirePayload for SubscribeReq {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let input = input.as_bytes();
        require_exact_len(input.len(), 2, "trailing bytes after SubscribeReq")?;
        let flags = u16::from_be_bytes([input[0], input[1]]);
        Ok(SubscribeReq { flags })
    }
}

impl WireEncode for StatsResp {
    fn encoded_len(&self) -> Option<usize> {
        Some(16)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 16 {
            return Err(CodecError::Truncated);
        }
        out[0..4].copy_from_slice(&self.traps.to_be_bytes());
        out[4..8].copy_from_slice(&self.aborts.to_be_bytes());
        out[8..12].copy_from_slice(&self.fuel_used.to_be_bytes());
        out[12..16].copy_from_slice(&self.active_version.to_be_bytes());
        Ok(16)
    }
}

impl WirePayload for StatsResp {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let input = input.as_bytes();
        require_exact_len(input.len(), 16, "trailing bytes after StatsResp")?;
        Ok(StatsResp {
            traps: u32::from_be_bytes([input[0], input[1], input[2], input[3]]),
            aborts: u32::from_be_bytes([input[4], input[5], input[6], input[7]]),
            fuel_used: u32::from_be_bytes([input[8], input[9], input[10], input[11]]),
            active_version: u32::from_be_bytes([input[12], input[13], input[14], input[15]]),
        })
    }
}

impl WireEncode for PolicyStats {
    fn encoded_len(&self) -> Option<usize> {
        Some(38)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 38 {
            return Err(CodecError::Truncated);
        }
        out[0..4].copy_from_slice(&self.aborts.to_be_bytes());
        out[4..8].copy_from_slice(&self.traps.to_be_bytes());
        out[8..12].copy_from_slice(&self.annotations.to_be_bytes());
        out[12..16].copy_from_slice(&self.effects.to_be_bytes());
        out[16..20].copy_from_slice(&self.effects_ok.to_be_bytes());
        out[20..24].copy_from_slice(&self.commits.to_be_bytes());
        out[24..28].copy_from_slice(&self.reverts.to_be_bytes());
        out[28..32].copy_from_slice(&self.last_commit.unwrap_or(0).to_be_bytes());
        out[32] = u8::from(self.last_commit.is_some());
        out[33..37].copy_from_slice(&self.last_revert.unwrap_or(0).to_be_bytes());
        out[37] = u8::from(self.last_revert.is_some());
        Ok(38)
    }
}

impl WirePayload for PolicyStats {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let input = input.as_bytes();
        require_exact_len(input.len(), 38, "trailing bytes after PolicyStats")?;
        let last_commit = u32::from_be_bytes([input[28], input[29], input[30], input[31]]);
        let last_revert = u32::from_be_bytes([input[33], input[34], input[35], input[36]]);
        Ok(PolicyStats {
            aborts: u32::from_be_bytes([input[0], input[1], input[2], input[3]]),
            traps: u32::from_be_bytes([input[4], input[5], input[6], input[7]]),
            annotations: u32::from_be_bytes([input[8], input[9], input[10], input[11]]),
            effects: u32::from_be_bytes([input[12], input[13], input[14], input[15]]),
            effects_ok: u32::from_be_bytes([input[16], input[17], input[18], input[19]]),
            commits: u32::from_be_bytes([input[20], input[21], input[22], input[23]]),
            reverts: u32::from_be_bytes([input[24], input[25], input[26], input[27]]),
            last_commit: if input[32] == 0 {
                None
            } else {
                Some(last_commit)
            },
            last_revert: if input[37] == 0 {
                None
            } else {
                Some(last_revert)
            },
        })
    }
}

impl WireEncode for TransitionReport {
    fn encoded_len(&self) -> Option<usize> {
        Some(42)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 42 {
            return Err(CodecError::Truncated);
        }
        out[0..4].copy_from_slice(&self.version.to_be_bytes());
        self.policy_stats.encode_into(&mut out[4..])?;
        Ok(42)
    }
}

impl WirePayload for TransitionReport {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let input = input.as_bytes();
        require_exact_len(input.len(), 42, "trailing bytes after TransitionReport")?;
        Ok(TransitionReport {
            version: u32::from_be_bytes([input[0], input[1], input[2], input[3]]),
            policy_stats: PolicyStats::decode_payload(Payload::new(&input[4..42]))?,
        })
    }
}

impl WireEncode for LoadReport {
    fn encoded_len(&self) -> Option<usize> {
        Some(4)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 4 {
            return Err(CodecError::Truncated);
        }
        out[0..4].copy_from_slice(&self.staged_version.to_be_bytes());
        Ok(4)
    }
}

impl WirePayload for LoadReport {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let input = input.as_bytes();
        require_exact_len(input.len(), 4, "trailing bytes after LoadReport")?;
        Ok(LoadReport {
            staged_version: u32::from_be_bytes([input[0], input[1], input[2], input[3]]),
        })
    }
}

impl WireEncode for StatsReply {
    fn encoded_len(&self) -> Option<usize> {
        Some(21)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 21 {
            return Err(CodecError::Truncated);
        }
        self.stats.encode_into(out)?;
        out[16] = u8::from(self.staged_version.is_some());
        out[17..21].copy_from_slice(&self.staged_version.unwrap_or(0).to_be_bytes());
        Ok(21)
    }
}

impl WirePayload for StatsReply {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let input = input.as_bytes();
        require_exact_len(input.len(), 21, "trailing bytes after StatsReply")?;
        let stats = StatsResp::decode_payload(Payload::new(&input[..16]))?;
        let staged_version = if input[16] == 0 {
            None
        } else {
            Some(u32::from_be_bytes([
                input[17], input[18], input[19], input[20],
            ]))
        };
        Ok(StatsReply {
            stats,
            staged_version,
        })
    }
}

impl WireEncode for LoadBegin {
    fn encoded_len(&self) -> Option<usize> {
        Some(13)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.len() < 13 {
            return Err(CodecError::Truncated);
        }
        out[0] = slot_id(self.slot) as u8;
        out[1..5].copy_from_slice(&self.code_len.to_be_bytes());
        out[5..7].copy_from_slice(&self.fuel_max.to_be_bytes());
        out[7..9].copy_from_slice(&self.mem_len.to_be_bytes());
        out[9..13].copy_from_slice(&self.hash.to_be_bytes());
        Ok(13)
    }
}

impl WirePayload for LoadBegin {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let input = input.as_bytes();
        require_exact_len(input.len(), 13, "trailing bytes after LoadBegin")?;
        let slot = decode_slot(input[0])?;
        let code_len = u32::from_be_bytes([input[1], input[2], input[3], input[4]]);
        let fuel_max = u16::from_be_bytes([input[5], input[6]]);
        let mem_len = u16::from_be_bytes([input[7], input[8]]);
        let hash = u32::from_be_bytes([input[9], input[10], input[11], input[12]]);
        Ok(LoadBegin {
            slot,
            code_len,
            fuel_max,
            mem_len,
            hash,
        })
    }
}

impl WireEncode for LoadChunk<'_> {
    fn encoded_len(&self) -> Option<usize> {
        Some(6 + self.bytes.len())
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        let len = self.bytes.len();
        if len > LOAD_CHUNK_MAX {
            return Err(CodecError::Invalid("chunk length exceeds LOAD_CHUNK_MAX"));
        }
        let total = 6 + len;
        if out.len() < total {
            return Err(CodecError::Truncated);
        }
        out[..4].copy_from_slice(&self.offset.to_be_bytes());
        out[4..6].copy_from_slice(&(len as u16).to_be_bytes());
        out[6..total].copy_from_slice(self.bytes);
        Ok(total)
    }
}

impl WirePayload for LoadChunk<'static> {
    type Decoded<'a> = LoadChunk<'a>;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let input = input.as_bytes();
        if input.len() < 6 {
            return Err(CodecError::Truncated);
        }
        let offset = u32::from_be_bytes([input[0], input[1], input[2], input[3]]);
        let len = u16::from_be_bytes([input[4], input[5]]);
        let len_usize = len as usize;
        if len_usize > LOAD_CHUNK_MAX {
            return Err(CodecError::Invalid("chunk length exceeds LOAD_CHUNK_MAX"));
        }
        require_exact_len(input.len(), 6 + len_usize, "trailing bytes after LoadChunk")?;
        Ok(LoadChunk {
            offset,
            bytes: &input[6..6 + len_usize],
        })
    }
}

impl WireEncode for SlotRequest {
    fn encoded_len(&self) -> Option<usize> {
        Some(1)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if out.is_empty() {
            return Err(CodecError::Truncated);
        }
        out[0] = slot_id(self.slot) as u8;
        Ok(1)
    }
}

impl WirePayload for SlotRequest {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let input = input.as_bytes();
        require_exact_len(input.len(), 1, "trailing bytes after SlotRequest")?;
        Ok(SlotRequest {
            slot: decode_slot(input[0])?,
        })
    }
}

pub(crate) fn slot_id(slot: PolicySlot) -> u32 {
    match slot {
        PolicySlot::Forward => 0,
        PolicySlot::EndpointRx => 1,
        PolicySlot::EndpointTx => 2,
        PolicySlot::Rendezvous => 3,
        PolicySlot::Route => 4,
    }
}

pub(crate) fn decode_slot(slot: u8) -> Result<PolicySlot, CodecError> {
    match slot {
        0 => Ok(PolicySlot::Forward),
        1 => Ok(PolicySlot::EndpointRx),
        2 => Ok(PolicySlot::EndpointTx),
        3 => Ok(PolicySlot::Rendezvous),
        4 => Ok(PolicySlot::Route),
        _ => Err(CodecError::Invalid("unknown management slot")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_chunk_decode_is_borrowed_not_fixed_buffer_copy() {
        let mut encoded = [0u8; 16];
        let chunk = LoadChunk::new(7, &[1, 2, 3, 4]);
        let len = chunk.encode_into(&mut encoded).expect("encode load chunk");
        let decoded =
            <LoadChunk<'static> as WirePayload>::decode_payload(Payload::new(&encoded[..len]))
                .expect("decode load chunk");

        assert_eq!(decoded.offset, 7);
        assert_eq!(decoded.len(), 4);
        assert_eq!(decoded.bytes(), &[1, 2, 3, 4]);
    }

    #[test]
    fn load_chunk_decode_rejects_trailing_bytes() {
        let encoded = [0, 0, 0, 7, 0, 4, 1, 2, 3, 4, 0xEE];

        assert!(
            <LoadChunk<'static> as WirePayload>::decode_payload(Payload::new(&encoded)).is_err(),
            "LoadChunk decode must be canonical and reject trailing bytes"
        );
    }

    #[test]
    fn fixed_management_payload_decoders_accept_exact_lengths() {
        let mgmt_error = [1];
        assert_eq!(
            MgmtError::decode_payload(Payload::new(&mgmt_error)).expect("decode MgmtError"),
            MgmtError::InvalidTransition
        );

        let invalid_slot = [0, 4];
        assert_eq!(
            MgmtError::decode_payload(Payload::new(&invalid_slot)).expect("decode InvalidSlot"),
            MgmtError::InvalidSlot(4)
        );

        let subscribe = [0, 1];
        assert_eq!(
            SubscribeReq::decode_payload(Payload::new(&subscribe)).expect("decode SubscribeReq"),
            SubscribeReq { flags: 1 }
        );

        let stats_resp = [0u8; 16];
        assert!(StatsResp::decode_payload(Payload::new(&stats_resp)).is_ok());

        let policy_stats = [0u8; 38];
        assert!(PolicyStats::decode_payload(Payload::new(&policy_stats)).is_ok());

        let transition_report = [0u8; 42];
        assert!(TransitionReport::decode_payload(Payload::new(&transition_report)).is_ok());

        let load_report = [0u8; 4];
        assert!(LoadReport::decode_payload(Payload::new(&load_report)).is_ok());

        let stats_reply = [0u8; 21];
        assert!(StatsReply::decode_payload(Payload::new(&stats_reply)).is_ok());

        let load_begin = [0u8; 13];
        assert!(LoadBegin::decode_payload(Payload::new(&load_begin)).is_ok());

        let slot_request = [0];
        assert!(SlotRequest::decode_payload(Payload::new(&slot_request)).is_ok());
    }

    #[test]
    fn fixed_management_payload_decoders_reject_trailing_bytes() {
        let mgmt_error = [1, 0xEE];
        assert!(
            MgmtError::decode_payload(Payload::new(&mgmt_error)).is_err(),
            "single-byte MgmtError must reject trailing bytes"
        );

        let invalid_slot = [0, 4, 0xEE];
        assert!(
            MgmtError::decode_payload(Payload::new(&invalid_slot)).is_err(),
            "InvalidSlot MgmtError must reject trailing bytes"
        );

        let chunk_error = [2, 0, 0, 0, 1, 0, 0, 0, 2, 0xEE];
        assert!(
            MgmtError::decode_payload(Payload::new(&chunk_error)).is_err(),
            "wide MgmtError must reject trailing bytes"
        );

        let subscribe = [0, 1, 0xEE];
        assert!(
            SubscribeReq::decode_payload(Payload::new(&subscribe)).is_err(),
            "SubscribeReq must reject trailing bytes"
        );

        let stats_resp = [0u8; 17];
        assert!(
            StatsResp::decode_payload(Payload::new(&stats_resp)).is_err(),
            "StatsResp must reject trailing bytes"
        );

        let policy_stats = [0u8; 39];
        assert!(
            PolicyStats::decode_payload(Payload::new(&policy_stats)).is_err(),
            "PolicyStats must reject trailing bytes"
        );

        let transition_report = [0u8; 43];
        assert!(
            TransitionReport::decode_payload(Payload::new(&transition_report)).is_err(),
            "TransitionReport must reject trailing bytes"
        );

        let load_report = [0u8; 5];
        assert!(
            LoadReport::decode_payload(Payload::new(&load_report)).is_err(),
            "LoadReport must reject trailing bytes"
        );

        let stats_reply = [0u8; 22];
        assert!(
            StatsReply::decode_payload(Payload::new(&stats_reply)).is_err(),
            "StatsReply must reject trailing bytes"
        );

        let load_begin = [0u8; 14];
        assert!(
            LoadBegin::decode_payload(Payload::new(&load_begin)).is_err(),
            "LoadBegin must reject trailing bytes"
        );

        let slot_request = [0, 0xEE];
        assert!(
            SlotRequest::decode_payload(Payload::new(&slot_request)).is_err(),
            "SlotRequest must reject trailing bytes"
        );
    }
}
