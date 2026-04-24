use hibana::{
    g::advanced::{RoleProgram, project},
    g::{self},
    substrate::{
        AttachError, RendezvousId, SessionId, SessionKit, Transport,
        binding::NoBinding,
        cap::GenericCapToken,
        cap::advanced::{LoopBreakKind, LoopContinueKind},
        runtime::{Clock, LabelUniverse},
        tap::TapEvent,
        wire::{CodecError, Payload, WireEncode, WirePayload},
    },
};

use super::{
    payload::SubscribeReq,
    request_reply::{ROLE_CLUSTER, ROLE_CONTROLLER},
};

const TAP_BATCH_HEADER_LEN: usize = 5;
const TAP_EVENT_WIRE_LEN: usize = 20;
const TAP_BATCH_MAX_EVENTS: usize = 50;
const STREAM_LOOP_POLICY_ID: u16 = 701;
const LABEL_LOOP_CONTINUE: u8 = 48;
const LABEL_LOOP_BREAK: u8 = 49;
const LABEL_OBSERVE_BATCH: u8 = 38;
const LABEL_OBSERVE_STREAM_END: u8 = 39;
const LABEL_OBSERVE_SUBSCRIBE: u8 = 45;

/// Internal batched observe-stream payload. Public substrate surface stays on
/// `tap::TapEvent`; batching is a lower-layer stream detail.
#[derive(Clone, Copy, Debug)]
pub struct TapBatch {
    events: [TapEvent; TAP_BATCH_MAX_EVENTS],
    count: u8,
    lost_events: u32,
}

impl Default for TapBatch {
    fn default() -> Self {
        Self::empty()
    }
}

impl TapBatch {
    #[inline]
    const fn empty() -> Self {
        Self {
            events: [TapEvent::zero(); TAP_BATCH_MAX_EVENTS],
            count: 0,
            lost_events: 0,
        }
    }

    #[inline]
    fn push(&mut self, event: TapEvent) -> bool {
        if (self.count as usize) < TAP_BATCH_MAX_EVENTS {
            self.events[self.count as usize] = event;
            self.count += 1;
            true
        } else {
            false
        }
    }

    #[inline]
    const fn len(&self) -> usize {
        self.count as usize
    }

    #[inline]
    const fn lost_events(&self) -> u32 {
        self.lost_events
    }

    #[inline]
    fn set_lost_events(&mut self, lost: u32) {
        self.lost_events = lost;
    }

    #[inline]
    fn iter(&self) -> impl Iterator<Item = &TapEvent> {
        self.events[..self.count as usize].iter()
    }
}

impl WireEncode for TapBatch {
    fn encoded_len(&self) -> Option<usize> {
        Some(TAP_BATCH_HEADER_LEN + self.len() * TAP_EVENT_WIRE_LEN)
    }

    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        let total_len = TAP_BATCH_HEADER_LEN + self.len() * TAP_EVENT_WIRE_LEN;
        if out.len() < total_len {
            return Err(CodecError::Truncated);
        }

        out[0] = self.len() as u8;
        out[1..5].copy_from_slice(&self.lost_events().to_be_bytes());

        let mut offset = TAP_BATCH_HEADER_LEN;
        for event in self.iter() {
            event.encode_into(&mut out[offset..])?;
            offset += TAP_EVENT_WIRE_LEN;
        }

        Ok(total_len)
    }
}

impl WirePayload for TapBatch {
    type Decoded<'a> = Self;

    fn decode_payload<'a>(input: Payload<'a>) -> Result<Self::Decoded<'a>, CodecError> {
        let input = input.as_bytes();
        if input.len() < TAP_BATCH_HEADER_LEN {
            return Err(CodecError::Truncated);
        }

        let count = input[0] as usize;
        if count > TAP_BATCH_MAX_EVENTS {
            return Err(CodecError::Invalid("batch count exceeds maximum"));
        }

        let lost_events = u32::from_be_bytes([input[1], input[2], input[3], input[4]]);
        let expected_len = TAP_BATCH_HEADER_LEN + count * TAP_EVENT_WIRE_LEN;
        if input.len() < expected_len {
            return Err(CodecError::Truncated);
        }
        if input.len() != expected_len {
            return Err(CodecError::Invalid("trailing bytes after TapBatch"));
        }

        let mut batch = TapBatch::empty();
        batch.set_lost_events(lost_events);

        let mut offset = TAP_BATCH_HEADER_LEN;
        for _ in 0..count {
            let event = TapEvent::decode_payload(Payload::new(
                &input[offset..offset + TAP_EVENT_WIRE_LEN],
            ))?;
            batch.push(event);
            offset += TAP_EVENT_WIRE_LEN;
        }

        Ok(batch)
    }
}

fn controller_program() -> RoleProgram<ROLE_CONTROLLER> {
    let stream_subscribe = g::send::<
        g::Role<ROLE_CONTROLLER>,
        g::Role<ROLE_CLUSTER>,
        g::Msg<LABEL_OBSERVE_SUBSCRIBE, SubscribeReq>,
        0,
    >();
    let stream_loop_continue_arm = g::seq(
        g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CLUSTER>,
            g::Msg<LABEL_LOOP_CONTINUE, GenericCapToken<LoopContinueKind>, LoopContinueKind>,
            0,
        >()
        .policy::<STREAM_LOOP_POLICY_ID>(),
        g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CONTROLLER>,
            g::Msg<LABEL_OBSERVE_BATCH, TapBatch>,
            0,
        >(),
    );
    let stream_loop_break_arm = g::seq(
        g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CLUSTER>,
            g::Msg<LABEL_LOOP_BREAK, GenericCapToken<LoopBreakKind>, LoopBreakKind>,
            0,
        >()
        .policy::<STREAM_LOOP_POLICY_ID>(),
        g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CONTROLLER>,
            g::Msg<LABEL_OBSERVE_STREAM_END, ()>,
            0,
        >(),
    );
    let program = g::seq(
        stream_subscribe,
        g::route(stream_loop_continue_arm, stream_loop_break_arm),
    );
    let projected: RoleProgram<ROLE_CONTROLLER> = project(&program);
    projected
}

fn cluster_program() -> RoleProgram<ROLE_CLUSTER> {
    let stream_subscribe = g::send::<
        g::Role<ROLE_CONTROLLER>,
        g::Role<ROLE_CLUSTER>,
        g::Msg<LABEL_OBSERVE_SUBSCRIBE, SubscribeReq>,
        0,
    >();
    let stream_loop_continue_arm = g::seq(
        g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CLUSTER>,
            g::Msg<LABEL_LOOP_CONTINUE, GenericCapToken<LoopContinueKind>, LoopContinueKind>,
            0,
        >()
        .policy::<STREAM_LOOP_POLICY_ID>(),
        g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CONTROLLER>,
            g::Msg<LABEL_OBSERVE_BATCH, TapBatch>,
            0,
        >(),
    );
    let stream_loop_break_arm = g::seq(
        g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CLUSTER>,
            g::Msg<LABEL_LOOP_BREAK, GenericCapToken<LoopBreakKind>, LoopBreakKind>,
            0,
        >()
        .policy::<STREAM_LOOP_POLICY_ID>(),
        g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CONTROLLER>,
            g::Msg<LABEL_OBSERVE_STREAM_END, ()>,
            0,
        >(),
    );
    let program = g::seq(
        stream_subscribe,
        g::route(stream_loop_continue_arm, stream_loop_break_arm),
    );
    let projected: RoleProgram<ROLE_CLUSTER> = project(&program);
    projected
}

#[allow(private_bounds)]
pub fn attach_controller<'r, 'cfg, T, U, C, const MAX_RV: usize>(
    kit: &'r SessionKit<'cfg, T, U, C, MAX_RV>,
    rv: RendezvousId,
    sid: SessionId,
) -> Result<hibana::Endpoint<'r, ROLE_CONTROLLER>, AttachError>
where
    T: Transport + 'cfg,
    U: LabelUniverse + 'cfg,
    C: Clock + 'cfg,
    'cfg: 'r,
{
    let program = controller_program();
    kit.enter(rv, sid, &program, NoBinding)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tap_batch_decode_is_canonical() {
        let mut batch = TapBatch::empty();
        assert!(batch.push(TapEvent::zero()));

        let mut encoded = [0u8; TAP_BATCH_HEADER_LEN + TAP_EVENT_WIRE_LEN];
        let len = batch.encode_into(&mut encoded).expect("encode tap batch");
        let decoded = TapBatch::decode_payload(Payload::new(&encoded[..len]))
            .expect("decode exact tap batch");

        assert_eq!(decoded.len(), 1);
    }

    #[test]
    fn tap_batch_decode_rejects_trailing_bytes() {
        let batch = TapBatch::empty();
        let mut encoded = [0u8; TAP_BATCH_HEADER_LEN + 1];
        let len = batch.encode_into(&mut encoded).expect("encode tap batch");

        assert!(
            TapBatch::decode_payload(Payload::new(&encoded[..len + 1])).is_err(),
            "TapBatch decode must reject trailing bytes"
        );
    }
}

#[allow(private_bounds)]
pub fn attach_cluster<'r, 'cfg, T, U, C, const MAX_RV: usize>(
    kit: &'r SessionKit<'cfg, T, U, C, MAX_RV>,
    rv: RendezvousId,
    sid: SessionId,
) -> Result<hibana::Endpoint<'r, ROLE_CLUSTER>, AttachError>
where
    T: Transport + 'cfg,
    U: LabelUniverse + 'cfg,
    C: Clock + 'cfg,
    'cfg: 'r,
{
    let program = cluster_program();
    kit.enter(rv, sid, &program, NoBinding)
}
