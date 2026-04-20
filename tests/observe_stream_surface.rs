#![cfg(feature = "std")]

use std::fs;
use std::path::PathBuf;

use hibana::substrate::tap::TapEvent;
use hibana_mgmt::SubscribeReq;

fn read(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let full = root.join(path);
    fs::read_to_string(&full)
        .unwrap_or_else(|err| panic!("read {} failed: {}", full.display(), err))
}

#[test]
fn observe_stream_surface_uses_attach_helpers_not_raw_program_exports() {
    let src = read("src/observe_stream.rs");

    assert!(
        src.contains("pub fn attach_controller") && src.contains("pub fn attach_cluster"),
        "observe_stream surface must expose attach helpers"
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
            "observe_stream surface must not export raw choreography values: {forbidden}"
        );
    }

    let kinds = read("src/control_kinds.rs");
    assert!(
        kinds.contains("pub struct MgmtRouteKind"),
        "management route owner must stay in hibana-mgmt"
    );
}

#[test]
fn observe_stream_payload_surface_stays_available() {
    let _subscribe = SubscribeReq::default();
    let _tap = TapEvent::default();
}
