use std::result::Result;
use std::io::Error;

// TODO - visit lines
pub fn read<F>(input: &Vec<String>, f: F) -> Result<(), Error>
where
    F: Fn(Box<dyn std::io::Read>)
{
    if input.is_empty() {
        f(Box::new(std::io::stdin()));
    } else {
        for i in input {
            let file = std::fs::File::open(i)?;
            f(Box::new(file));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    //use std::io::Cursor;

    use super::*;

    #[test]
    fn test_read() {
        let result = read(&vec![], |_| {});
        println!("result={:?}", result);
    }

}
