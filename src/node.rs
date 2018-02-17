use std::hash::Hash;
use std::hash::Hasher;

use hash::MerkleHasher;
use hash::BuildMerkleHasher;

pub type Child<V, S> = Option<Box<Node<V, S>>>;
pub type HashValue = Option<Box<[u8]>>;

pub enum Node<V, S>
where 
    V: Hash,
    S: BuildMerkleHasher
{
    Leaf(Leaf<V, S>),
    Branch(Branch<V, S>)
}

pub struct Leaf<V, S>
where
    V: Hash,
    S: BuildMerkleHasher
{
    value: V,
    hasher_builder: S,
    hash: HashValue  
}

pub struct Branch<V, S>
where
    V: Hash,
    S: BuildMerkleHasher
{
    left: Child<V, S>,
    right: Child<V, S>,
    hasher_builder: S,
    hash: HashValue        
}

impl<V, S> Node<V, S>
where 
    V: Hash,
    S: BuildMerkleHasher
{
    pub fn hash_value(&self) -> Box<[u8]> {
        match self {
            &Node::Leaf(ref leaf) => leaf.hash_value(),
            &Node::Branch(ref branch) => branch.hash_value()
        }
    }
}

impl<V, S> Hash for Node<V, S>
where
    V: Hash,
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
    V: Hash,
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
    V: Hash,
    S: BuildMerkleHasher
{
    pub fn new(left: Child<V, S>, right: Child<V, S>, hasher_builder: S) -> Branch<V, S> {
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
}

impl<V, S> Hash for Branch<V, S>
where
    V: Hash,
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
            },
            (_, &Some(ref right)) => {
                right.hash(state);                
            },
            _ => ()
        }
    }
}