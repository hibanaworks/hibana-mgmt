use hibana::{
    g::advanced::{CanonicalControl, ExternalControl, RoleProgram, project},
    g::{self},
    substrate::{
        AttachError, RendezvousId, SessionId, SessionKit, Transport,
        binding::NoBinding,
        cap::GenericCapToken,
        runtime::{Clock, LabelUniverse},
    },
};

use super::{
    control_kinds::{
        LoadBeginKind, LoadCommitKind, MgmtRouteActivateKind, MgmtRouteCommandFamilyKind,
        MgmtRouteCommandTailKind, MgmtRouteLoadAndActivateKind, MgmtRouteLoadFamilyKind,
        MgmtRouteLoadKind, MgmtRouteReplyActivatedKind, MgmtRouteReplyErrorKind,
        MgmtRouteReplyLoadedKind, MgmtRouteReplyRevertedKind, MgmtRouteReplyStatsKind,
        MgmtRouteReplySuccessFamilyKind, MgmtRouteReplySuccessFinalKind,
        MgmtRouteReplySuccessTailKind, MgmtRouteRevertKind, MgmtRouteStatsKind,
    },
    payload::{
        LoadBegin, LoadChunk, LoadReport, MgmtError, SlotRequest, StatsReply, TransitionReport,
    },
};

pub const ROLE_CONTROLLER: u8 = 0;
pub const ROLE_CLUSTER: u8 = 1;

const LOOP_POLICY_ID: u16 = 700;
const REQUEST_ROOT_POLICY_ID: u16 = 701;
const REQUEST_LOAD_POLICY_ID: u16 = 702;
const REQUEST_COMMAND_POLICY_ID: u16 = 703;
const REQUEST_COMMAND_TAIL_POLICY_ID: u16 = 704;
const REPLY_ROOT_POLICY_ID: u16 = 705;
const REPLY_SUCCESS_POLICY_ID: u16 = 706;
const REPLY_SUCCESS_TAIL_POLICY_ID: u16 = 707;
const REPLY_SUCCESS_FINAL_POLICY_ID: u16 = 708;

const LABEL_LOOP_CONTINUE: u8 = 48;
const LABEL_LOOP_BREAK: u8 = 49;
const LABEL_MGMT_REPLY_ERROR: u8 = 30;
const LABEL_MGMT_REPLY_LOADED: u8 = 31;
const LABEL_MGMT_REPLY_ACTIVATED: u8 = 32;
const LABEL_MGMT_REPLY_REVERTED: u8 = 33;
const LABEL_MGMT_REPLY_STATS: u8 = 34;
const LABEL_MGMT_ACTIVATE: u8 = 35;
const LABEL_MGMT_REVERT: u8 = 36;
const LABEL_MGMT_STATS: u8 = 37;
const LABEL_MGMT_LOAD_BEGIN: u8 = 40;
const LABEL_MGMT_LOAD_CHUNK: u8 = 42;
const LABEL_MGMT_LOAD_COMMIT: u8 = 43;
const LABEL_MGMT_STAGE: u8 = 44;
const LABEL_MGMT_LOAD_AND_ACTIVATE: u8 = 46;
const LABEL_MGMT_LOAD_FINAL_CHUNK: u8 = 77;
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

type RouteMsg<const LABEL: u8, Kind> = g::Msg<LABEL, GenericCapToken<Kind>, CanonicalControl<Kind>>;

fn controller_program() -> RoleProgram<ROLE_CONTROLLER> {
    let load_begin_token = || {
        g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CLUSTER>,
            g::Msg<
                LABEL_MGMT_LOAD_BEGIN,
                GenericCapToken<LoadBeginKind>,
                ExternalControl<LoadBeginKind>,
            >,
            0,
        >()
    };

    let load_commit_token = || {
        g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CLUSTER>,
            g::Msg<
                LABEL_MGMT_LOAD_COMMIT,
                GenericCapToken<LoadCommitKind>,
                ExternalControl<LoadCommitKind>,
            >,
            0,
        >()
    };

    let load_stream_loop = || {
        let continue_arm = g::seq(
            g::send::<
                g::Role<ROLE_CONTROLLER>,
                g::Role<ROLE_CONTROLLER>,
                g::Msg<
                    LABEL_LOOP_CONTINUE,
                    GenericCapToken<hibana::substrate::cap::advanced::LoopContinueKind>,
                    CanonicalControl<hibana::substrate::cap::advanced::LoopContinueKind>,
                >,
                0,
            >()
            .policy::<LOOP_POLICY_ID>(),
            g::send::<
                g::Role<ROLE_CONTROLLER>,
                g::Role<ROLE_CLUSTER>,
                g::Msg<LABEL_MGMT_LOAD_CHUNK, LoadChunk>,
                0,
            >(),
        );
        let break_arm = g::seq(
            g::send::<
                g::Role<ROLE_CONTROLLER>,
                g::Role<ROLE_CONTROLLER>,
                g::Msg<
                    LABEL_LOOP_BREAK,
                    GenericCapToken<hibana::substrate::cap::advanced::LoopBreakKind>,
                    CanonicalControl<hibana::substrate::cap::advanced::LoopBreakKind>,
                >,
                0,
            >()
            .policy::<LOOP_POLICY_ID>(),
            g::send::<
                g::Role<ROLE_CONTROLLER>,
                g::Role<ROLE_CLUSTER>,
                g::Msg<LABEL_MGMT_LOAD_FINAL_CHUNK, LoadChunk>,
                0,
            >(),
        );
        g::route(continue_arm, break_arm)
    };

    let load_stream = || {
        g::seq(
            g::seq(load_begin_token(), load_stream_loop()),
            load_commit_token(),
        )
    };

    let load_request = || {
        let arm_head = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CONTROLLER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_LOAD }, MgmtRouteLoadKind>,
            0,
        >()
        .policy::<REQUEST_LOAD_POLICY_ID>();
        let arm_body = g::seq(
            g::send::<
                g::Role<ROLE_CONTROLLER>,
                g::Role<ROLE_CLUSTER>,
                g::Msg<LABEL_MGMT_STAGE, LoadBegin>,
                0,
            >(),
            load_stream(),
        );
        g::seq(arm_head, arm_body)
    };

    let load_and_activate_request = || {
        let arm_head = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CONTROLLER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_LOAD_AND_ACTIVATE }, MgmtRouteLoadAndActivateKind>,
            0,
        >()
        .policy::<REQUEST_LOAD_POLICY_ID>();
        let arm_body = g::seq(
            g::send::<
                g::Role<ROLE_CONTROLLER>,
                g::Role<ROLE_CLUSTER>,
                g::Msg<LABEL_MGMT_LOAD_AND_ACTIVATE, LoadBegin>,
                0,
            >(),
            load_stream(),
        );
        g::seq(arm_head, arm_body)
    };

    let load_family = || {
        let family_head = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CONTROLLER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_LOAD_FAMILY }, MgmtRouteLoadFamilyKind>,
            0,
        >()
        .policy::<REQUEST_ROOT_POLICY_ID>();
        let family_route = g::route(load_request(), load_and_activate_request());
        g::seq(family_head, family_route)
    };

    let activate_request = || {
        let arm_head = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CONTROLLER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_ACTIVATE }, MgmtRouteActivateKind>,
            0,
        >()
        .policy::<REQUEST_COMMAND_POLICY_ID>();
        let arm_body = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CLUSTER>,
            g::Msg<LABEL_MGMT_ACTIVATE, SlotRequest>,
            0,
        >();
        g::seq(arm_head, arm_body)
    };

    let revert_request = || {
        let arm_head = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CONTROLLER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_REVERT }, MgmtRouteRevertKind>,
            0,
        >()
        .policy::<REQUEST_COMMAND_TAIL_POLICY_ID>();
        let arm_body = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CLUSTER>,
            g::Msg<LABEL_MGMT_REVERT, SlotRequest>,
            0,
        >();
        g::seq(arm_head, arm_body)
    };

    let stats_request = || {
        let arm_head = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CONTROLLER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_STATS }, MgmtRouteStatsKind>,
            0,
        >()
        .policy::<REQUEST_COMMAND_TAIL_POLICY_ID>();
        let arm_body = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CLUSTER>,
            g::Msg<LABEL_MGMT_STATS, SlotRequest>,
            0,
        >();
        g::seq(arm_head, arm_body)
    };

    let command_tail = || {
        let tail_head = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CONTROLLER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_COMMAND_TAIL }, MgmtRouteCommandTailKind>,
            0,
        >()
        .policy::<REQUEST_COMMAND_POLICY_ID>();
        let tail_route = g::route(revert_request(), stats_request());
        g::seq(tail_head, tail_route)
    };

    let command_family = || {
        let family_head = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CONTROLLER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_COMMAND_FAMILY }, MgmtRouteCommandFamilyKind>,
            0,
        >()
        .policy::<REQUEST_ROOT_POLICY_ID>();
        let family_route = g::route(activate_request(), command_tail());
        g::seq(family_head, family_route)
    };

    let request_route = g::route(load_family(), command_family());

    let error_reply = || {
        let arm_head = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CLUSTER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_REPLY_ERROR }, MgmtRouteReplyErrorKind>,
            0,
        >()
        .policy::<REPLY_ROOT_POLICY_ID>();
        let arm_body = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CONTROLLER>,
            g::Msg<LABEL_MGMT_REPLY_ERROR, MgmtError>,
            0,
        >();
        g::seq(arm_head, arm_body)
    };

    let loaded_reply = || {
        let arm_head = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CLUSTER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_REPLY_LOADED }, MgmtRouteReplyLoadedKind>,
            0,
        >()
        .policy::<REPLY_SUCCESS_POLICY_ID>();
        let arm_body = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CONTROLLER>,
            g::Msg<LABEL_MGMT_REPLY_LOADED, LoadReport>,
            0,
        >();
        g::seq(arm_head, arm_body)
    };

    let activated_reply = || {
        let arm_head = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CLUSTER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_REPLY_ACTIVATED }, MgmtRouteReplyActivatedKind>,
            0,
        >()
        .policy::<REPLY_SUCCESS_TAIL_POLICY_ID>();
        let arm_body = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CONTROLLER>,
            g::Msg<LABEL_MGMT_REPLY_ACTIVATED, TransitionReport>,
            0,
        >();
        g::seq(arm_head, arm_body)
    };

    let reverted_reply = || {
        let arm_head = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CLUSTER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_REPLY_REVERTED }, MgmtRouteReplyRevertedKind>,
            0,
        >()
        .policy::<REPLY_SUCCESS_FINAL_POLICY_ID>();
        let arm_body = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CONTROLLER>,
            g::Msg<LABEL_MGMT_REPLY_REVERTED, TransitionReport>,
            0,
        >();
        g::seq(arm_head, arm_body)
    };

    let stats_reply = || {
        let arm_head = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CLUSTER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_REPLY_STATS }, MgmtRouteReplyStatsKind>,
            0,
        >()
        .policy::<REPLY_SUCCESS_FINAL_POLICY_ID>();
        let arm_body = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CONTROLLER>,
            g::Msg<LABEL_MGMT_REPLY_STATS, StatsReply>,
            0,
        >();
        g::seq(arm_head, arm_body)
    };

    let success_final_reply = || {
        let family_head = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CLUSTER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_REPLY_SUCCESS_FINAL }, MgmtRouteReplySuccessFinalKind>,
            0,
        >()
        .policy::<REPLY_SUCCESS_TAIL_POLICY_ID>();
        let family_route = g::route(reverted_reply(), stats_reply());
        g::seq(family_head, family_route)
    };

    let success_tail_reply = || {
        let family_head = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CLUSTER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_REPLY_SUCCESS_TAIL }, MgmtRouteReplySuccessTailKind>,
            0,
        >()
        .policy::<REPLY_SUCCESS_POLICY_ID>();
        let family_route = g::route(activated_reply(), success_final_reply());
        g::seq(family_head, family_route)
    };

    let success_reply = || {
        let family_head = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CLUSTER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_REPLY_SUCCESS_FAMILY }, MgmtRouteReplySuccessFamilyKind>,
            0,
        >()
        .policy::<REPLY_ROOT_POLICY_ID>();
        let family_route = g::route(loaded_reply(), success_tail_reply());
        g::seq(family_head, family_route)
    };

    let reply_route = g::route(error_reply(), success_reply());
    let program = g::seq(request_route, reply_route);
    let projected: RoleProgram<ROLE_CONTROLLER> = project(&program);
    projected
}

fn cluster_program() -> RoleProgram<ROLE_CLUSTER> {
    let load_begin_token = || {
        g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CLUSTER>,
            g::Msg<
                LABEL_MGMT_LOAD_BEGIN,
                GenericCapToken<LoadBeginKind>,
                ExternalControl<LoadBeginKind>,
            >,
            0,
        >()
    };

    let load_commit_token = || {
        g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CLUSTER>,
            g::Msg<
                LABEL_MGMT_LOAD_COMMIT,
                GenericCapToken<LoadCommitKind>,
                ExternalControl<LoadCommitKind>,
            >,
            0,
        >()
    };

    let load_stream_loop = || {
        let continue_arm = g::seq(
            g::send::<
                g::Role<ROLE_CONTROLLER>,
                g::Role<ROLE_CONTROLLER>,
                g::Msg<
                    LABEL_LOOP_CONTINUE,
                    GenericCapToken<hibana::substrate::cap::advanced::LoopContinueKind>,
                    CanonicalControl<hibana::substrate::cap::advanced::LoopContinueKind>,
                >,
                0,
            >()
            .policy::<LOOP_POLICY_ID>(),
            g::send::<
                g::Role<ROLE_CONTROLLER>,
                g::Role<ROLE_CLUSTER>,
                g::Msg<LABEL_MGMT_LOAD_CHUNK, LoadChunk>,
                0,
            >(),
        );
        let break_arm = g::seq(
            g::send::<
                g::Role<ROLE_CONTROLLER>,
                g::Role<ROLE_CONTROLLER>,
                g::Msg<
                    LABEL_LOOP_BREAK,
                    GenericCapToken<hibana::substrate::cap::advanced::LoopBreakKind>,
                    CanonicalControl<hibana::substrate::cap::advanced::LoopBreakKind>,
                >,
                0,
            >()
            .policy::<LOOP_POLICY_ID>(),
            g::send::<
                g::Role<ROLE_CONTROLLER>,
                g::Role<ROLE_CLUSTER>,
                g::Msg<LABEL_MGMT_LOAD_FINAL_CHUNK, LoadChunk>,
                0,
            >(),
        );
        g::route(continue_arm, break_arm)
    };

    let load_stream = || {
        g::seq(
            g::seq(load_begin_token(), load_stream_loop()),
            load_commit_token(),
        )
    };

    let load_request = || {
        let arm_head = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CONTROLLER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_LOAD }, MgmtRouteLoadKind>,
            0,
        >()
        .policy::<REQUEST_LOAD_POLICY_ID>();
        let arm_body = g::seq(
            g::send::<
                g::Role<ROLE_CONTROLLER>,
                g::Role<ROLE_CLUSTER>,
                g::Msg<LABEL_MGMT_STAGE, LoadBegin>,
                0,
            >(),
            load_stream(),
        );
        g::seq(arm_head, arm_body)
    };

    let load_and_activate_request = || {
        let arm_head = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CONTROLLER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_LOAD_AND_ACTIVATE }, MgmtRouteLoadAndActivateKind>,
            0,
        >()
        .policy::<REQUEST_LOAD_POLICY_ID>();
        let arm_body = g::seq(
            g::send::<
                g::Role<ROLE_CONTROLLER>,
                g::Role<ROLE_CLUSTER>,
                g::Msg<LABEL_MGMT_LOAD_AND_ACTIVATE, LoadBegin>,
                0,
            >(),
            load_stream(),
        );
        g::seq(arm_head, arm_body)
    };

    let load_family = || {
        let family_head = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CONTROLLER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_LOAD_FAMILY }, MgmtRouteLoadFamilyKind>,
            0,
        >()
        .policy::<REQUEST_ROOT_POLICY_ID>();
        let family_route = g::route(load_request(), load_and_activate_request());
        g::seq(family_head, family_route)
    };

    let activate_request = || {
        let arm_head = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CONTROLLER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_ACTIVATE }, MgmtRouteActivateKind>,
            0,
        >()
        .policy::<REQUEST_COMMAND_POLICY_ID>();
        let arm_body = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CLUSTER>,
            g::Msg<LABEL_MGMT_ACTIVATE, SlotRequest>,
            0,
        >();
        g::seq(arm_head, arm_body)
    };

    let revert_request = || {
        let arm_head = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CONTROLLER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_REVERT }, MgmtRouteRevertKind>,
            0,
        >()
        .policy::<REQUEST_COMMAND_TAIL_POLICY_ID>();
        let arm_body = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CLUSTER>,
            g::Msg<LABEL_MGMT_REVERT, SlotRequest>,
            0,
        >();
        g::seq(arm_head, arm_body)
    };

    let stats_request = || {
        let arm_head = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CONTROLLER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_STATS }, MgmtRouteStatsKind>,
            0,
        >()
        .policy::<REQUEST_COMMAND_TAIL_POLICY_ID>();
        let arm_body = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CLUSTER>,
            g::Msg<LABEL_MGMT_STATS, SlotRequest>,
            0,
        >();
        g::seq(arm_head, arm_body)
    };

    let command_tail = || {
        let tail_head = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CONTROLLER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_COMMAND_TAIL }, MgmtRouteCommandTailKind>,
            0,
        >()
        .policy::<REQUEST_COMMAND_POLICY_ID>();
        let tail_route = g::route(revert_request(), stats_request());
        g::seq(tail_head, tail_route)
    };

    let command_family = || {
        let family_head = g::send::<
            g::Role<ROLE_CONTROLLER>,
            g::Role<ROLE_CONTROLLER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_COMMAND_FAMILY }, MgmtRouteCommandFamilyKind>,
            0,
        >()
        .policy::<REQUEST_ROOT_POLICY_ID>();
        let family_route = g::route(activate_request(), command_tail());
        g::seq(family_head, family_route)
    };

    let request_route = g::route(load_family(), command_family());

    let error_reply = || {
        let arm_head = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CLUSTER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_REPLY_ERROR }, MgmtRouteReplyErrorKind>,
            0,
        >()
        .policy::<REPLY_ROOT_POLICY_ID>();
        let arm_body = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CONTROLLER>,
            g::Msg<LABEL_MGMT_REPLY_ERROR, MgmtError>,
            0,
        >();
        g::seq(arm_head, arm_body)
    };

    let loaded_reply = || {
        let arm_head = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CLUSTER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_REPLY_LOADED }, MgmtRouteReplyLoadedKind>,
            0,
        >()
        .policy::<REPLY_SUCCESS_POLICY_ID>();
        let arm_body = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CONTROLLER>,
            g::Msg<LABEL_MGMT_REPLY_LOADED, LoadReport>,
            0,
        >();
        g::seq(arm_head, arm_body)
    };

    let activated_reply = || {
        let arm_head = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CLUSTER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_REPLY_ACTIVATED }, MgmtRouteReplyActivatedKind>,
            0,
        >()
        .policy::<REPLY_SUCCESS_TAIL_POLICY_ID>();
        let arm_body = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CONTROLLER>,
            g::Msg<LABEL_MGMT_REPLY_ACTIVATED, TransitionReport>,
            0,
        >();
        g::seq(arm_head, arm_body)
    };

    let reverted_reply = || {
        let arm_head = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CLUSTER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_REPLY_REVERTED }, MgmtRouteReplyRevertedKind>,
            0,
        >()
        .policy::<REPLY_SUCCESS_FINAL_POLICY_ID>();
        let arm_body = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CONTROLLER>,
            g::Msg<LABEL_MGMT_REPLY_REVERTED, TransitionReport>,
            0,
        >();
        g::seq(arm_head, arm_body)
    };

    let stats_reply = || {
        let arm_head = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CLUSTER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_REPLY_STATS }, MgmtRouteReplyStatsKind>,
            0,
        >()
        .policy::<REPLY_SUCCESS_FINAL_POLICY_ID>();
        let arm_body = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CONTROLLER>,
            g::Msg<LABEL_MGMT_REPLY_STATS, StatsReply>,
            0,
        >();
        g::seq(arm_head, arm_body)
    };

    let success_final_reply = || {
        let family_head = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CLUSTER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_REPLY_SUCCESS_FINAL }, MgmtRouteReplySuccessFinalKind>,
            0,
        >()
        .policy::<REPLY_SUCCESS_TAIL_POLICY_ID>();
        let family_route = g::route(reverted_reply(), stats_reply());
        g::seq(family_head, family_route)
    };

    let success_tail_reply = || {
        let family_head = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CLUSTER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_REPLY_SUCCESS_TAIL }, MgmtRouteReplySuccessTailKind>,
            0,
        >()
        .policy::<REPLY_SUCCESS_POLICY_ID>();
        let family_route = g::route(activated_reply(), success_final_reply());
        g::seq(family_head, family_route)
    };

    let success_reply = || {
        let family_head = g::send::<
            g::Role<ROLE_CLUSTER>,
            g::Role<ROLE_CLUSTER>,
            RouteMsg<{ LABEL_MGMT_ROUTE_REPLY_SUCCESS_FAMILY }, MgmtRouteReplySuccessFamilyKind>,
            0,
        >()
        .policy::<REPLY_ROOT_POLICY_ID>();
        let family_route = g::route(loaded_reply(), success_tail_reply());
        g::seq(family_head, family_route)
    };

    let reply_route = g::route(error_reply(), success_reply());
    let program = g::seq(request_route, reply_route);
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
