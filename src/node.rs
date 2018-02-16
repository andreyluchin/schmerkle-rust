use digest::Digest;

pub type Child<'a> = Option<&'a Node<'a>>;

pub struct Node<'a> {
    hasher: &'a Digest,
    left: Child<'a>,
    right: Child<'a>
}

impl<'a> Node<'a> {
    fn height(&self) -> usize {
        match (self.left, self.right) {
            (Some(left), Some(right)) => {
                let (left_h, right_h) = (left.height(), right.height());
                if left_h >= right_h {
                    left_h
                } else {
                    right_h
                }
            },
            (Some(left), _) => left.height(),
            (_, Some(right)) => right.height(),
            _ => 0
        }
    }
}