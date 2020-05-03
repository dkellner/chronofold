use chronofold::{Change, Chronofold, ChronofoldError, LogIndex, Op, Timestamp};

#[test]
fn unknown_timestamp() {
    let mut cfold = Chronofold::<u8, char>::new();
    let unknown = Timestamp(LogIndex(0), 42);
    assert_eq!(
        Err(ChronofoldError::UnknownTimestamp(unknown)),
        cfold.apply(Op::new(
            Timestamp(LogIndex(0), 1),
            Some(unknown),
            Change::Insert('!'),
        ))
    );
}

#[test]
fn existing_timestamp() {
    // Applying the same op twice results in a
    // `ChronofoldError::ExistingTimestamp`:
    let mut cfold = Chronofold::<u8, char>::new();
    let op = Op::new(Timestamp(LogIndex(0), 1), None, Change::Insert('.'));
    assert_eq!(Ok(()), cfold.apply(op.clone()));
    eprintln!("{:?}", cfold);
    assert_eq!(
        Err(ChronofoldError::ExistingTimestamp(op.id)),
        cfold.apply(op)
    );
}
