# Chronofold

Chronofold is a conflict-free replicated data structure (a.k.a. *CRDT*) for
versioned text.

This crate aims to offer a fast implementation with an easy-to-use
`Vec`-like API. It should be near impossible to shoot yourself in the foot
and end up with corrupted or lost data.

**Note:** We are not there yet! While this implementation should be
correct, it is not yet optimized for speed and memory usage. The API might
see some changes as we continue to explore different use cases.

This implementation is based on ideas published in the paper ["Chronofold:
a data structure for versioned text"][paper] by Victor Grishchenko and
Mikhail Patrakeev. If you look for a formal introduction to what a
chronofold is, reading that excellent paper is highly recommended!

[paper]: https://arxiv.org/abs/2002.09511

# Example usage

```rust
use chronofold::{Chronofold, LogIndex, Op};

type AuthorId = &'static str;

// Alice creates a chronofold on her machine, makes some initial changes
// and sends a copy to Bob.
let mut cfold_a = Chronofold::<AuthorId, char>::default();
cfold_a.session("alice").extend("Hello chronfold!".chars());
let mut cfold_b = cfold_a.clone();

// Alice adds some more text, ...
let ops_a: Vec<Op<AuthorId, char>> = {
    let mut session = cfold_a.session("alice");
    session.splice(
        LogIndex(16)..LogIndex(16),
        " - a data structure for versioned text".chars(),
    );
    session.iter_ops().map(Op::cloned).collect()
};

// ... while Bob fixes a typo.
let ops_b: Vec<Op<AuthorId, char>> = {
    let mut session = cfold_b.session("bob");
    session.insert_after(LogIndex(11), 'o');
    session.iter_ops().map(Op::cloned).collect()
};

// Now their respective states have diverged.
assert_eq!(
    "Hello chronfold - a data structure for versioned text!",
    format!("{cfold_a}"),
);
assert_eq!("Hello chronofold!", format!("{cfold_b}"));

// As soon as both have seen all ops, their states have converged.
for op in ops_a {
    cfold_b.apply(op).unwrap();
}
for op in ops_b {
    cfold_a.apply(op).unwrap();
}
let final_text = "Hello chronofold - a data structure for versioned text!";
assert_eq!(final_text, format!("{cfold_a}"));
assert_eq!(final_text, format!("{cfold_b}"));
```

# Roadmap

## 1.0 - A minimal first release

- API is well-designed and considered stable
- Good test coverage
- Internal representations of data structures don't matter yet
- No optimizations
