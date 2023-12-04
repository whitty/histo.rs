
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
        return Self::new_it(&mut buckets.iter().map(|(x, title)| (*x, title.to_string())));
    }

    pub fn new_it<T: Into<i64> + Copy, It: Iterator<Item = (T, String)>>(buckets: &mut It) -> Histogram {
        return Histogram { buckets: buckets.map( |(x, title)| (title, x.into())).collect(), ..Default::default() }
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
