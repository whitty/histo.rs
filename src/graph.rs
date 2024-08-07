// SPDX-License-Identifier: GPL-3.0-or-later
// (C) Copyright 2023-2024 Greg Whiteley

use rust_decimal::prelude::*;

use super::{Result, Error};

#[derive(Default)]
pub struct Histogram {
    buckets: Vec<(String, i64)>,

    width: Option<usize>,

    show_counts: bool,
}

impl Histogram {

    const COLUMNS_DEFAULT: usize = 72;

    pub fn new<T: Into<i64> + Copy>(buckets: &[(T, &str)]) -> Histogram {
        return Self::new_it(&mut buckets.iter().map(|(x, title)| (title.to_string(), *x)));
    }

    pub fn new_it<T, It>(buckets: &mut It) -> Histogram
    where
        T: Into<i64> + Copy,
        It: Iterator<Item = (String, T)>
    {
        Histogram { buckets: buckets.map( |(title, x)| (title, x.into())).collect(), ..Default::default() }
    }

    pub fn new_indexed_it<T, It>(buckets: &mut It) -> Histogram
    where
        T: Into<i64> + Copy,
        It: Iterator<Item = T>
    {
        Histogram { buckets: buckets.enumerate().map( |(ix, x)| (ix.to_string(), x.into())).collect(), ..Default::default() }
    }

    pub fn new_indexed<T: Into<i64> + Copy>(buckets: &[T]) -> Histogram {
        return Self::new_indexed_it(&mut buckets.iter().copied());
    }

    pub fn set_width(&mut self, width: usize) -> &mut Self {
        self.width = Some(width);
        self
    }

    pub fn set_opt_width(&mut self, width: Option<usize>) -> &mut Self {
        match width {
            Some(width) => self.set_width(width),
            None => self.set_auto_width()
        }
    }

    fn from_int_env(v: &str) -> Result<usize> {
        let v = std::env::var(v)?;
        let v = v.parse::<usize>()?;
        Ok(v)
    }

    // Its unclear if this really works - COLUMNS isn't exported by default
    fn get_terminal_columns_from_env() -> Option<usize> {
        Self::from_int_env("COLUMNS").ok()
    }

    #[cfg(not(feature = "terminal"))]
    fn get_hw_terminal_columns() -> Option<usize> {
        None
    }

    #[cfg(feature = "terminal")]
    fn get_hw_terminal_columns() -> Option<usize> {
        use terminal_size::{Width, terminal_size};
        terminal_size()
            .map(|(Width(w),_)| w.into())
    }

    fn get_terminal_columns() -> usize {

        // Hmm,... terminal_size crate doesn't seem to honour COLUMNS, but does find
        // more dimensions than termion did.  So we need to check the env _first_.
        Self::get_terminal_columns_from_env()
            .or_else(Self::get_hw_terminal_columns)
            .unwrap_or(Self::COLUMNS_DEFAULT)
    }

    pub fn set_auto_width(&mut self) -> &mut Self {
        self.width = Some(Self::get_terminal_columns());
        self
    }

    pub fn show_counts(&mut self) -> &mut Self {
        self.set_show_counts(true)
    }

    pub fn set_show_counts(&mut self, val: bool) -> &mut Self {
        self.show_counts = val;
        self
    }

    pub fn hide_counts(&mut self) -> &mut Self {
        self.set_show_counts(false)
    }

    fn scale(v: i64, min: i64, max: i64, width: usize) -> usize {
        // Clamp v to max
        let v = v.min(max);
        // Take v as relative to min (and clamp to min)
        let v = v.abs_sub(&min) as u64;

        let delta = max.abs_diff(min);
        // to simulate round-to-nearest we add half the variance
        let round_offset = delta
            .checked_div(width as u64)
            .and_then(|x| x.checked_div(2))
            .unwrap_or(0);

        // scale v by delta:width
        (v + round_offset).checked_mul(width as u64)
            .and_then(|x| x.checked_div(delta))
            .and_then(|x| usize::try_from(x).ok())
            .unwrap_or(width)
    }

    fn count_size(&self) -> usize {
        if self.show_counts {
            if let Some(first) = self.buckets.first() {

                let mut min = first.1;
                let mut max = first.1;
                for x in &self.buckets {
                    max = x.1.max(max);
                    min = x.1.min(min);
                }

                return format!("{}", max).len().max(format!("{}", min).len());
            }
        }
        0
    }

    pub fn draw(&self) -> Result<String> {
        use std::{fmt::Write, ops::Div};

        let mut buf = String::new();

        let mut max_name_len = 8; // min
        let mut min_val = i64::MAX;
        let mut max_val = i64::MIN;
        for (name, v) in &self.buckets {
            max_name_len = max_name_len.max(name.len());
            min_val = min_val.min(*v);
            max_val = max_val.max(*v);
        }

        let zero_base = true; // TODO - derive this

        if zero_base && min_val > 0 {
            min_val = 0;
        }

        let term_columns = self.width
            .unwrap_or(Self::COLUMNS_DEFAULT);

        // no more than half the size
        let max_name_len = max_name_len.min(term_columns.div(2));
        let name_field_len = max_name_len + 1;

        let count_field_len = self.count_size();
        let count_field_allowance = if count_field_len > 0 {
            count_field_len + 2 // colon, space
        } else { count_field_len };

        let columns = term_columns
            .saturating_sub(name_field_len + 1 + count_field_allowance);

        if columns == 0 {
            return Err(Error::DataTagsTooLongToFitTerminal(term_columns));
        }

        // generate template of max required number of #'s
        let template = "#".repeat(columns);

        for (name, v) in &self.buckets {
            let count = Self::scale(*v, min_val, max_val, columns);

            write!(buf, "{:>max_name_len$}", &name[0..max_name_len.min(name.len())])?;

            if self.show_counts {
                write!(buf, ": {:>count_field_len$}", v)?;
            }

            // if value is literally zero against a zero base don't print anything (round to nothing)
            // otherwise it will always round to _at least one_
            if *v == 0 && zero_base {
                writeln!(buf)?;
            } else {
                let count = count.max(1); // always print at least one if we aren't zero
                writeln!(buf, " {}", &template[0..count])?;
            }
        }

        Ok(buf)
    }
}

#[derive(Debug)]
pub struct Buckets {
    count : usize,
    min: Option<Decimal>,
    max: Option<Decimal>,
    delta: Option<Decimal>,
}

impl Buckets {
    pub fn set_count(&mut self, c: usize) -> &mut Self {
        self.count = c;
        self
    }

    pub fn set_delta(&mut self, d: Decimal) -> &mut Self {
        self.delta = Some(d);
        self
    }

    pub fn set_delta_opt(&mut self, d: Option<Decimal>) -> &mut Self {
        self.delta = d;
        self
    }

    pub fn analyse(&mut self, v: &[Decimal]) -> &mut Self {
        self.min = v.iter().min().copied();
        self.max = v.iter().max().copied();
        if let (Some(delta), Some(min), Some(max)) = (self.delta, self.min, self.max) {
            let min = (min / delta).floor() * delta;
            let max = (max / delta).ceil() * delta;
            self.count = ((max - min) / delta).floor().to_i32().unwrap() as usize; // TODO = + 1?
            self.min = Some(min); self.max = Some(max);
        }
        self
    }

    fn linear_buckets(&self) -> Vec<(Decimal, Decimal)> {
        let span = self.max.expect("max not set") - self.min.expect("min not set");
        let delta = span / Decimal::new(self.count as i64, 0);
        let min = self.min.unwrap();
        (0..self.count).map(|n| {
            let s = min + (Decimal::new(n as i64, 0) * delta);
            let mut e = s + delta;
            if (n + 1) == self.count {
                e = self.max.unwrap();
            }
            (s, e)
        }).collect()
    }

    pub fn generate(&self, v: &[Decimal]) -> std::collections::BTreeMap<Decimal, i64> {
        let mut map = std::collections::BTreeMap::new();
        let buckets = self.linear_buckets();
        for b in &buckets {
            map.entry(b.1).or_insert(0);
        }
        let span = self.max.expect("max not set") - self.min.expect("min not set");
        let delta = span / Decimal::new(self.count as i64, 0);
        let min = self.min.unwrap();
        for val in v {
            let x = ((val - min) / delta).floor();
            let mut x = x.to_i32().unwrap() as usize;
            if x == buckets.len() && x >= 1 {
                // last value, but not first value
                if val <= &buckets[x-1].1 {
                    x -= 1;
                }
            }
            assert!(x < buckets.len(), "x {} < buckets.len() {}", x, buckets.len());
            *map.get_mut(&buckets[x].1).unwrap() += 1;
        }
        map
    }
}

impl Default for Buckets {
    fn default() -> Self {
        Self {
            count : 80,
            min: None,
            max: None,
            delta: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw() {
        // TODO - not a test
        let s = Histogram::new_indexed(&vec![100, 200, 300, 400, 200, 100]).draw().unwrap();
        println!("{}", s);
        assert_eq!(s, r#"       0 ###############
       1 ###############################
       2 ##############################################
       3 ##############################################################
       4 ###############################
       5 ###############
"#);

        let h = Histogram::new(&vec![(100, "1-5"), (200, "6-10"), (300, "11-15"), (400, "16-20"), (200, "21-25"), (100, "25-30"), (0, "31-35")]);
        let s = h.draw().unwrap();
        println!("{}", s);
        assert_eq!(s, r#"     1-5 ###############
    6-10 ###############################
   11-15 ##############################################
   16-20 ##############################################################
   21-25 ###############################
   25-30 ###############
   31-35
"#
 );

        // ensure non-zero doesn't get crushed to zero
        let h = Histogram::new(&vec![(100, "1-5"), (200, "6-10"), (300, "11-15"), (400, "16-20"), (200, "21-25"), (100, "25-30"), (1, "31-35")]);
        let s = h.draw().unwrap();
        println!("{}", s);
        assert_eq!(s, r#"     1-5 ###############
    6-10 ###############################
   11-15 ##############################################
   16-20 ##############################################################
   21-25 ###############################
   25-30 ###############
   31-35 #
"#
 );

        // ensure non-zero doesn't get crushed to zero
        let s = Histogram::new(&vec![
            (100, "1-5"),
            (200, "6-10"),
            (300, "11-15"),
            (400, "16-20"),
            (200, "21-25"),
            (100, "25-30"),
            (1, "31-35")
        ]).show_counts()
            .draw()
            .unwrap();
        println!("{}", s);
        assert_eq!(s, r#"     1-5: 100 ##############
    6-10: 200 ############################
   11-15: 300 ###########################################
   16-20: 400 #########################################################
   21-25: 200 ############################
   25-30: 100 ##############
   31-35:   1 #
"#
 );
    }

    fn dec_v(v :&[&str]) -> Vec<Decimal> {
        v.iter().map(|x| Decimal::from_str_exact(x).unwrap()).collect()
    }

    fn inc_range(begin: Decimal, delta: Decimal, count: usize) -> Vec<(Decimal, Decimal)> {
        (0..count).map(|n| {
            let s = begin + delta * Decimal::new(n as i64, 0);
            (s, s + delta)
        }).collect()
    }
    fn inc_range_s(begin: &str, delta: &str, count: usize) -> Vec<(Decimal, Decimal)> {
        inc_range(Decimal::from_str_exact(begin).unwrap(),
                  Decimal::from_str_exact(delta).unwrap(),
                  count)
    }

    #[test]
    fn test_linear_buckets() {
        let data:Vec<Decimal> = dec_v(&["1.0", "4.0"]);
        let buckets = Buckets::default()
            .set_count(3)
            .analyse(&data)
            .linear_buckets();
        assert_eq!(buckets.len(), 3);
        assert_eq!(buckets, vec![
            (Decimal::new(1,0), Decimal::new(2,0)),
            (Decimal::new(2,0), Decimal::new(3,0)),
            (Decimal::new(3,0), Decimal::new(4,0)),
        ]);
        assert_eq!(buckets, inc_range_s("1.0", "1.0", 3));

        let buckets = Buckets::default()
            .set_delta(Decimal::new(1,0))
            .analyse(&data)
            .linear_buckets();
        assert_eq!(buckets, inc_range_s("1.0", "1.0", 3));

        let data:Vec<Decimal> = dec_v(&["1.8", "3.1"]);
        let buckets = Buckets::default()
            .set_delta(Decimal::new(1,0))
            .analyse(&data)
            .linear_buckets();
        assert_eq!(buckets, inc_range_s("1.0", "1.0", 3));

        let data:Vec<Decimal> = dec_v(&["1.8", "4.1"]);
        let buckets = Buckets::default()
            .set_delta(Decimal::new(1,0))
            .analyse(&data)
            .linear_buckets();
        assert_eq!(buckets, inc_range_s("1.0", "1.0", 4));
    }

    #[test]
    fn test_scale() {
        // test in a simple range
        let scale = |val: i64| {
            Histogram::scale(val, 0, 50, 80)
        };
        assert_eq!(scale(0), 0);
        assert_eq!(scale(5), 8);
        assert_eq!(scale(25), 40);
        assert_eq!(scale(45), 72);
        assert_eq!(scale(50), 80);
        assert_eq!(scale(-10), 0);
        assert_eq!(scale(55), 80);

        // Repeat test with min/max/value offset - ie so that we move
        // relative to max-min range
        let scale = |val: i64| {
            let diff: i64 = 10;
            Histogram::scale(val - diff, 0 - diff, 50 - diff, 80)
        };
        assert_eq!(scale(0), 0);
        assert_eq!(scale(5), 8);
        assert_eq!(scale(25), 40);
        assert_eq!(scale(45), 72);
        assert_eq!(scale(50), 80);
        assert_eq!(scale(-10), 0);
        assert_eq!(scale(55), 80);

        // test large values in a smaller range range
        let scale = |val: i64| {
            Histogram::scale(val, 0, 1000, 80)
        };
        assert_eq!(scale(-1), 0);
        assert_eq!(scale(0), 0);
        assert_eq!(scale(100), 8);
        assert_eq!(scale(333), 27);
        assert_eq!(scale(500), 40);
        assert_eq!(scale(900), 72);
        assert_eq!(scale(1000), 80);
        assert_eq!(scale(1010), 80);

        // Repeat test with min/max/value offset - ie so that we move
        // relative to max-min range
        let scale = |val: i64| {
            Histogram::scale(val - 500, 0 - 500, 1000 - 500, 80)
        };
        assert_eq!(scale(-1), 0);
        assert_eq!(scale(0), 0);
        assert_eq!(scale(100), 8);
        assert_eq!(scale(333), 27);
        assert_eq!(scale(500), 40);
        assert_eq!(scale(900), 72);
        assert_eq!(scale(1000), 80);
        assert_eq!(scale(1010), 80);

        // test values from our example histogram in the draw test
        // Not really a test, just validating the values seem to look sane
        let scale = |val: i64| {
            Histogram::scale(val, 0, 400, 72 - 10) // header allowance is 10
        };

        assert_eq!(scale(-1), 0);
        assert_eq!(scale(0), 0);
        assert_eq!(scale(1), 0);
        assert_eq!(scale(100 - 1), 15);
        assert_eq!(scale(100 + 0), 15);
        assert_eq!(scale(100 + 1), 16);
        assert_eq!(scale(200 - 1), 31);
        assert_eq!(scale(200 + 0), 31);
        assert_eq!(scale(200 + 1), 31);
        assert_eq!(scale(300 - 1), 46);
        assert_eq!(scale(300 + 0), 46);
        assert_eq!(scale(300 + 1), 47);
        assert_eq!(scale(400 - 1), 62);
        assert_eq!(scale(400 + 0), 62);
        assert_eq!(scale(400 + 1), 62);
    }
}
