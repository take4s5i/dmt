use crate::prelude::*;
use nom::IResult;

pub struct SelectorResult = Box<dyn std::iter::Iterator<Item = Value>>;

#[derive(PartialEq, Eq, Debug)]
pub struct SelectorError {
    msg: String
}

#[derive(PartialEq, Eq, Debug)]
pub struct SelectorParseError {
    msg: String
}

macro_rules! selector_err {
    ($($expr: expr),+) => {
        Box::new(Some(Err(SelectorError{ msg: std::format!($($expr),+) })))
    }
}

macro_rules! empty_result {
    () => {
        Box::new(None.into_iter())
    }
}

macro_rules! single_result {
    ($expr:expr) => {
        Box::new(Some($expr).into_iter())
    }
}

macro_rules! sname {
    ($expr:expr) => {
        UnionSelector::Name(NameSelector(($expr).to_owned()))
    }
}

macro_rules! sindex {
    ($expr:expr) => {
        UnionSelector::Index(IndexSelector($expr))
    }
}

macro_rules! sel {
    [$($expr:expr),+] => {
        Selector::from_vec(vec![
            $($expr),+
        ])
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct NameSelector(String);
impl NameSelector {
    fn try_match(&self, v: &Value) -> SelectorResult {
        match v {
            Value::Map(m) => {
                if let Some(v) = m.get(&self.0) {
                    single_result!(v.clone())
                } else {
                    empty_result!()
                }
            }
            _ => empty_result!()
        }
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        use nom::{
            character::complete::*,
            combinator::*,
        };
        let mut parser = map_res(alphanumeric1, |s: &str| {
            if nom::character::is_alphabetic(s.bytes().nth(0).unwrap()) {
                Ok(NameSelector(s.to_owned()))
            } else {
                Err("name of field must starts with an alphabet.")
            }
        });

        parser(input)
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct IndexSelector(usize);
impl IndexSelector {
    fn try_match(&self, v: &Value) -> SelectorResult {
        match v {
            Value::List(ls) => {
                if self.0 < ls.len() {
                    single_result!(ls[self.0].clone())
                } else {
                    empty_result!()
                }
            }
            _ => empty_result!()
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
                s.parse::<usize>().map(|idx| IndexSelector(idx))
            }),
            tag("]"),
        );
        parser(input)
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum UnionSelector {
    Name(NameSelector),
    Index(IndexSelector),
}

impl UnionSelector {
    fn try_match(&self, v: &Value) -> SelectorResult {
        match self {
            Self::Name(m) => m.try_match(v),
            Self::Index(m) => m.try_match(v),
        }
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        use nom::{
            branch::*,
            combinator::*,
        };
        let mut parser = alt((
            map(NameSelector::parse, |m| Self::Name(m)),
            map(IndexSelector::parse, |m| Self::Index(m)),
        ));

        parser(input)
    }
}


#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Selector {
    Nil,
    Node(UnionSelector, Box<Selector>),
}

impl Selector {
    pub fn from_vec(sels: Vec<UnionSelector>) -> Selector {
        let mut sel = Self::Nil;

        for uni in sels.into_iter().rev() {
            sel = Self::Node(uni, Box::new(sel));
        }

        sel
    }

    pub fn try_match(&self, v: &Value) -> SelectorResult {
        match self.clone() {
            Self::Nil => single_result!(v.clone()),
            Self::Node(sel, next) => {
                let iter = sel.try_match(v)
                    .flat_map(move |child| next.try_match(&child));
                Box::new(iter)
            },
        }
    }

    pub fn parse(input: &str) -> IResult<&str, Self> {
        use nom::{
            bytes::complete::*,
            multi::*,
            combinator::*,
        };
        let mut parser = map(
            separated_list1(tag("."), UnionSelector::parse),
            |vec| Selector::from_vec(vec),
        );

        parser(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;
    mod name_selector {
        use super::*;

        #[test]
        fn try_match() {
            let m = NameSelector("hoge".to_owned());
            let v = vmap!{
                "hoge" => vint!(10)
            };

            let res: Vec<_> = m.try_match(&vunit!()).collect();
            assert!(res.is_empty());

            let res: Vec<_> = m.try_match(&v).collect();
            assert_eq!(res.len(), 1);
            assert_eq!(res[0], vint!(10));
        }

        #[test]
        fn parse() {
            let res = NameSelector::parse("hoge1234");
            assert_eq!(res, Ok(
                ("", NameSelector("hoge1234".to_owned()))
            ));

            let res = NameSelector::parse("1234hoge");
            assert!(res.is_err());
        }
    }

    mod index_selector {
        use super::*;

        #[test]
        fn try_match() {
            let m1 = IndexSelector(1);
            let m10 = IndexSelector(10);
            let v = vlist![
                vint!(1),
                vint!(2),
                vint!(3)
            ];

            let res: Vec<_> = m1.try_match(&vunit!()).collect();
            assert!(res.is_empty());

            let res: Vec<_> = m1.try_match(&v).collect();
            assert_eq!(res.len(), 1);
            assert_eq!(res[0], vint!(2));

            let res: Vec<_> = m10.try_match(&v).collect();
            assert!(res.is_empty());
        }

        #[test]
        fn parse() {
            let res = IndexSelector::parse("[0]");
            assert_eq!(res, Ok(
                ("", IndexSelector(0))
            ));
        }
    }

    mod selector {
        use super::*;

        #[test]
        fn try_match() {
            let m = sel![
                sname!("hoge"),
                sindex!(0)
            ];

            let v = vmap!{
                "hoge" => vlist![
                    vint!(10)
                ]
            };

            let res: Vec<_> = m.try_match(&v).collect();
            assert_eq!(res.len(), 1);
            assert_eq!(res[0], vint!(10));
        }

        #[test]
        fn parse() {
            let (rest, res) = Selector::parse("hoge.[0]").unwrap();
            assert_eq!(rest, "");
            assert_eq!(res, sel![
                sname!("hoge"),
                sindex!(0)
            ]);
        }
    }
}
