# Reproduction steps

Tested on stable (1.60) and nightly (2022-05-04).

## Step 1

```sh
$ cargo test
```

The test `fibonacci1` passes.

```sh
$ cargo test --release
```

## Step 2

The test `fibonacci1` now fails with:
```
thread 'tests::fibonacci1' panicked at 'assertion failed: `(left == right)`
  left: `Some(0.0)`,
 right: `Some(inf)`', src/lib.rs:52:9
```

Specifying `opt-level = 0`, `1` or `2` in `Cargo.toml` makes the test pass again.  
Running MIRI with `-Zmiri-tag-raw-pointers` doesn't detect any UB.

## Step 3


```sh
$ cargo test --release --features enable_second_test
```

The `enable_second_test` feature adds the test `fibonacci2`; now **both** tests pass ??!
