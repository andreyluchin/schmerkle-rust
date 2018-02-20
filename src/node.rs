use std::hash::Hash;
use std::hash::Hasher;
use std::fmt;

use hash::MerkleHasher;
use hash::BuildMerkleHasher;

pub type Child<V, S> = Box<Node<V, S>>;
pub type HashValue = Box<[u8]>;

#[derive(Clone)]
pub enum Node<V, S>
where 
    V: Hash + Clone,
    S: BuildMerkleHasher
{
    Leaf(Leaf<V, S>),
    Branch(Branch<V, S>)
}

#[derive(Clone)]
pub struct Leaf<V, S>
where
    V: Hash,
    S: BuildMerkleHasher
{
    value: V,
    hasher_builder: S,
    hash: Option<HashValue>
}

#[derive(Clone)]
pub struct Branch<V, S>
where
    V: Hash + Clone,
    S: BuildMerkleHasher
{
    left: Option<Child<V, S>>,
    right: Option<Child<V, S>>,
    hasher_builder: S,
    hash: Option<HashValue>          
}

impl<V, S> Node<V, S>
where 
    V: Hash + Clone,
    S: BuildMerkleHasher
{
    pub fn new_branch(
        left: Option<Child<V, S>>, 
        right: Option<Child<V, S>>, 
        hasher_builder: S) -> Node<V, S> 
    {
        Node::Branch(Branch::new(left, right, hasher_builder))
    }

    pub fn new_leaf(value: V, hasher_builder: S) -> Node<V, S> {
        Node::Leaf(Leaf::new(value, hasher_builder))
    }

    pub fn hash_value(&self) -> Box<[u8]> {
        match self {
            &Node::Leaf(ref leaf) => leaf.hash_value(),
            &Node::Branch(ref branch) => branch.hash_value()
        }
    }

    pub fn len(&self) -> usize {
        match self {
            &Node::Leaf(_) => 1,
            &Node::Branch(ref branch) => branch.len()
        }
    }

    pub fn height(&self) -> usize {
        match self {
            &Node::Leaf(_) => 0,
            &Node::Branch(ref branch) => branch.height()
        }
    }

    pub fn is_final(&self) -> bool {
        match self {
            &Node::Leaf(_) => true,
            &Node::Branch(ref branch) => branch.is_final()
        }
    }

    pub fn left(&self) -> &Option<Child<V, S>> {
        match self {
            &Node::Branch(ref branch) => branch.left(),
            _ => &None
        }
    }

    pub fn right(&self) -> &Option<Child<V, S>> {
        match self {
            &Node::Branch(ref branch) => branch.right(),
            _ => &None
        }
    }
}

impl<V, S> Hash for Node<V, S>
where
    V: Hash + Clone,
    S: BuildMerkleHasher
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            &Node::Leaf(ref leaf) => leaf.hash(state),
            &Node::Branch(ref branch) => branch.hash(state)
        }
    }
}

impl<V, S> Leaf<V, S>
where 
    V: Hash + Clone,
    S: BuildMerkleHasher
{
    pub fn new(value: V, hasher_builder: S) -> Leaf<V, S> {
        let mut leaf = Leaf {
            value: value,
            hasher_builder: hasher_builder,
            hash: None
        };
        let mut hasher = leaf.hasher_builder().build_hasher();
        leaf.hash(&mut hasher);
        leaf.hash = Some(hasher.finish_full());
        leaf
    }

    pub fn hash_value(&self) -> Box<[u8]> {
        if let Some(ref hash) = self.hash {
            hash.clone()
        } else {
            panic!("Hash was not set for leaf object")
        }
    }

    pub fn hasher_builder(&self) -> &S {
        &self.hasher_builder
    }
}

impl<V, S> Hash for Leaf<V, S>
where
    V: Hash,
    S: BuildMerkleHasher
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state)
    }
}

impl<V, S> Branch<V, S>
where 
    V: Hash + Clone,
    S: BuildMerkleHasher
{
    pub fn new(left: Option<Child<V, S>>, right: Option<Child<V, S>>, hasher_builder: S) -> Branch<V, S> {
        let mut branch = Branch {
            left: left,
            right: right,
            hasher_builder: hasher_builder,
            hash: None
        };
        let mut hasher = branch.hasher_builder().build_hasher();
        branch.hash(&mut hasher);
        branch.hash = Some(hasher.finish_full());
        branch
    }

    pub fn hash_value(&self) -> Box<[u8]> {
        if let Some(ref hash) = self.hash {
            hash.clone()
        } else {
            panic!("Hash was not set for leaf object")
        }
    }

    pub fn hasher_builder(&self) -> &S {
        &self.hasher_builder
    }

    pub fn len(&self) -> usize {
        match (&self.left, &self.right) {
            (&Some(ref left), &Some(ref right)) => left.len() + right.len(),
            (&Some(ref left), _) => left.len(),
            (_, &Some(ref right)) => right.len(),
            _ => 0
        }
    }

    pub fn height(&self) -> usize {
        match (&self.left, &self.right) {
            (&Some(ref left), &Some(ref right)) => bigger(left.height(), right.height()) + 1,
            (&Some(ref left), _) => left.height() + 1,
            (_, &Some(ref right)) => right.height() + 1,
            _ => 0
        }
    }

    pub fn is_final(&self) -> bool {
        match (&self.left, &self.right) {
            (&Some(ref left), &Some(ref right)) => left.is_final() && right.is_final(),
            _ => false
        }
    }

    pub fn left(&self) -> &Option<Child<V, S>> {
        &self.left
    }

    pub fn right(&self) -> &Option<Child<V, S>> {
        &self.right
    }
}

impl<V, S> Hash for Branch<V, S>
where
    V: Hash + Clone,
    S: BuildMerkleHasher
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        match (&self.left, &self.right) {
            (&Some(ref left), &Some(ref right)) => {
                left.hash(state);
                right.hash(state)
            },
            (&Some(ref left), _) => {
                left.hash(state);
                left.hash(state);                
            },
            (_, &Some(ref right)) => {
                right.hash(state);
                right.hash(state);                                                
            },
            _ => ()
        }
    }
}

fn bigger(first: usize, second: usize) -> usize {
    if first >= second {
        first
    } else {
        second
    }
}

impl<V, S> fmt::Display for Node<V, S>
where
    V: Hash + Clone,
    S: BuildMerkleHasher 
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &byte in self.hash_value().as_ref() {
            write!(f, "{:X}", byte)?;
        }
        write!(f, "\n");
        if let &Some(ref left) = self.left() {
            for &byte in left.hash_value().as_ref() {
                write!(f, "{:X}", byte)?;
            }
            write!(f, " === ");
        };
        if let &Some(ref right) = self.right() {
            for &byte in right.hash_value().as_ref() {
                write!(f, "{:X}", byte)?;
            }
        };
        if let &Some(ref left) = self.left() {
            write!(f, "\n");            
            left.fmt(f)?;
        };
        if let &Some(ref right) = self.right() {
            write!(f, "\n");                    
            right.fmt(f)?;
        };

        write!(f, "")
    }
}