use chronofold::{Chronofold, ChronofoldError, LogIndex, Op, Timestamp};

#[test]
fn unknown_timestamp() {
    let mut cfold = Chronofold::<u8, char>::default();
    let unknown = Timestamp(LogIndex(1), 42);
    let op = Op::insert(Timestamp(LogIndex(1), 1), Some(unknown), '!');
    let err = cfold.apply(op.clone()).unwrap_err();
    assert_eq!(ChronofoldError::UnknownReference(op), err);
    assert_eq!("unknown reference <1, 42>", format!("{}", err));
}

#[test]
fn future_timestamp() {
    let mut cfold = Chronofold::<u8, char>::default();
    let op = Op::insert(
        Timestamp(LogIndex(9), 1),
        Some(Timestamp(LogIndex(0), 0)),
        '.',
    );
    let err = cfold.apply(op.clone()).unwrap_err();
    assert_eq!(ChronofoldError::FutureTimestamp(op), err);
    assert_eq!("future timestamp <9, 1>", format!("{}", err));
}

#[test]
fn existing_timestamp() {
    // Applying the same op twice results in a
    // `ChronofoldError::ExistingTimestamp`:
    let mut cfold = Chronofold::<u8, char>::default();
    let op = Op::insert(
        Timestamp(LogIndex(1), 1),
        Some(Timestamp(LogIndex(0), 0)),
        '.',
    );
    assert_eq!(Ok(()), cfold.apply(op.clone()));
    let err = cfold.apply(op.clone()).unwrap_err();
    assert_eq!(ChronofoldError::ExistingTimestamp(op), err);
    assert_eq!("existing timestamp <1, 1>", format!("{}", err));
}
