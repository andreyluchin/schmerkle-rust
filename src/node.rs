use digest::Digest;

pub type Child<'a> = Option<Box<Node<'a>>>;

pub struct Node<'a> {
    hasher: &'a Digest,
    left: Child<'a>,
    right: Child<'a>
}

impl<'a> Node<'a> {
    fn new(hasher: &'a Digest) -> Node {
        Node {
            hasher: hasher,
            left: None,
            right: None
        }
    }

    fn height(&self) -> usize {
        match (&self.left, &self.right) {
            (&Some(ref left), &Some(ref right)) => {
                let (left_h, right_h) = (left.height(), right.height());
                if left_h >= right_h {
                    left_h
                } else {
                    right_h
                }
            },
            (&Some(ref left), _) => left.height(),
            (_, &Some(ref right)) => right.height(),
            _ => 0
        }
    }
}