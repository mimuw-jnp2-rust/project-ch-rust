fn add_one(x: i32) -> i32 {
    x + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dummy_add_one() {
        assert_eq!(add_one(0), 1);
    }
}
