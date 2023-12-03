
pub struct Histogram {
    buckets: Vec<(String, i64)>,
}

impl Histogram {

    pub fn new<T: Into<i64> + Copy>(buckets: &Vec<(T, &str)>) -> Histogram {
        return Histogram { buckets: buckets.iter().map( |(x, ix)| (ix.to_string(), (*x).into())).collect() }
    }

    pub fn new_indexed<T: Into<i64> + Copy>(buckets: &Vec<T>) -> Histogram {
        return Histogram { buckets: buckets.iter().enumerate().map( |(ix, x)| (ix.to_string(), (*x).into())).collect() }
    }

    pub fn draw(&self) -> String {
        asciigraph::Graph::new(72, 25)
            .set_1d_labeled_data(self.buckets.clone())
            .draw()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw() {
        let h = Histogram::new_indexed(&vec![100, 200, 300, 400, 200, 100]);
        println!("{}", h.draw());

        let h = Histogram::new(&vec![(100, "1-5"), (200, "6-10"), (300, "11-15"), (400, "16-20"), (200, "21-25"), (100, "25-30"), (0, "31-35")]);
        println!("{}", h.draw());
        // TODO - not a test
    }

}
