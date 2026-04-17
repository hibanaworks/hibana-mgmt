#![cfg(feature = "std")]

use hibana::{
    g,
    g::advanced::{
        project,
        steps::{SendStep, SeqSteps, StepCons, StepNil},
    },
    substrate::{cap::advanced::MintConfig, policy::PolicySlot},
};
use hibana_mgmt::{LoadRequest, ROLE_CLUSTER, ROLE_CONTROLLER, Request, request_reply};

type AppSteps =
    StepCons<SendStep<g::Role<ROLE_CONTROLLER>, g::Role<ROLE_CLUSTER>, g::Msg<120, u32>>, StepNil>;
type ProgramSteps = SeqSteps<request_reply::ProgramSteps, AppSteps>;

const APP: g::Program<AppSteps> =
    g::send::<g::Role<ROLE_CONTROLLER>, g::Role<ROLE_CLUSTER>, g::Msg<120, u32>, 0>();
const PROGRAM: g::Program<ProgramSteps> = g::seq(request_reply::PROGRAM, APP);

#[test]
fn request_reply_program_projects_from_standalone_repo() {
    let prefix = request_reply::PROGRAM;
    let _controller: hibana::g::advanced::RoleProgram<'_, ROLE_CONTROLLER, MintConfig> =
        project(&prefix);
    let _cluster: hibana::g::advanced::RoleProgram<'_, ROLE_CLUSTER, MintConfig> = project(&prefix);

    let _request = Request::LoadAndActivate(LoadRequest {
        slot: PolicySlot::Rendezvous,
        code: &[0x30, 0x03, 0x00, 0x01],
        fuel_max: 64,
        mem_len: 128,
    });
}

#[test]
fn request_reply_program_stays_composable_as_prefix() {
    let program = PROGRAM;
    let _controller: hibana::g::advanced::RoleProgram<'_, ROLE_CONTROLLER, MintConfig> =
        project(&program);
    let _cluster: hibana::g::advanced::RoleProgram<'_, ROLE_CLUSTER, MintConfig> =
        project(&program);
}
