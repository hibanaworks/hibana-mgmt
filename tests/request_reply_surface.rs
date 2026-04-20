#![cfg(feature = "std")]

use std::fs;
use std::path::PathBuf;

use hibana::substrate::policy::PolicySlot;
use hibana_mgmt::{LoadRequest, Request};

fn read(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let full = root.join(path);
    fs::read_to_string(&full)
        .unwrap_or_else(|err| panic!("read {} failed: {}", full.display(), err))
}

#[test]
fn request_reply_surface_uses_attach_helpers_not_raw_program_exports() {
    let src = read("src/request_reply.rs");

    assert!(
        src.contains("pub fn attach_controller") && src.contains("pub fn attach_cluster"),
        "request_reply surface must expose attach helpers"
    );

    for forbidden in [
        "pub const PROGRAM",
        "pub const PREFIX",
        "g::advanced::steps",
        "const APP: g::Program<_>",
        "static APP: g::Program<_>",
        "const PROGRAM: g::Program<_>",
        "static PROGRAM: g::Program<_>",
        "project(&PROGRAM)",
        "project::<",
    ] {
        assert!(
            !src.contains(forbidden),
            "request_reply surface must not export raw choreography values: {forbidden}"
        );
    }

    let kinds = read("src/control_kinds.rs");
    for required in ["pub struct LoadBeginKind;", "pub struct LoadCommitKind;"] {
        assert!(
            kinds.contains(required),
            "management kind owner must live in hibana-mgmt: {required}"
        );
    }
}

#[test]
fn request_reply_payload_surface_stays_available() {
    let _request = Request::LoadAndActivate(LoadRequest {
        slot: PolicySlot::Rendezvous,
        code: &[0x30, 0x03, 0x00, 0x01],
        fuel_max: 64,
        mem_len: 128,
    });
}
