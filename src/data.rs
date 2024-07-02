// SPDX-License-Identifier: GPL-3.0-or-later
// (C) Copyright 2023-2024 Greg Whiteley

use std::io::BufRead;
use std::collections::{VecDeque, HashMap};
use regex::Regex;
use rust_decimal::prelude::*;

use super::{Result, Error};

type InputType = std::io::BufReader<Box<dyn std::io::Read>>;
type LinesT = std::io::Lines<InputType>;

// Cat-like access to lines
struct LineVisitor {
    input: VecDeque<String>,
    curr: Option<LinesT>,
}

impl LineVisitor {
    fn new(inp: Vec<String>) -> LineVisitor {
        if inp.is_empty() {
            // return stdin
            return LineVisitor { input: VecDeque::new(), curr: open_stdio() };
        }
        LineVisitor { input: VecDeque::from(inp), curr: None }
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
            Some(InputType::new(Box::new(f)).lines())
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
            if let Some(curr) = &mut self.curr {
                if let Some(Ok(next)) = curr.next() {
                    return Some(next)
                }
                self.curr = None
            };
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

pub fn select_load(inp: Vec<String>, selector: &Regex) ->std::collections::BTreeMap<String, i64> {
    select_load_in(LineVisitor::new(inp), selector)
}

fn apply_selector(s: String, selector: &Regex) -> Option<String> {
    if let Some(c) = selector.captures(s.as_str()) {
        if let Some(select) = c.name("select").or_else(|| c.get(1)) {
            return Some(String::from(select.as_str()))
        }
    }
    None
}

fn select_load_in<I>(inp: I, selector: &Regex) ->std::collections::BTreeMap<String, i64>
where
    I: Iterator<Item = String>
{
    simple_load_w_filter_in(inp.filter_map(|s| apply_selector(s, selector)), &None)
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

pub fn scoped_time_load(inp: Vec<String>, time_select: &Regex, scoped_in: &Regex, scoped_out: &Regex) -> Vec<Decimal> {
    scoped_time_parse(LineVisitor::new(inp), time_select, scoped_in, scoped_out)
}

pub fn scoped_match_time_load(inp: Vec<String>, time_select: &Regex, scoped_in: &Regex, scoped_out: &Regex) -> Result<Vec<Decimal>> {
    if scoped_in.captures_len() != scoped_out.captures_len() {
        return Err(Error::ScopedMatchCountError(scoped_in.as_str().into(), scoped_out.as_str().into()));
    }

    // if not using matched context we can do optimisation by avoiding match handling
    if scoped_in.captures_len() == 1 {
        return Ok(scoped_time_load(inp, time_select, scoped_in, scoped_out));
    }

    Ok(scoped_match_time_parse(LineVisitor::new(inp), time_select, scoped_in, scoped_out))
}

fn scoped_match_time_parse<I>(inp: I, time_select: &Regex, scoped_in: &Regex, scoped_out: &Regex) -> Vec<Decimal>
where
    I: Iterator<Item = String>
{
    type Key = Vec<String>;
    fn match_to_key(regex: &Regex, line: &str) -> Option<Key> {
        regex.captures(line)
            .map(|the_match| the_match
                 .iter()
                 .skip(1)
                 .map(|y| y.unwrap().as_str().into())
                 .collect())
    }

    let mut v: Vec<Decimal> = vec![];
    let mut prev: HashMap<Key, Vec<Decimal>> = HashMap::new();

    let symmetric: bool = std::ptr::eq(scoped_in, scoped_out) || (scoped_in.as_str() == scoped_out.as_str());

    for x in inp {
        if let Some(now) = time_from(x.as_str(), time_select) {
            if let Some(match_key) = match_to_key(scoped_in, &x) {
                prev.entry(match_key)
                    .or_default()
                    .push(now);

                if symmetric {
                    // Don't look for end match?
                    // maybe always continue?
                    continue;
                }
            }

            if let Some(match_key) = match_to_key(scoped_out, &x) {
                if let Some(then) = prev.get_mut(&match_key).and_then(Vec::<_>::pop) {
                    v.push(now - then);
                } else {
                    println!(" not matched {:?}", match_key);
                }
            }
        }
    }
    v
}

fn scoped_time_parse<I>(inp: I, time_select: &Regex, scoped_in: &Regex, scoped_out: &Regex) -> Vec<Decimal>
where
    I: Iterator<Item = String>
{
    let mut v: Vec<Decimal> = vec![];
    let mut prev: Vec<Decimal> = vec![];
    for x in inp {
        let time = time_from(x.as_str(), time_select);
        if let Some(now) = time {
            if scoped_in.is_match(&x) {
                prev.push(now);
            } else if scoped_out.is_match(&x) {
                if let Some(then) = prev.pop() {
                    v.push(now - then);
                } else {
                    println!(" not matched");
                }
            }
        }
    }
    v
}


#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn default_time() -> Regex {
        Regex::new(r"^(\d+\.\d+)").expect("regex failed to compile")
    }

    fn strace_time_ignore_mins_for_now() -> Regex {
        Regex::new(r"^\d+:\d+:(\d+\.\d+)").expect("regex failed to compile")
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

    // use BufReader::lines() like the main application does
    // (resolves windows issues)
    fn to_lines(s: &str) -> std::vec::IntoIter<String> {
        use std::io::{BufReader, Cursor};
        BufReader::new(Cursor::new(s))
            .lines()
            .map_while(|x| x.ok())
            .collect::<Vec<String>>().into_iter()
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

        assert_eq!(time_diff_parse(to_lines(d), &default_time(), &None),
                   dec_v(vec![
                       "576.1890", "161.7767", "120.1351", "42.0575", "953.7649",
                       "42.0574", "1079.9571", "102.1173", "306.1201", "107.9229",
                       "203.8183", "678.3242", "221.6361", "1248.1245", "383.9979",
                       "635.8485", "60.2755", "317.7319", "1890.1634", "390.0038",
                       "545.7759", "228.0426", "270.0844", "629.8757", "2892.1272",
                       "396.0098", "785.7978", "204.2189", "545.9591", "143.7864"]));

        assert_eq!(time_diff_parse(to_lines(d), &r(r"(\d+)"), &None),
                   dec_v(vec![
                       "577", "161", "121", "42", "953", "42", "1080",
                       "102", "307", "108", "203", "679", "221",
                       "1248", "384", "636", "61", "317", "1890",
                       "390", "546", "228", "270", "630", "2892",
                       "396", "786", "204", "546", "144"
                   ]));

        assert_eq!(time_diff_parse(to_lines(d), &default_time(), &ro("ABC")),
                   dec_v(vec![]));

        assert_eq!(time_diff_parse(to_lines(d), &default_time(), &ro(r"^\d{5}")),
                   dec_v(vec![ "390.0038",
                       "545.7759", "228.0426", "270.0844", "629.8757", "2892.1272",
                       "396.0098", "785.7978", "204.2189", "545.9591", "143.7864"]));
    }

    #[test]
    fn test_simple_load() {
        let d = include_str!("../tests/seq.txt");
        let data = simple_load_w_filter_in(to_lines(d), &None);
        assert_eq!(data.len(), 20); // 1..20
        assert!(data.into_values().all(|x| x == 1)); // all values are unique

        let data = simple_load_w_filter_in(to_lines(d), &ro("2"));
        assert_eq!(data.len(), 3); // 20, 12, 2
    }

    #[test]
    fn test_select_load() {
        let d = include_str!("../tests/seq.txt");
        let data = select_load_in(to_lines(d), &r(r"\d([0-4])"));
        assert_eq!(data, BTreeMap::from([
            (String::from("0"), 2), // 10, 20
            (String::from("1"), 1), // 11
            (String::from("2"), 1), // 12
            (String::from("3"), 1), // 13
            (String::from("4"), 1), // 14
        ]));
    }

    #[test]
    fn test_scoped_load_simple_in_out() {
        let d = include_str!("../tests/example_scoped.txt");
        let data = scoped_time_parse(to_lines(d), &default_time(),
                                     &r(r"->reset"), &r(r"<-reset"));
        assert_eq!(data, dec_v(vec![ "900.1583", "203.8183",]));

        let data = scoped_time_parse(to_lines(d), &default_time(),
                                     &r(r"->recurse"), &r(r"<-recurse"));
        assert_eq!(data, dec_v(vec![ "60.2755", "3288.0172", "5699.9640", "1.0000",]));
    }

    #[test]
    fn test_scoped_match_load_simple_in_out() {
        // Should be equivalent to non-match variant (matching key == '()')
        let d = include_str!("../tests/example_scoped.txt");
        let data = scoped_match_time_parse(to_lines(d), &default_time(),
                                     &r(r"->reset"), &r(r"<-reset"));
        assert_eq!(data, dec_v(vec![ "900.1583", "203.8183",]));

        let data = scoped_match_time_parse(to_lines(d), &default_time(),
                                     &r(r"->recurse"), &r(r"<-recurse"));
        assert_eq!(data, dec_v(vec![ "60.2755", "3288.0172", "5699.9640", "1.0000",]));

        // But should be able to categorise reset/recurse
        let data = scoped_match_time_parse(to_lines(d), &default_time(),
                                     &r(r"->(reset|recurse)"), &r(r"<-(reset|recurse)"));
        assert_eq!(data, dec_v(vec![ "900.1583", "203.8183",  "60.2755", "3288.0172", "5699.9640", "1.0000"]));
    }

    #[test]
    fn test_scoped_match_strace() {
        let d = include_str!("../tests/strace.txt");
        let data = scoped_match_time_parse(to_lines(d), &strace_time_ignore_mins_for_now(),
                                     &r(r"openat\(.*\) = (\d+)\z"), &r(r"close\((\d+)\)"));
        assert_eq!(data, dec_v(vec![ "0.000155", "0.000331", "0.000404", "0.000589", "0.000453", "0.000700"]));
    }
}
