#![cfg(feature = "std")]

use hibana::{
    g,
    g::advanced::{
        project,
        steps::{SendStep, SeqSteps, StepCons, StepNil},
    },
    substrate::cap::advanced::MintConfig,
};
use hibana_mgmt::{ROLE_CLUSTER, ROLE_CONTROLLER, SubscribeReq, observe_stream};

type AppSteps =
    StepCons<SendStep<g::Role<ROLE_CONTROLLER>, g::Role<ROLE_CLUSTER>, g::Msg<121, ()>>, StepNil>;
type ProgramSteps = SeqSteps<observe_stream::ProgramSteps, AppSteps>;

const APP: g::Program<AppSteps> =
    g::send::<g::Role<ROLE_CONTROLLER>, g::Role<ROLE_CLUSTER>, g::Msg<121, ()>, 0>();
const PROGRAM: g::Program<ProgramSteps> = g::seq(observe_stream::PROGRAM, APP);

#[test]
fn observe_stream_program_projects_from_standalone_repo() {
    let prefix = observe_stream::PROGRAM;
    let _controller: hibana::g::advanced::RoleProgram<'_, ROLE_CONTROLLER, MintConfig> =
        project(&prefix);
    let _cluster: hibana::g::advanced::RoleProgram<'_, ROLE_CLUSTER, MintConfig> = project(&prefix);

    let _subscribe = SubscribeReq::default();
    let _tap = hibana::substrate::tap::TapEvent::default();
}

#[test]
fn observe_stream_program_stays_composable_as_prefix() {
    let program = PROGRAM;
    let _controller: hibana::g::advanced::RoleProgram<'_, ROLE_CONTROLLER, MintConfig> =
        project(&program);
    let _cluster: hibana::g::advanced::RoleProgram<'_, ROLE_CLUSTER, MintConfig> =
        project(&program);
}
