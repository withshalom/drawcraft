//! DRAWcraft: native files save under the `.drawcraft` name, as VectorCraft files of the current
//! version, and open as current files.

use serde_json::{Value, json};

use crate::*;

#[test]
fn native_files_save_as_drawcraft_and_open_as_current() {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
    s.execute("shape.rectangle", &json!({"x": 10, "y": 10, "width": 20, "height": 10})).unwrap();
    // The native format's first extension is the one Save writes.
    let native = cmd::fileio::FORMATS.iter().find(|f| f.id == "vectorcraft").unwrap();
    assert_eq!(native.extensions, [vectorcraft_format::SAVE_EXTENSION, vectorcraft_format::EXTENSION]);
    assert_eq!(vectorcraft_format::SAVE_EXTENSION, "drawcraft");
    let saved = s.execute("document.save", &json!({})).unwrap();
    let bytes = vectorcraft_format::base64_decode(saved["dataBase64"].as_str().unwrap()).unwrap();
    let head: Value = serde_json::from_slice(vectorcraft_format::json_part(&bytes)).unwrap();
    // The format's own name and the current version in the header, so VectorCraft reads it as a
    // current file.
    assert_eq!(head["format"], "vectorcraft");
    assert_eq!(head["version"], vectorcraft_format::VERSION);
    assert!(head["generator"].as_str().unwrap().starts_with("DRAWcraft "));
    // Opened under the `.drawcraft` name, it isn't an old file: Save writes it back in place.
    let mut o = Session::new();
    o.execute("document.open", &json!({"name": "Poster.drawcraft", "dataBase64": saved["dataBase64"]})).unwrap();
    assert!(!o.doc().unwrap().converted);
}
