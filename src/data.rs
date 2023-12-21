use std::io::{Error, BufRead, ErrorKind::NotFound};
use std::collections::VecDeque;
use regex::Regex;
use rust_decimal::prelude::*;

type InputType = std::io::BufReader<Box<dyn std::io::Read>>;
type LinesT = std::io::Lines<InputType>;

// Cat-like access to lines
pub struct LineVisitor {
    input: VecDeque<String>,
    curr: Option<LinesT>,
}

impl LineVisitor {
    pub fn new(inp: Vec<String>) -> LineVisitor {
        if inp.is_empty() {
            // return stdin
            return LineVisitor { input: VecDeque::new(), curr: open_stdio() };
        }
        return LineVisitor { input: VecDeque::from(inp), curr: None };
    }

    fn invalidate(&mut self) {
        self.curr = None;
        self.input.clear();
    }
}

fn open_stdio() -> Option<LinesT> {
    Some(InputType::new(Box::new(std::io::stdin().lock())).lines())
}

fn open(f: &String) -> Option<LinesT> {
    if f == "-" {
        return open_stdio();
    }
    let file = std::fs::File::open(f);
    match file {
        Ok(f) => {
            return Some(InputType::new(Box::new(f)).lines());
        },
        Err(e) => {
            // TODO - handle errors better
            panic!("Error opening {}: {}", f, e);
            //return None;
        }
    }
}

// Lines iterator
impl Iterator for LineVisitor {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {

        loop {

            // if we have nothing open try and open something
            if self.curr.is_none() {
                if let Some(f) = self.input.pop_front() {
                    self.curr = open(&f);
                    if self.curr.is_none() {
                        self.invalidate();
                        return None;
                    }
                } else {
                    return None;
                }
            }

            // If there's some data try and consume it
            if self.curr.is_some() {
                // get next line
                let next = self.curr.as_mut().and_then(|reader| {
                    reader.next()
                }).unwrap_or(Err(Error::from(NotFound)));

                if next.is_err() {
                    self.curr = None;
                } else {
                    return Some(next.unwrap())
                }
            }
        }
    }
}

fn time_from(s: &str, time_select: &Regex) -> Option<Decimal> {
    if let Some(time_match) = time_select.captures(s) {
        if let Some(time) = time_match.name("time").or_else(|| time_match.get(1)) {
            if let Ok(d) = Decimal::from_str_exact(time.as_str()) {
                return Some(d);
            }
        }
    }
    None
}

pub fn simple_load(inp: Vec<String>) ->std::collections::BTreeMap<String, i64> {
    let mut map = std::collections::BTreeMap::new();
    for x in LineVisitor::new(inp) {
        let val = map.entry(x).or_insert(0);
        *val += 1;
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_time() -> Regex {
        Regex::new(r"^(\d+\.\d+)").expect("regex failed to compile")
    }

    fn d(s: &str) -> Option<Decimal> {
        if let Ok(d) = Decimal::from_str(s) {
            return Some(d)
        }
        None
    }

    fn r(s: &str) -> Regex {
        Regex::new(s).expect(format!("regex failed to compile '{}'", s).as_str())
    }

    #[test]
    fn test_time_from() {
        assert_eq!(time_from("0001.02: entry", &default_time()), d("1.02"));
        assert_eq!(time_from("10001.123456789: entry", &default_time()), d("10001.123456789"));
        assert_eq!(time_from("entry: 0001.02", &default_time()), None);
        assert_eq!(time_from("entry: 0001.02", &r(r".$")), None);
        assert_eq!(time_from("entry: 0001.02", &r(r"(.*) (\d+\.\d+)$")), None);
        assert_eq!(time_from("entry: 0001.02", &r(r"(.*) (?<time>\d+\.\d+)$")), d("1.02"));
    }
}
