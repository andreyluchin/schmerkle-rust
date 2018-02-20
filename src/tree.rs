use std::hash::Hash;
use std::collections::VecDeque;
use std::fmt;

use hash::{BuildMerkleHasher, MerkleHasher};
use node::{Node, Child};


pub enum Proof {
    Left(Box<[u8]>),
    Right(Box<[u8]>)    
}


pub struct MerkleTree<V, S>
where 
    V: Hash + Clone,
    S: BuildMerkleHasher
{
    hasher_builder: S,
    nodes: VecDeque<Child<V, S>>,
    root: Option<Child<V,S>>
}

impl<V, S> MerkleTree<V, S>
where
    V: Hash + Clone,
    S: BuildMerkleHasher
{
    pub fn with_hasher(hasher_builder: S) -> MerkleTree<V, S> {
        MerkleTree {
            hasher_builder: hasher_builder,
            nodes: VecDeque::new(),
            root: None
        }
    }

    pub fn insert(&mut self, value: V) {
        self.nodes.push_back(Box::new(Node::new_leaf(value, self.hasher_builder.clone())));
        self.rebuild_tree();
    }

    pub fn insert_items<T>(&mut self, items: T)
    where
        T: IntoIterator<Item=V>
    {
        for item in items {
            let leaf = Box::new(Node::new_leaf(item, self.hasher_builder.clone()));
            self.nodes.push_back(leaf)
        }
        self.rebuild_tree();
    }

    pub fn root_hash(&self) -> Option<Box<[u8]>> {
        if let Some(ref root) = self.root {
            Some(root.hash_value())
        } else {
            println!("Root is none");
            None
        }
    }

    pub fn value_proof(&self, value: &V) -> Vec<Proof> {
        if let Some(ref root) = self.root {
            let mut hasher = self.hasher_builder.build_hasher();
            value.hash(&mut hasher);
            self.data_proof(hasher.finish_full(), 0, root)
        } else {
            vec![]
        }
    }

    pub fn tree_proof(&self, tree: MerkleTree<V, S>) -> Vec<Proof> {
        if let Some(root_hash) = tree.root_hash() {
            if let Some(ref root) = self.root {
                self.data_proof(root_hash, 0, root)
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    pub fn height(&self) -> usize {
        if let Some(ref root) = self.root {
            root.height()
        } else {
            0
        }
    }

    pub fn leaf_count(&self)-> Option<usize> {
        if let Some(ref root) = self.root {
            Some(root.leaf_count())
        } else {
            None
        }
    }

    fn data_proof(&self, hash: Box<[u8]>, height: usize, root: &Box<Node<V, S>>) -> Vec<Proof> {
        let root_height = root.height();
        if height + 1 == root_height {
            match (root.left(), root.right()) {
                (&Some(box ref left), &Some(box ref right)) => {
                    if *left.hash_value() == *hash {
                        vec![Proof::Right(right.hash_value())]
                    } else if *right.hash_value() == *hash {
                        vec![Proof::Left(left.hash_value())]
                    } else {
                        vec![]
                    }
                },
                (&Some(box ref left), _) => {
                    if *left.hash_value() == *hash {
                        vec![Proof::Right(left.hash_value())]
                    } else {
                        vec![]
                    }
                },
                (_, &Some(box ref right)) => {
                    if *right.hash_value() == *hash {
                        vec![Proof::Left(right.hash_value())]
                    } else {
                        vec![]
                    }
                },
                _ => vec![]
            }
        } else {
            match (root.left(), root.right()) {
                (&Some(ref left), &Some(ref right)) => {
                    let mut vec = Vec::new();
                    vec.extend(self.data_proof(hash.clone(), height, left));
                    vec.extend(self.data_proof(hash.clone(), height, right));
                    vec
                },
                (&Some(ref left), _) => {
                    let mut vec = Vec::new();
                    vec.extend(self.data_proof(hash.clone(), height, left));
                    vec
                },
                (_, &Some(ref right)) => {
                    let mut vec = Vec::new();
                    vec.extend(self.data_proof(hash.clone(), height, right));
                    vec
                },
                _ => vec![]
            }
        }
    }

    fn rebuild_tree(&mut self) {
        let nodes_len = self.nodes_leaf_count();
        match self.root {
            _ if self.nodes.is_empty() => self.root = None,
            Some(ref node) if node.leaf_count() >= self.nodes_leaf_count() => (),
            _ => {
                let root = self.build_tree(f64::log2(nodes_len as f64).ceil() as usize);
                self.nodes = VecDeque::new();
                if let Some(ref unwrapped_root) = root {
                    self.recycle(unwrapped_root)
                }
                self.root = root;
            }
        };
    }

    fn build_tree(&mut self, height: usize) -> Option<Child<V, S>> {
        let front_height = self.nodes.front()?.height();
        if front_height == height {
            Some(self.nodes.pop_front().unwrap())
        } else {
            let left = self.build_tree(height - 1);
            let right = self.build_tree(height - 1);
            Some(Box::new(Node::new_branch(left, right, self.hasher_builder.clone())))
        }
    }

    fn recycle(&mut self, root: &Child<V, S>) {
        if root.is_final() {
            self.nodes.push_back(root.clone())
        } else {
            if let &Some(ref left) = root.left() {
                self.recycle(left);
            }
            if let &Some(ref right) = root.right() {
                self.recycle(right);
            }
        }
    }

    fn nodes_leaf_count(&self) -> usize {
        self.nodes.iter()
            .fold(0, |acc, child| acc + child.leaf_count())
    }
}

impl<V, S> fmt::Display for MerkleTree<V, S>
where
    V: Hash + Clone,
    S: BuildMerkleHasher 
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.root {
            Some(ref root) => root.fmt(f),
            _ => write!(f, "")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::transmute;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{BuildHasherDefault, Hasher, Hash};
    use std::str;

    use tree::{MerkleTree, Proof};
    use hash::{MerkleHasher, BuildMerkleHasher};


    #[derive(Clone, Hash)]
    struct TestStruct(u64);

    impl MerkleHasher for DefaultHasher {
        fn finish_full(&self) -> Box<[u8]> {
            let slice: [u8;8] = unsafe { transmute((*self).finish().to_be()) };
            Box::new(slice)
        }
    }

    impl BuildMerkleHasher for BuildHasherDefault<DefaultHasher> {
        type Hasher = DefaultHasher;
        fn build_hasher(&self) -> DefaultHasher {
            DefaultHasher::default()
        }
    }

    fn make_tree() -> MerkleTree<TestStruct, BuildHasherDefault<DefaultHasher>> {
        let mut tree = MerkleTree::with_hasher(BuildHasherDefault::default());
        tree.insert_items(vec![TestStruct(0), TestStruct(1), TestStruct(2), TestStruct(3), TestStruct(4), TestStruct(5), TestStruct(6)]);
        tree
    }
    
    fn make_small_tree() -> MerkleTree<TestStruct, BuildHasherDefault<DefaultHasher>> {
        let mut tree = MerkleTree::with_hasher(BuildHasherDefault::default());
        tree.insert_items(vec![TestStruct(1), TestStruct(2)]);
        tree
    }

    #[test]
    fn test_height() {
        let small_tree = make_small_tree();        
        let tree = make_tree();        
        println!("Small tree height = {}", small_tree.height());
        println!("{}", small_tree);
        assert!(small_tree.height() == 1); 
        println!("Tree height = {}", tree.height());
        println!("{}", tree);              
        assert!(tree.height() == 3);
    }

    #[test]
    fn test_leaf_count() {
        let first = make_tree();
        let mut second = make_tree();
        println!("{} vs {}", first.leaf_count().unwrap(),second.leaf_count().unwrap());
        assert!(first.leaf_count().unwrap() == second.leaf_count().unwrap());
        second.insert(TestStruct(0));
        println!("{} vs {}", first.leaf_count().unwrap(),second.leaf_count().unwrap());        
        assert!(first.leaf_count().unwrap() + 1 == second.leaf_count().unwrap());
    }

    #[test]
    fn test_tree_hashes() {
        println!("RUNNING TEST");
        let first = make_tree();
        let mut second = make_tree();
        let first_hash = first.root_hash().unwrap();
        let mut second_hash = second.root_hash().unwrap();
        for &byte in first_hash.as_ref() {
            print!("{:X}", byte);
        }
        print!(" vs ");
        for &byte in second_hash.as_ref() {
            print!("{:X}", byte);
        }
        println!();
        assert!(*first_hash == *second_hash);
        second.insert(TestStruct(0));
        second_hash = second.root_hash().unwrap();
        for &byte in first_hash.as_ref() {
            print!("{:X}", byte);
        }
        print!(" vs ");
        for &byte in second_hash.as_ref() {
            print!("{:X}", byte);
        }
        println!();
        assert!(*first_hash != *second_hash);
    }

    #[test]
    fn test_value_proof() {
        let value = TestStruct(3);
        let hasher_builder = BuildHasherDefault::default();

        let tree = make_tree();
        let proof = tree.value_proof(&value);
        assert!(!proof.is_empty());

        let mut current_hash;
        match proof[0] {
            Proof::Left(ref hash) => {
                let mut hasher = hasher_builder.build_hasher();
                hasher.write(hash.as_ref());
                value.hash(&mut hasher);                   
                current_hash = hasher.finish_full();
            },
            Proof::Right(ref hash) => {
                let mut hasher = hasher_builder.build_hasher();
                value.hash(&mut hasher);                   
                hasher.write(hash.as_ref());                                       
                current_hash = hasher.finish_full();
            }
        }

        for piece in proof[1..].iter() {
            match piece {
                &Proof::Left(ref hash) => {
                    let mut hasher = hasher_builder.build_hasher();
                    hasher.write(hash.as_ref());
                    hasher.write(current_hash.as_ref());                    
                    current_hash = hasher.finish_full();
                },
                &Proof::Right(ref hash) => {
                    let mut hasher = hasher_builder.build_hasher();
                    hasher.write(current_hash.as_ref());
                    hasher.write(hash.as_ref());                                       
                    current_hash = hasher.finish_full();
                }
            }
            
        }

        let root_hash = tree.root_hash().unwrap();
        for &byte in current_hash.as_ref() {
            print!("{:X}", byte);
        }
        print!(" vs ");
        for &byte in root_hash.as_ref() {
            print!("{:X}", byte);
        }
        println!();
        assert!(*current_hash == *root_hash)
    }
    
}