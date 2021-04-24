use chronofold::{Change, Chronofold, ChronofoldError, LogIndex, Op, Timestamp};

#[test]
fn unknown_timestamp() {
    let mut cfold = Chronofold::<u8, char>::default();
    let unknown = Timestamp(LogIndex(1), 42);
    let op = Op::new(
        Timestamp(LogIndex(1), 1),
        Some(unknown),
        Change::Insert('!'),
    );
    let err = cfold.apply(op.clone()).unwrap_err();
    assert_eq!(ChronofoldError::UnknownReference(op), err);
    assert_eq!("unknown reference <1, 42>", format!("{}", err));
}

#[test]
fn existing_timestamp() {
    // Applying the same op twice results in a
    // `ChronofoldError::ExistingTimestamp`:
    let mut cfold = Chronofold::<u8, char>::default();
    let op = Op::new(
        Timestamp(LogIndex(1), 1),
        Some(Timestamp(LogIndex(0), 0)),
        Change::Insert('.'),
    );
    assert_eq!(Ok(()), cfold.apply(op.clone()));
    let err = cfold.apply(op.clone()).unwrap_err();
    assert_eq!(ChronofoldError::ExistingTimestamp(op), err);
    assert_eq!("existing timestamp <1, 1>", format!("{}", err));
}
