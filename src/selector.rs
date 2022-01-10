use crate::prelude::*;
use nom::IResult;

pub type MResult<T> = std::result::Result<T, MatcherError>;

#[derive(PartialEq, Eq, Debug)]
pub struct MatcherError {
    msg: String
}

#[derive(PartialEq, Eq, Debug)]
pub struct MatcherParseError {
    msg: String
}

macro_rules! matcher_err {
    ($($expr: expr),+) => {
        Err(MatcherError{ msg: std::format!($($expr),+) })
    }
}

macro_rules! mname {
    ($expr:expr) => {
        Matcher::Name(NameMatcher(($expr).to_owned()))
    }
}

macro_rules! mindex {
    ($expr:expr) => {
        Matcher::Index(IndexMatcher($expr))
    }
}

macro_rules! mchain {
    [$($expr:expr),+] => {
        MatcherChain(vec![
            $($expr),+
        ])
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct NameMatcher(String);
impl NameMatcher {
    fn try_match(&self, v: &Value) -> MResult<Value> {
        match v {
            Value::Map(m) => {
                if let Some(v) = m.get(&self.0) {
                    Ok(v.clone())
                } else {
                    matcher_err!("map don't contains key what is '{}'", &(self.0))
                }
            }
            _ => matcher_err!("value isn't a map")
        }
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        use nom::{
            character::complete::*,
            combinator::*,
        };
        let mut parser = map_res(alphanumeric1, |s: &str| {
            if nom::character::is_alphabetic(s.bytes().nth(0).unwrap()) {
                Ok(NameMatcher(s.to_owned()))
            } else {
                Err("name of field must starts with an alphabet.")
            }
        });

        parser(input)
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct IndexMatcher(usize);
impl IndexMatcher {
    fn try_match(&self, v: &Value) -> MResult<Value> {
        match v {
            Value::List(ls) => {
                if self.0 < ls.len() {
                    Ok(ls[self.0].clone())
                } else {
                    matcher_err!("out of index what is {}", self.0)
                }
            }
            _ => matcher_err!("value isn't a list")
        }
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        use nom::{
            bytes::complete::*,
            character::complete::*,
            combinator::*,
            sequence::*,
        };
        let mut parser = delimited(
            tag("["),
            map_res(digit1, |s: &str| {
                s.parse::<usize>().map(|idx| IndexMatcher(idx))
            }),
            tag("]"),
        );
        parser(input)
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Matcher {
    Name(NameMatcher),
    Index(IndexMatcher),
}

impl Matcher {
    fn try_match(&self, v: &Value) -> MResult<Value> {
        match self {
            Matcher::Name(m) => m.try_match(v),
            Matcher::Index(m) => m.try_match(v),
        }
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        use nom::{
            branch::*,
            combinator::*,
        };
        let mut parser = alt((
            map(NameMatcher::parse, |m| Matcher::Name(m)),
            map(IndexMatcher::parse, |m| Matcher::Index(m)),
        ));

        parser(input)
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct MatcherChain(Vec<Matcher>);

impl MatcherChain {
    pub fn try_match(&self, v: &Value) -> MResult<Value> {
        let mut v = v.clone();
        for m in self.0.iter() {
            v = m.try_match(&v)?;
        }
        Ok(v)
    }

    pub fn parse(input: &str) -> IResult<&str, Self> {
        use nom::{
            bytes::complete::*,
            multi::*,
            combinator::*,
        };
        let mut parser = map(
            separated_list1(tag("."), Matcher::parse),
            |vec| MatcherChain(vec),
        );

        parser(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;
    mod name_matcher {
        use super::*;

        #[test]
        fn try_match() {
            let m = NameMatcher("hoge".to_owned());
            let v = vmap!{
                "hoge" => vint!(10)
            };
            assert!(m.try_match(&vunit!()).is_err());
            assert_eq!(m.try_match(&v).unwrap(), vint!(10));
        }

        #[test]
        fn parse() {
            let res = NameMatcher::parse("hoge1234");
            assert_eq!(res, Ok(
                ("", NameMatcher("hoge1234".to_owned()))
            ));

            let res = NameMatcher::parse("1234hoge");
            assert!(res.is_err());
        }
    }

    mod index_matcher {
        use super::*;

        #[test]
        fn try_match() {
            let m1 = IndexMatcher(1);
            let m10 = IndexMatcher(10);
            let v = vlist![
                vint!(1),
                vint!(2),
                vint!(3)
            ];

            assert!(m1.try_match(&vunit!()).is_err());
            assert_eq!(m1.try_match(&v).unwrap(), vint!(2));
            assert!(m10.try_match(&v).is_err());
        }

        #[test]
        fn parse() {
            let res = IndexMatcher::parse("[0]");
            assert_eq!(res, Ok(
                ("", IndexMatcher(0))
            ));
        }
    }

    mod matcher_chain {
        use super::*;

        #[test]
        fn try_match() {
            let m = mchain![
                mname!("hoge"),
                mindex!(0)
            ];

            let v = vmap!{
                "hoge" => vlist![
                    vint!(10)
                ]
            };

            assert_eq!(m.try_match(&v), Ok(vint!(10)));
        }

        #[test]
        fn parse() {
            let (rest, res) = MatcherChain::parse("hoge.[0]").unwrap();
            assert_eq!(rest, "");
            assert_eq!(res, mchain![
                mname!("hoge"),
                mindex!(0)
            ]);
        }
    }
}
