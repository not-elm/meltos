fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod tests {
    #[test]
    fn sample() {
        assert_eq!(1, 1);
    }
}
