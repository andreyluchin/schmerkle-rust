#![feature(box_patterns)]
mod hash;
mod node;
mod tree;
mod proof;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
