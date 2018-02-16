use digest::Digest;

pub type Child<'a> = Option<&'a Node<'a>>;

pub enum Node<'a> {
    Branch(Branch<'a>),
    Leaf(Branch<'a>),
}

impl<'a> Node<'a> {
    fn height(&self) -> usize {
        if let &Node::Branch(ref b) = self {
            b.height()
        } else {
            0
        }
    }
}

pub struct Branch<'a> {
    hasher: &'a Digest,
    left: Child<'a>,
    right: Child<'a>
}

impl<'a> Branch<'a> {
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