#![feature(box_patterns)]
#![allow(dead_code)]
mod hash;
mod node;
mod tree;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
