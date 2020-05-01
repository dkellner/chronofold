//! Tests for the `Vec`-like API.
//!
//! The main purpose of these tests is not to cover all corner cases, but
//! rather to show that they behave like there counterparts on `Vec`.

use chronofold::{Change, Chronofold, LogIndex, Session};

#[test]
fn is_empty() {
    let vec = Vec::<char>::default();
    let mut cfold = Chronofold::<u8, char>::default();
    let idx = cfold.session(1).push_back('!');
    cfold.session(1).remove(idx);
    assert_eq!(vec.is_empty(), cfold.is_empty());
}

#[test]
fn len() {
    let mut vec = Vec::<char>::default();
    vec.extend("len".chars());
    let mut cfold = Chronofold::<u8, char>::default();
    cfold.session(1).extend("len".chars());
    let idx = cfold.session(1).push_back('?');
    cfold.session(1).remove(idx);
    assert_eq!(vec.len(), cfold.len());
}

#[test]
fn get() {
    let mut vec = Vec::<char>::default();
    vec.extend("abc".chars());
    let mut cfold = Chronofold::<u8, char>::default();
    cfold.session(1).extend("abc".chars());
    assert_eq!(Some(&'b'), vec.get(1));
    assert_eq!(Some(&Change::Insert('b')), cfold.get(LogIndex(1)));
}

#[test]
fn clear() {
    assert_elements_eq(
        "foobar".chars(),
        |vec| {
            vec.clear();
        },
        |cfold_session| {
            cfold_session.clear();
        },
    );
}

#[test]
fn insert_after() {
    assert_elements_eq(
        "fobar".chars(),
        |vec| {
            vec.insert(2, 'o');
        },
        |cfold_session| {
            cfold_session.insert_after(Some(LogIndex(1)), 'o');
        },
    );
}

#[test]
fn extend() {
    // Extend empty sequence:
    assert_elements_eq(
        "".chars(),
        |vec| {
            vec.extend("foo".chars());
        },
        |cfold_session| {
            cfold_session.extend("foo".chars());
        },
    );

    // Extend non-empty sequence:
    assert_elements_eq(
        "foo".chars(),
        |vec| {
            vec.extend("bar".chars());
        },
        |cfold_session| {
            cfold_session.extend("bar".chars());
        },
    );
}

#[test]
fn splice() {
    // Replace the whole sequence by using an unbounded range:
    assert_elements_eq(
        "bar".chars(),
        |vec| {
            vec.splice(.., "foo".chars());
        },
        |cfold_session| {
            cfold_session.splice(.., "foo".chars());
        },
    );

    // Insert a new sequence without removing anything:
    assert_elements_eq(
        "foo!".chars(),
        |vec| {
            vec.splice(3..3, "bar".chars());
        },
        |cfold_session| {
            cfold_session.splice(LogIndex(3)..LogIndex(3), "bar".chars());
        },
    );

    // Insert a new sequence in an empty vector/chronofold:
    assert_elements_eq(
        "".chars(),
        |vec| {
            vec.splice(0..0, "foo".chars());
        },
        |cfold_session| {
            cfold_session.splice(LogIndex(0)..LogIndex(0), "foo".chars());
        },
    );

    // Extend a sequence by using an out-of-bound index. There's a subtle
    // difference here: `Vec.splice` only allows indices <= `Vec.len()`, the
    // chronofold extends for all out-of-bound indices.
    assert_elements_eq(
        "foo".chars(),
        |vec| {
            vec.splice(3..3, "bar".chars());
        },
        |cfold_session| {
            cfold_session.splice(LogIndex(3)..LogIndex(3), "bar".chars());
        },
    );
}

fn assert_elements_eq<I, T, F, G>(initial_values: I, mutate_vec: F, mutate_chronofold: G)
where
    I: Iterator<Item = T>,
    F: FnOnce(&mut Vec<T>) -> (),
    G: FnOnce(&mut Session<u8, T>) -> (),
    T: PartialEq + Clone + std::fmt::Debug,
{
    let mut vec: Vec<T> = initial_values.collect();
    let mut cfold = Chronofold::<u8, T>::default();
    let mut cfold_session = cfold.session(1);
    cfold_session.extend(vec.clone().into_iter());

    mutate_vec(&mut vec);
    mutate_chronofold(&mut cfold_session);

    assert_eq!(
        vec.iter().collect::<Vec<_>>(),
        cfold.iter_elements().collect::<Vec<_>>()
    );
}
