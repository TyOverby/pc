
macro_rules! ct {
    () => { () };
    ($first: ident) => { ($first,) };
    ($first: ident, $second: ident) => { ($first, $second) };
    ($first: ident, $($b: tt),+) => {
        ($first, ct!($($b),+))
    };
}

#[test] fn test_ct() {
    let tup2 = (1, 2);
    match tup2 {
        ct!(a, b) => a + b
    };

    let tup3 = (1, (2, 3));
    match tup3 {
        ct!(a, b, c) => a + b + c
    };
    let ct!(a, b, c) = tup3;
}
