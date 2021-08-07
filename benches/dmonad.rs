//! # Benchmarks inspired by `dmonad/crdt-benchmarks`
//!
//! Rust port of the benchmarks at [`dmonad/crdt-benchmarks`]. The results will
//! not be directly comparable for a number of reasons, but at least we're
//! measuring the same operations which should give an idea about overall
//! performance characteristics.
//!
//! ## Currently implemented benchmarks
//!
//! - B1.1 (`append_chars`)
//! - B1.2 (`insert_string`)
//! - B1.3 (`prepend_chars`)
//! - B1.4 (`insert_chars_at_random_positions`)
//! - B1.5 (`insert_words_at_random_positions`)
//! - B1.6 (`insert_and_delete_string`)
//! - B1.7 (`insert_and_delete_strings_at_random_positions`)
//!
//! ## TODO
//!
//! - Measure memory usage
//! - Add missing benchmarks
//!
//! [`dmonad/crdt-benchmarks`]: https://github.com/dmonad/crdt-benchmarks

use chronofold::{Chronofold, LogIndex, Session};
use criterion::{criterion_group, criterion_main, Criterion};
use rand::{rngs::ThreadRng, seq::IteratorRandom, Rng};
use std::ops::RangeInclusive;
use std::time::{Duration, Instant};

type CFold = Chronofold<u8, char>;
type Sess<'a> = Session<'a, u8, char>;

const N: usize = 6000;

fn append_chars(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let input = random_string(&mut rng, N..=N);
    bench(
        c,
        &format!("[B1.1] Append {} characters", N),
        input.chars().collect(),
        |sess, c| {
            sess.push_back(c);
        },
        assert_docs_equal(&input),
    );
}

fn insert_string(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let input = random_string(&mut rng, N..=N);
    bench(
        c,
        &format!("[B1.2] Insert string of length {}", N),
        vec![input.clone()],
        |sess, s| {
            sess.extend(s.chars());
        },
        assert_docs_equal(&input),
    );
}

fn prepend_chars(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let input = random_string(&mut rng, N..=N);
    bench(
        c,
        &format!("[B1.3] Prepend {} characters", N),
        input.chars().collect(),
        |sess, c| {
            sess.push_front(c);
        },
        assert_docs_equal(&input.chars().rev().collect::<String>()),
    );
}

fn insert_chars_at_random_positions(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let mut expected = Vec::<char>::new();
    let positions_and_chars: Vec<_> = (0..N)
        .map(|i| {
            let pos = (0..=i).choose(&mut rng).unwrap();
            let c = random_char(&mut rng);
            expected.insert(pos, c);
            (pos, c)
        })
        .collect();

    bench(
        c,
        &format!("[B1.4] Insert {} characters at random positions", N),
        positions_and_chars,
        |sess, (pos, c)| {
            let idx = if pos == 0 {
                LogIndex(0) // insert as first element
            } else {
                // This is expected to be really slow, as accessing a specific
                // position (as opposed to a log index) requires walking the
                // linked list up to that position.
                sess.as_ref().iter().nth(pos - 1).unwrap().1
            };
            sess.insert_after(idx, c);
        },
        assert_docs_equal(&expected.into_iter().collect::<String>()),
    );
}

fn insert_words_at_random_positions(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let mut expected = Vec::<char>::new();
    let positions_and_words: Vec<_> = (0..N)
        .map(|_i| {
            let pos = (0..=expected.len()).choose(&mut rng).unwrap();
            let s = random_string(&mut rng, 2..=10);
            expected.splice(pos..pos, s.chars());
            (pos, s)
        })
        .collect();

    bench(
        c,
        &format!("[B1.5] Insert {} words at random positions", N),
        positions_and_words,
        |sess, (pos, s)| {
            // This is expected to be really slow, as accessing a specific
            // position (as opposed to a log index) requires walking the
            // linked list up to that position.
            let e = sess.as_ref().iter().nth(pos);
            match e {
                Some((_, idx)) => sess.splice(idx..idx, s.chars()),
                None => sess.extend(s.chars()),
            };
        },
        assert_docs_equal(&expected.into_iter().collect::<String>()),
    );
}

fn insert_and_delete_string(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let s = random_string(&mut rng, N..=N);
    bench(
        c,
        &format!("[B1.6] Insert string of length {}, then delete it", N),
        vec![s],
        |sess, s| {
            let last_idx = sess.extend(s.chars()).unwrap();
            sess.splice(LogIndex(0)..=last_idx, "".chars());
        },
        assert_docs_equal(""),
    );
}

fn insert_and_delete_strings_at_random_positions(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let mut expected = Vec::<char>::new();
    let input: Vec<_> = (0..N)
        .map(|_i| {
            let pos = (0..=expected.len()).choose(&mut rng).unwrap();
            if expected.len() == pos || rng.gen() {
                // Insert
                let s = random_string(&mut rng, 2..=10);
                expected.splice(pos..pos, s.chars());
                (pos, pos, s)
            } else {
                // Delete
                let del_count = (1..=usize::min(9, expected.len() - pos))
                    .choose(&mut rng)
                    .unwrap();
                expected.splice(pos..pos + del_count, "".chars());
                (pos, pos + del_count, "".to_owned())
            }
        })
        .collect();

    bench(
        c,
        &format!("[B1.7] Insert/Delete {} strings at random positions", N),
        input,
        |sess, (start, end, s)| {
            let a = sess.as_ref().iter().nth(start).map(|(_, idx)| idx);
            let b = sess.as_ref().iter().nth(end).map(|(_, idx)| idx);
            match (a, b) {
                (None, _) => sess.extend(s.chars()),
                (Some(start_idx), None) => sess.splice(start_idx.., s.chars()),
                (Some(start_idx), Some(end_idx)) => sess.splice(start_idx..end_idx, s.chars()),
            };
        },
        assert_docs_equal(&expected.iter().collect::<String>()),
    );
}

fn bench<T: Clone, F: Fn(&mut Sess<'_>, T) + Copy, G: Fn(&CFold, &CFold) + Copy>(
    c: &mut Criterion,
    name: &str,
    input: Vec<T>,
    apply_change: F,
    check: G,
) {
    c.bench_function(&format!("{} (time)", name), |b| {
        b.iter_custom(|iters| {
            let mut elapsed = Duration::new(0, 0);
            for _i in 0..iters {
                elapsed += measure_time(input.clone(), apply_change, check);
            }
            elapsed
        });
    });
    measure_space(name, input, apply_change);
}

fn measure_time<T, F: Fn(&mut Sess<'_>, T), G: Fn(&CFold, &CFold)>(
    input: Vec<T>,
    apply_change: F,
    check: G,
) -> Duration {
    let mut doc1 = CFold::default();
    let mut doc2 = CFold::default();

    let start = Instant::now();
    for d in input.into_iter() {
        let mut session = doc1.session(1);
        apply_change(&mut session, d);
        for op in session.iter_ops() {
            let _ = doc2.apply(op.cloned());
        }
    }
    let elapsed = start.elapsed();

    check(&doc1, &doc2);
    elapsed
}

fn measure_space<T, F: Fn(&mut Sess<'_>, T)>(name: &str, input: Vec<T>, apply_change: F) {
    let mut cfold = CFold::default();
    let mut total_ops_bytes = 0;
    let n = input.len();
    for d in input.into_iter() {
        let mut session = cfold.session(1);
        apply_change(&mut session, d);
        for op in session.iter_ops::<&char>() {
            total_ops_bytes += serde_json::to_vec(&op).unwrap().len();
        }
    }
    let avg_update_size = total_ops_bytes / n;
    let doc_size = serde_json::to_vec(&cfold).unwrap().len();
    println!("{} (avgUpdateSize): {} bytes", name, avg_update_size);
    println!("{} (docSize): {} bytes", name, doc_size);
    println!();
}

fn assert_docs_equal(expected: &str) -> impl Fn(&CFold, &CFold) + Copy + '_ {
    move |doc1, doc2| {
        let text1 = format!("{}", doc1);
        let text2 = format!("{}", doc2);
        assert_eq!(text1, text2);
        assert_eq!(expected, text1);
    }
}

fn random_char(rng: &mut ThreadRng) -> char {
    "abcdefghijklmnopqrstuvwxyz".chars().choose(rng).unwrap()
}

fn random_string(rng: &mut ThreadRng, len_range: RangeInclusive<usize>) -> String {
    let len = len_range.choose(rng).unwrap();
    (0..len).map(|_| random_char(rng)).collect()
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets =
      append_chars,
      insert_string,
      prepend_chars,
      insert_chars_at_random_positions,
      insert_words_at_random_positions,
      insert_and_delete_string,
      insert_and_delete_strings_at_random_positions,
);
criterion_main!(benches);
