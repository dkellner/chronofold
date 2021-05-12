use chronofold::{Chronofold, LogIndex, Op, Timestamp, Version};

#[test]
fn partial_order() {
    assert!(v(vec![]) == v(vec![]));

    assert!(v(vec![]) < v(vec![t(0, 0)]));
    assert!(v(vec![t(0, 0)]) > v(vec![]));

    assert!(v(vec![t(0, 1)]) < v(vec![t(1, 1)]));
    assert!(v(vec![t(1, 1)]) > v(vec![t(0, 1)]));

    assert!(!(v(vec![t(0, 1)]) == v(vec![t(0, 2)])));
    assert!(!(v(vec![t(0, 1)]) < v(vec![t(0, 2)])));
    assert!(!(v(vec![t(0, 1)]) > v(vec![t(0, 2)])));
}

#[test]
fn iter_newer_ops() {
    let mut cfold = Chronofold::<u8, char>::default();
    cfold.session(1).extend("foo".chars());
    let v1 = cfold.version().clone();
    cfold.session(1).push_back('!');
    cfold.session(2).push_back('?');

    assert_eq!(
        vec![
            Op::insert(t(4, 1), Some(t(3, 1)), &'!'),
            Op::insert(t(5, 2), Some(t(4, 1)), &'?')
        ],
        cfold.iter_newer_ops(&v1).collect::<Vec<_>>()
    );

    let mut v2 = Version::new();
    v2.inc(&Timestamp(LogIndex(1), 3));
    assert_eq!(
        vec![
            Op::root(t(0, 0)),
            Op::insert(t(1, 1), Some(t(0, 0)), &'f'),
            Op::insert(t(2, 1), Some(t(1, 1)), &'o'),
            Op::insert(t(3, 1), Some(t(2, 1)), &'o'),
            Op::insert(t(4, 1), Some(t(3, 1)), &'!'),
            Op::insert(t(5, 2), Some(t(4, 1)), &'?')
        ],
        cfold.iter_newer_ops(&v2).collect::<Vec<_>>()
    );
}

fn t(log_index: usize, author: u8) -> Timestamp<u8> {
    Timestamp(LogIndex(log_index), author)
}

fn v(timestamps: Vec<Timestamp<u8>>) -> Version<u8> {
    let mut version = Version::<u8>::new();
    for t in timestamps.iter() {
        version.inc(t);
    }
    version
}
