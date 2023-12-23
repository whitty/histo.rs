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

fn time_diff_parse<I>(inp: I, time_select: &Regex, filter_reg: &Option<Regex>) -> Vec<Decimal>
where
    I: Iterator<Item = String>
{
    let mut v: Vec<Decimal> = vec![];
    let mut prev: Option<Decimal> = None;
    for x in inp {
        if let Some(filter) = filter_reg {
            if !filter.is_match(&x) {
                continue
            }
        }
        let time = time_from(x.as_str(), time_select);
        if let Some(now) = time {
            if let Some(p) = prev {
                v.push(now - p);
            }
            prev = time;
        }
    }
    v
}

pub fn time_diff_load(inp: Vec<String>, time_select: &Regex, filter_reg: &Option<Regex>) -> Vec<Decimal> {
    time_diff_parse(LineVisitor::new(inp), time_select, filter_reg)
}

pub fn simple_load(inp: Vec<String>) ->std::collections::BTreeMap<String, i64> {
    simple_load_w_filter(inp, &None)
}

pub fn simple_load_w_filter(inp: Vec<String>, filter_reg: &Option<Regex>) ->std::collections::BTreeMap<String, i64> {
    simple_load_w_filter_in(LineVisitor::new(inp), filter_reg)
}

// Actual implementation of simple loads
fn simple_load_w_filter_in<I>(inp: I, filter_reg: &Option<Regex>) ->std::collections::BTreeMap<String, i64>
where
    I: Iterator<Item = String>
{
    let mut map = std::collections::BTreeMap::new();
    for x in inp {
        if let Some(filter) = filter_reg {
            if !filter.is_match(&x) {
                continue
            }
        }
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

    fn ro(s: &str) -> Option<Regex> {
        Some(r(s))
    }

    #[test]
    fn test_time_from() {
        assert_eq!(time_from("0001.02: entry", &default_time()), d("1.02"));
        assert_eq!(time_from("10001.123456789: entry", &default_time()), d("10001.123456789"));
        assert_eq!(time_from("entry: 0001.02", &default_time()), None);
        assert_eq!(time_from("entry: 0001.02", &r(r".$")), None);
        assert_eq!(time_from("entry: 0001.02", &r(r"(.*) (\d+\.\d+)$")), None);
        assert_eq!(time_from("entry: 0001.02", &r(r"(.*) (?<time>\d+\.\d+)$")), d("1.02"));
        assert_eq!(time_from("0001.02", &r(r"\d")), None);
    }

    fn dec_v(v :Vec<&str>) -> Vec<Decimal> {
        v.iter().map(|x| Decimal::from_str_exact(x).unwrap()).collect()
    }

    #[test]
    fn test_diff_parse() {
        let d = include_str!("../tests/example.txt");

        assert_eq!(time_diff_parse(d.split('\n').map(|x| x.to_string()), &default_time(), &None),
                   dec_v(vec![
                       "576.1890", "161.7767", "120.1351", "42.0575", "953.7649",
                       "42.0574", "1079.9571", "102.1173", "306.1201", "107.9229",
                       "203.8183", "678.3242", "221.6361", "1248.1245", "383.9979",
                       "635.8485", "60.2755", "317.7319", "1890.1634", "390.0038",
                       "545.7759", "228.0426", "270.0844", "629.8757", "2892.1272",
                       "396.0098", "785.7978", "204.2189", "545.9591", "143.7864"]));

        assert_eq!(time_diff_parse(d.split('\n').map(|x| x.to_string()), &r(r"(\d+)"), &None),
                   dec_v(vec![
                       "577", "161", "121", "42", "953", "42", "1080",
                       "102", "307", "108", "203", "679", "221",
                       "1248", "384", "636", "61", "317", "1890",
                       "390", "546", "228", "270", "630", "2892",
                       "396", "786", "204", "546", "144"
                   ]));

        assert_eq!(time_diff_parse(d.split('\n').map(|x| x.to_string()), &default_time(), &ro("ABC")),
                   dec_v(vec![]));

        assert_eq!(time_diff_parse(d.split('\n').map(|x| x.to_string()), &default_time(), &ro(r"^\d{5}")),
                   dec_v(vec![ "390.0038",
                       "545.7759", "228.0426", "270.0844", "629.8757", "2892.1272",
                       "396.0098", "785.7978", "204.2189", "545.9591", "143.7864"]));
    }

    #[test]
    fn test_simple_load() {
        let d = include_str!("../tests/seq.txt");
        let data = simple_load_w_filter_in(d.split('\n').map(|x| x.to_string()), &None);
        assert_eq!(data.len(), 21); // 1..20 + empty line
        assert!(data.into_values().all(|x| x == 1)); // all values are unique

        let data = simple_load_w_filter_in(d.split('\n').map(|x| x.to_string()), &ro("2"));
        assert_eq!(data.len(), 3); // 20, 12, 2
    }
}
