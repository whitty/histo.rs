use rust_decimal::prelude::*;

use super::Result;

pub struct Histogram {
    buckets: Vec<(String, i64)>,

    geom: Option<(usize, usize)>,
}

impl Default for Histogram {
    fn default() -> Self {
        Self {
            buckets: Default::default(),
            geom: None,
        }
    }
}

impl Histogram {

    pub fn new<T: Into<i64> + Copy>(buckets: &Vec<(T, &str)>) -> Histogram {
        return Self::new_it(&mut buckets.iter().map(|(x, title)| (title.to_string(), *x)));
    }

    pub fn new_it<T: Into<i64> + Copy, It: Iterator<Item = (String, T)>>(buckets: &mut It) -> Histogram {
        return Histogram { buckets: buckets.map( |(title, x)| (title, x.into())).collect(), ..Default::default() }
    }

    pub fn new_indexed_it<T: Into<i64> + Copy, It: Iterator<Item = T>>(buckets: &mut It) -> Histogram {
        return Histogram { buckets: buckets.enumerate().map( |(ix, x)| (ix.to_string(), x.into())).collect(), ..Default::default() }
    }

    pub fn new_indexed<T: Into<i64> + Copy>(buckets: &Vec<T>) -> Histogram {
        return Self::new_indexed_it(&mut buckets.iter().map(|x| *x));
    }

    pub fn set_geometry(&mut self, width: usize, height: usize) -> &mut Self {
        self.geom = Some((width, height));
        self
    }

    fn from_int_env(v: &str) -> Result<usize> {
        let v = std::env::var(v)?;
        let v = v.parse::<usize>()?;
        Ok(v)
    }

    pub fn set_auto_geometry(&mut self, height: usize) -> &mut Self {
        let width = Histogram::from_int_env("COLUMNS").unwrap_or(72);
        self.geom = Some((width, height));
        self
    }

    pub fn draw(&self) -> String {
        let mut graph = if self.geom.is_some() {
            let g = self.geom.unwrap();
            asciigraph::Graph::new(g.0, g.1)
        } else {
            asciigraph::Graph::default()
        };

        graph.set_1d_labeled_data(self.buckets.clone())
            .set_skip_values(asciigraph::SkipValue::None)
            .set_y_min(0)
            .draw()
    }
}

#[derive(Debug)]
pub struct Buckets {
    count : usize,
    min: Option<Decimal>,
    max: Option<Decimal>,
}

impl Buckets {
    pub fn set_count(&mut self, c: usize) -> &mut Self {
        self.count = c;
        self
    }

    pub fn analyse(&mut self, v: &Vec<Decimal>) -> &mut Self {
        self.min = v.iter().min().map(|x| x.clone());
        self.max = v.iter().max().map(|x| x.clone());
        self
    }

    pub fn linear_buckets(&self) -> Vec<(Decimal, Decimal, String)> {
        let span = self.max.expect("max not set") - self.min.expect("min not set");
        let delta = span / Decimal::new(self.count as i64, 0);
        let min = self.min.unwrap();
        let two = Decimal::new(2, 0);

        (0..=self.count).map(|n| {
            let s = min + (Decimal::new(n as i64, 0) * delta);
            (s, s + delta, (s + (delta / two)).to_string())
        }).collect()
    }

    pub fn generate(&self, v: &Vec<Decimal>) -> std::collections::BTreeMap<String, i64> {
        let mut map = std::collections::BTreeMap::new();
        let buckets = self.linear_buckets();
        for b in &buckets {
            map.entry(b.2.clone()).or_insert(0);
        }
        let span = self.max.expect("max not set") - self.min.expect("min not set");
        let delta = span / Decimal::new(self.count as i64, 0);
        let min = self.min.unwrap();
        for val in v {
            let x = ((val - min) / delta).floor();
            let x = x.to_i32().unwrap() as usize;
            assert!(x < buckets.len());
            *map.get_mut(&buckets[x].2).unwrap() += 1;
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw() {
        // TODO - not a test
        let h = Histogram::new_indexed(&vec![100, 200, 300, 400, 200, 100]);
        let s = h.draw();
        println!("{}", s);
        assert_eq!(s, "438 |                                                \n    |                                                \n    |                        ████████                \n384 |                        ████████                \n    |                        ████████                \n    |                        ████████                \n330 |                        ████████                \n    |                ▄▄▄▄▄▄▄▄████████                \n    |                ████████████████                \n276 |                ████████████████                \n    |                ████████████████                \n    |                ████████████████                \n222 |                ████████████████                \n    |        ████████████████████████████████        \n    |        ████████████████████████████████        \n168 |        ████████████████████████████████        \n    |        ████████████████████████████████        \n    |        ████████████████████████████████        \n114 |▄▄▄▄▄▄▄▄████████████████████████████████▄▄▄▄▄▄▄▄\n    |████████████████████████████████████████████████\n    |████████████████████████████████████████████████\n 60 |████████████████████████████████████████████████\n    |████████████████████████████████████████████████\n    |████████████████████████████████████████████████\n    └------------------------------------------------\n     0       1       2       3       4       5       \n");

        let h = Histogram::new(&vec![(100, "1-5"), (200, "6-10"), (300, "11-15"), (400, "16-20"), (200, "21-25"), (100, "25-30"), (0, "31-35")]);
        let s = h.draw();
        println!("{}", s);
        assert_eq!(s, "451 |                                                \n    |                                                \n    |                     ▄▄▄▄▄▄▄                    \n394 |                     ███████                    \n    |                     ███████                    \n    |                     ███████                    \n337 |                     ███████                    \n    |              ▄▄▄▄▄▄▄███████                    \n    |              ██████████████                    \n280 |              ██████████████                    \n    |              ██████████████                    \n    |              ██████████████                    \n223 |              ██████████████                    \n    |       ████████████████████████████             \n    |       ████████████████████████████             \n166 |       ████████████████████████████             \n    |       ████████████████████████████             \n    |       ████████████████████████████             \n109 |██████████████████████████████████████████      \n    |██████████████████████████████████████████      \n    |██████████████████████████████████████████      \n 52 |██████████████████████████████████████████      \n    |██████████████████████████████████████████      \n    |██████████████████████████████████████████▄▄▄▄▄▄\n    └------------------------------------------------\n     1-5     6-10    11-15   16-20   21-25   25-30   \n");
    }

}
