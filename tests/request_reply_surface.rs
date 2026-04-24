#![cfg(feature = "std")]

use std::fs;
use std::path::PathBuf;

use hibana::substrate::{
    policy::PolicySlot,
    wire::{Payload, WireEncode, WirePayload},
};
use hibana_mgmt::{LoadChunk, LoadRequest, Request};

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
fn request_reply_payload_surface_stays_available() {
    let _request = Request::LoadAndActivate(LoadRequest {
        slot: PolicySlot::Rendezvous,
        code: &[0x30, 0x03, 0x00, 0x01],
        fuel_max: 64,
        mem_len: 128,
    });
}

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

#[test]
fn dependency_surface_uses_pinned_git_dependencies() {
    let cargo_toml = read("Cargo.toml");
    let cargo_config = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".cargo/config.toml");

    assert!(cargo_toml.contains("git = \"https://github.com/hibanaworks/hibana\""));
    assert!(cargo_toml.contains("git = \"https://github.com/hibanaworks/hibana-epf\""));
    assert!(cargo_toml.contains("rev = \"5b0a522a85694718743b19caa1cadb470cf3a22d\""));
    assert!(cargo_toml.contains("rev = \"e0a977bf969baa9aa63d6879e9878a4af80e796c\""));
    assert!(!cargo_toml.contains("path = \"../hibana\""));
    assert!(!cargo_toml.contains("path = \"../hibana-epf\""));
    assert!(!cargo_config.exists());
}
