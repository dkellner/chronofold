#![cfg(feature = "serde")]
use chronofold::{Chronofold, LogIndex};

#[test]
fn roundtrip() {
    let mut cfold = Chronofold::<usize, char>::new();
    cfold.session(1).extend("Hello world!".chars());
    let json = serde_json::to_string(&cfold).unwrap();
    eprintln!("{}", json);
    assert_eq!(cfold, serde_json::from_str(&json).unwrap());
}

#[test]
fn empty() {
    let cfold = Chronofold::<usize, char>::new();
    assert_json_max_len(&cfold, 132);
}

#[test]
fn local_edits_only() {
    let mut cfold = Chronofold::<usize, char>::new();
    cfold.session(1).extend("Hello world!".chars());
    cfold
        .session(1)
        .splice(LogIndex(6)..LogIndex(11), "cfold".chars());
    assert_json_max_len(&cfold, 390);
}

fn assert_json_max_len(cfold: &Chronofold<usize, char>, max_len: usize) {
    let json = serde_json::to_string(&cfold).unwrap();
    assert!(
        json.len() <= max_len,
        format!(
            "length of {} is not <= {} (it is {})",
            json,
            max_len,
            json.len()
        )
    );
}
