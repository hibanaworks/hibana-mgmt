#![cfg(feature = "std")]

use std::fs;
use std::path::PathBuf;

use hibana_mgmt::{LoadRequest, PolicyTarget, Request};

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
        "#[allow(private_bounds)]",
        "#[expect(private_bounds",
        "g::advanced",
        "hibana::g::advanced",
        "substrate::{\n        AttachError, RendezvousId",
        "substrate::{AttachError, RendezvousId",
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
    assert!(
        kinds.contains("pub struct MgmtRouteKind<const LABEL: u8, const ARM: u8>;"),
        "management route decisions must use the single const-generic route kind owner"
    );
    assert!(
        !kinds.contains("pub type MgmtRoute"),
        "management route kind surface must not grow per-label aliases"
    );
    for required in [
        "GenericCapToken<LoadBeginKind>",
        "GenericCapToken<LoadCommitKind>",
    ] {
        assert!(
            src.contains(required),
            "request_reply attach path must use control-kind messages: {required}"
        );
    }
    for forbidden in ["Msg<LABEL_MGMT_LOAD_BEGIN,", "Msg<LABEL_MGMT_LOAD_COMMIT,"] {
        assert!(
            !src.contains(forbidden),
            "request_reply attach path must not use the old raw label path: {forbidden}"
        );
    }
}

#[test]
fn request_reply_uses_final_form_substrate_paths() {
    let src = read("src/request_reply.rs");
    for forbidden in [
        "#[allow(private_bounds)]",
        "#[expect(private_bounds",
        "g::advanced",
        "hibana::g::advanced",
        "hibana::substrate::RendezvousId",
        "hibana::substrate::SessionId",
        "substrate::{\n        AttachError, RendezvousId",
        "substrate::{AttachError, RendezvousId",
    ] {
        assert!(
            !src.contains(forbidden),
            "request_reply must not keep old substrate/g path residue: {forbidden}"
        );
    }
}

#[test]
fn request_reply_payload_surface_stays_available() {
    let _request = Request::LoadAndActivate(LoadRequest {
        target: PolicyTarget::Rendezvous,
        code: &[0x30, 0x03, 0x00, 0x01],
        fuel_max: 64,
        mem_len: 128,
    });

    let payload = read("src/payload.rs");
    assert!(payload.contains("pub enum PolicyTarget"));
    assert!(payload.contains("pub target: PolicyTarget"));
    for forbidden in [
        "PolicySlot",
        "policy::advanced",
        "pub slot:",
        "decode_slot",
        "slot_id",
    ] {
        assert!(
            !payload.contains(forbidden),
            "management payload surface must own policy targets without advanced slot leakage: {forbidden}"
        );
    }
}

#[test]
fn load_chunk_decode_surface_stays_borrowed_not_fixed_buffer_copy() {
    let payload = read("src/payload.rs");
    assert!(
        !payload.contains("pub bytes: [u8; LOAD_CHUNK_MAX]"),
        "LoadChunk must not expose or decode into a fixed 1KB public buffer"
    );
    assert!(
        !payload.contains("let mut bytes = [0u8; LOAD_CHUNK_MAX]"),
        "LoadChunk decode must not materialize a fixed 1KB stack buffer"
    );
    assert!(
        payload.contains("type Decoded<'a> = LoadChunk<'a>"),
        "LoadChunk wire decode must return a borrowed chunk view"
    );
}

#[test]
fn policy_lifecycle_vocabulary_uses_revert_not_restore() {
    for path in [
        "src/payload.rs",
        "src/control_kinds.rs",
        "src/request_reply.rs",
    ] {
        let src = read(path);
        for forbidden in [
            "Restore",
            "Restored",
            "restore",
            "restored",
            "restores",
            "last_restore",
        ] {
            assert!(
                !src.contains(forbidden),
                "management lifecycle vocabulary must stay on revert: {path}: {forbidden}"
            );
        }
    }

    let payload = read("src/payload.rs");
    assert!(payload.contains("Revert(SlotRequest)"));
    assert!(payload.contains("Reverted(TransitionReport)"));
}

fn manifest_rev<'a>(cargo_toml: &'a str, crate_name: &str) -> &'a str {
    let needle =
        format!("{crate_name} = {{ git = \"https://github.com/hibanaworks/{crate_name}\", rev = \"");
    let start = cargo_toml
        .find(&needle)
        .unwrap_or_else(|| panic!("hibana-mgmt must depend on an immutable {crate_name} GitHub rev"))
        + needle.len();
    let rev = cargo_toml[start..]
        .split('"')
        .next()
        .unwrap_or_else(|| panic!("{crate_name} dependency must include a rev"));
    assert_eq!(rev.len(), 40);
    assert!(rev.bytes().all(|byte| byte.is_ascii_hexdigit()));
    rev
}

#[test]
fn dependency_surface_uses_immutable_git_revs() {
    let cargo_toml = read("Cargo.toml");
    let cargo_config = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".cargo/config.toml");

    let _ = manifest_rev(&cargo_toml, "hibana");
    let _ = manifest_rev(&cargo_toml, "hibana-epf");
    assert!(!cargo_toml.contains("hibana = { path = \"../hibana\""));
    assert!(!cargo_toml.contains("hibana-epf = { path = \"../hibana-epf\""));
    assert!(!cargo_config.exists());
}
