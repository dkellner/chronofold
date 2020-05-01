//! A simple smoke test with two authors synchronizing random edits.
//!
//! TODO: Replace by property based tests.

use chronofold::{Chronofold, Op};
use rand::{rngs::ThreadRng, Rng};

type AuthorId = &'static str;

#[test]
fn random_edits_by_two_authors() {
    let mut rng = rand::thread_rng();

    // Alice creates a chronofold and makes some edits before sending Bob a
    // copy.
    let mut cfold_alice = Chronofold::<AuthorId, char>::default();
    random_edits(&mut rng, "alice", &mut cfold_alice);
    let mut cfold_bob = cfold_alice.clone();

    // Alice and Bob both work on an their own copy, sending each other their
    // ops after they finish their edits each day. After ten days, they compare
    // their results.
    for _ in 0..10 {
        let ops_alice = random_edits(&mut rng, "alice", &mut cfold_alice);
        let ops_bob = random_edits(&mut rng, "bob", &mut cfold_bob);
        for op in ops_alice {
            cfold_bob.apply(op).unwrap();
        }
        for op in ops_bob {
            cfold_alice.apply(op).unwrap();
        }
    }
    assert_eq!(format!("{}", cfold_alice), format!("{}", cfold_bob));
}

fn random_edits(
    rng: &mut ThreadRng,
    author: AuthorId,
    cfold: &mut Chronofold<AuthorId, char>,
) -> Vec<Op<AuthorId, char>> {
    let mut session = cfold.session(author);

    // 1 to 5 inserts of random words at random positions
    for _ in 0..rng.gen_range(1, 6) {
        let current = session.iter().map(|(_, i)| i).collect::<Vec<_>>();
        if !current.is_empty() {
            let idx = current[rng.gen_range(0, current.len())];
            session.splice(idx..idx, random_word(rng).chars());
        } else {
            session.extend(random_word(rng).chars());
        }
    }

    // 1 to 2 deletions of 1 to 3 characters at random positions
    for _ in 0..rng.gen_range(0, 2) {
        let current = session.iter().map(|(_, i)| i).collect::<Vec<_>>();
        if !current.is_empty() {
            let length = usize::min(rng.gen_range(1, 4), current.len());
            let start = current[rng.gen_range(0, current.len() - length + 1)];
            eprintln!("{} {} {}", current.len(), length, start);
            let (_, end) = session.iter_range(start..).take(length).last().unwrap();
            session.splice(start..=end, "".chars());
        }
    }

    session.ops
}

fn random_word(rng: &mut ThreadRng) -> String {
    let alphabet: Vec<_> = "abcdefghijklmnopqrstuvwxyz".chars().collect();
    (0..rng.gen_range(1, 4))
        .map(|_| alphabet[rng.gen_range(0, alphabet.len())])
        .collect::<String>()
        + " "
}
