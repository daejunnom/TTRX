use ttrx_core::{SourceKind, decode, encode, inspect_source, json, verify};

const SINGLE: &str = r#"{
  "version": 1,
  "gamemode": "custom",
  "replay": {
    "frames": 3,
    "options": {
      "version": 19,
      "boardwidth": 4,
      "kickset": "SRS-X",
      "custom_option": {"preserved": true}
    },
    "events": [
      {"frame": 0,"type":"start","data":{}},
      {"frame": 0,"type":"keydown","data":{"key":"moveLeft","subframe":0.1}},
      {"frame": 1,"type":"custom.future","data":{"number":-0,"text":"한글"}},
      {"frame": 2,"type":"end","data":{"reason":"clear"}}
    ]
  }
}"#;

const MULTI: &str = r#"{
  "version":1,
  "replay":{"rounds":[[
    {"id":"a","replay":{"frames":2,"options":{"version":19},"events":[
      {"frame":0,"type":"start","data":{}},
      {"frame":1,"type":"ige","data":{"type":"target","frame":7,"data":{"targets":["b"]}}}
    ]}},
    {"id":"b","replay":{"frames":2,"options":{"version":19},"events":[
      {"frame":0,"type":"start","data":{}},
      {"frame":1,"type":"end","data":{"reason":"winner"}}
    ]}}
  ]]}
}"#;

#[test]
fn single_source_semantics_round_trip() {
    let encoded = encode(SINGLE.as_bytes(), SourceKind::Ttr).unwrap();
    assert!(encoded.starts_with(b"TTRX"));
    let (decoded, info) = decode(&encoded).unwrap();
    assert_eq!(info.source_kind, SourceKind::Ttr);
    assert_eq!(
        json::parse(&decoded).unwrap(),
        json::parse(SINGLE.as_bytes()).unwrap()
    );
    assert!(encoded.len() < SINGLE.len());
}

#[test]
fn multiplayer_round_trip_and_summary() {
    let encoded = encode(MULTI.as_bytes(), SourceKind::Ttrm).unwrap();
    let (decoded, info) = decode(&encoded).unwrap();
    assert_eq!(info.source_kind, SourceKind::Ttrm);
    assert_eq!(
        json::parse(&decoded).unwrap(),
        json::parse(MULTI.as_bytes()).unwrap()
    );
    let summary = inspect_source(MULTI.as_bytes(), SourceKind::Unknown).unwrap();
    assert_eq!(summary.stream_count, 2);
    assert_eq!(summary.event_count, 4);
    assert_eq!(summary.ige_event_count(), 1);
}

#[test]
fn verification_reports_exact_tree() {
    let result = verify(SINGLE.as_bytes(), SourceKind::Unknown).unwrap();
    assert!(result.exact_data_model);
    assert_eq!(result.summary.kind, SourceKind::Ttr);
    assert_eq!(result.input_bytes, SINGLE.len());
}

#[test]
fn custom_values_and_duplicate_keys_are_preserved_by_core_codec() {
    let value = json::parse(br#"{"same":1,"same":2,"x":1e0,"future":[null,false,true]}"#).unwrap();
    let bytes = ttrx_core::codec::encode(&value, SourceKind::Unknown, 58).unwrap();
    let decoded = ttrx_core::codec::decode(&bytes).unwrap();
    assert_eq!(decoded.value, value);
}
