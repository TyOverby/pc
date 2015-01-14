#![feature(unboxed_closures)]
#![allow(unstable, unused)]

use std::ops::Fn;

#[macro_use]
mod macros;

type ParseResult<G, O> = Result<(G, O), ()>;

trait Parser<I, O> {
    fn parse(&self, I) -> ParseResult<I, O>;
}

trait Generator<I: ?Sized>: Clone {
    fn forward(&self, offset: usize) -> Self;
    fn get(&self) -> &I;
}

#[derive(Clone)]
struct StrGenerator<'a> {
    st: &'a str
}

impl <'a> Generator<str> for StrGenerator<'a> {
    fn forward(&self, offset: usize) -> StrGenerator<'a> {
        StrGenerator {st: &self.st[offset..]}
    }

    fn get(&self) -> &str {
        self.st
    }
}

impl <I: Generator<str>> Parser<I, char> for char {
    fn parse(&self, input: I) -> ParseResult<I, char> {
        match input.get().chars().nth(0) {
            Some(c) if c == *self => {
                let size = c.len_utf8();
                Ok((input.forward(size), c))
            }
            _ => Err(())
        }
    }
}

impl <'a, I: Generator<str>> Parser<I, String> for &'a str {
    fn parse(&self, input: I) -> ParseResult<I, String> {
        if input.get().starts_with(*self) {
            let size = self.len();
            Ok((input.forward(size), input.get().slice_to(size).to_string()))
        } else {
            Err(())
        }
    }
}

#[derive(Clone, Copy)]
struct ConcatParser<L, R> {
    l: L,
    r: R
}

impl <I, Ao, Bo, L: Parser<I, Ao>, R: Parser<I, Bo>> Parser<I, (Ao, Bo)> for ConcatParser<L, R> {
    fn parse(&self, input: I) -> ParseResult<I, (Ao, Bo)> {
        let (input, ans_a) = try!(self.l.parse(input));
        let (input, ans_b) = try!(self.r.parse(input));
        Ok((input, (ans_a, ans_b)))
    }
}


#[derive(Clone, Copy)]
struct MaybeParser<P> {
    p: P
}

impl <I, O, P: Parser<I, O>> Parser<I, Option<O>> for MaybeParser<P> {
    fn parse(&self, input: I) -> ParseResult<I, Option<O>> {
        match self.p.parse(input) {
            Ok((i, r)) => Ok((i, Some(r))),
            Err(())  => Err(())
        }
    }
}

#[derive(Clone, Copy)]
struct RepeatParser<P> {
    p: P,
    limit: Option<usize>
}

impl <I: Clone, O, P: Parser<I, O>> Parser<I, Vec<O>> for RepeatParser<P> {
    fn parse(&self, input: I) -> ParseResult<I, Vec<O>> {
        let mut vec = vec![];
        let mut pos = input.clone();

        loop {
            match self.p.parse(pos.clone()) {
                Ok((p, r)) => {
                    pos = p;
                    vec.push(r)
                }
                Err(()) => {
                    break;
                }
            }
        }

        if let Some(limit) = self.limit {
            if vec.len() >= limit {
                Ok((pos, vec))
            } else {
                Err(())
            }
        } else {
            Ok((pos, vec))
        }
    }
}

#[derive(Clone, Copy)]
struct MapParser<'a, F: 'a, P, O> {
    p: P,
    f: &'a F
}

impl <'a, I, O, B, P: Parser<I, O>, F: Fn(O) -> B + 'a> Parser<I, B> for MapParser<'a, F, P, O> {
    fn parse(&self, input: I) -> ParseResult<I, B> {
        match self.p.parse(input) {
            Ok((p, r)) => Ok((p, (self.f)(r))),
            Err(()) => Err(())
        }
    }
}

#[derive(Clone, Copy)]
struct IgnoreLeftParser<L, R, Ao> {
    l: L,
    r: R
}

impl <I, Ao, Bo, L: Parser<I, Ao>, R: Parser<I, Bo>> Parser<I, Bo> for IgnoreLeftParser<L, R, Ao> {
    fn parse(&self, input: I) -> ParseResult<I, Bo> {
        let (input, ans_a) = try!(self.l.parse(input));
        let (input, ans_b) = try!(self.r.parse(input));
        Ok((input, ans_b))
    }
}

#[derive(Clone, Copy)]
struct IgnoreRightParser<L, R, Bo> {
    l: L,
    r: R
}

impl <I, Ao, Bo, L: Parser<I, Ao>, R: Parser<I, Bo>> Parser<I, Ao> for IgnoreRightParser<L, R, Bo> {
    fn parse(&self, input: I) -> ParseResult<I, Ao> {
        let (input, ans_a) = try!(self.l.parse(input));
        let (input, ans_b) = try!(self.r.parse(input));
        Ok((input, ans_a))
    }
}

#[derive(Clone, Copy)]
struct OrParser<L, R> {
    l: L,
    r: R
}

impl <I, O, L, R> Parser<I, O> for OrParser<L, R>
where I: Clone, L: Parser<I, O>, R: Parser<I, O> {
    fn parse(&self, input: I) -> ParseResult<I, O> {
        if let Ok((p, r)) = self.l.parse(input.clone()) {
            return Ok((p, r))
        } else if let Ok((p, r)) = self.r.parse(input) {
            return Ok((p, r))
        } else { Err(()) }
    }
}


#[test] fn test_basic() {
    enum Ops {
        Mul, Div, Add, Sub
    }

    let num = parse_regex("[0-9]+").map(|s| s.parse::<i32>());
    let op  = ('*').or('/').or('+').or('-').map(|c| {
        match c {
            '*' => Ops::Mul,
            '/' => Ops::Div,
            '+' => Ops::Add,
            '-' => Ops::Sub,
             _  => unreachable!()
        }
    });

    /*
    let parse_foo = parse_str("foo");
    let parse_bar = parse_str("bar");
    let parse_under = parse_char('_');

    let parse_under_bar = concat(&parse_under, &parse_bar);
    let parse_foo_bar = concat(&parse_foo, &parse_under_bar);

    let input = StrGenerator{st: "foo_bar"};

    if let Ok((_, s)) = parse_foo(input.clone()) {
        assert_eq!(s, "foo".to_string())
    }

    if let Ok((_, s)) = parse_foo_bar(input) {
        assert_eq!(s, ("foo".to_string(), ('_', "bar".to_string())));
    }*/
}
