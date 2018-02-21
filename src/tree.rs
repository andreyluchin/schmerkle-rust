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

    pub fn root(&self) -> &Option<Child<V,S>> {
        &self.root
    }

    pub fn value_proof(&self, value: &V) -> Vec<Proof> {
        if let Some(ref root) = self.root {
            let node = Box::new(Node::new_leaf(value.clone(), self.hasher_builder.clone()));
            self.data_proof(&node, &mut vec![root])
        } else {
            vec![]
        }
    }

    pub fn tree_proof(&self, tree: MerkleTree<V, S>) -> Vec<Proof> {
        if let &Some(ref target) = tree.root() {
            if let Some(ref root) = self.root {
                self.data_proof(target,&mut vec![root])
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

    pub fn hasher_builder(&self) -> S {
        self.hasher_builder.clone()
    }

    fn data_proof<'a>(
        &self, target: &Child<V, S>, 
        parent_stack: &mut Vec<&'a Child<V, S>>) -> Vec<Proof> 
    {
        match *parent_stack.as_slice() {
            [] => vec![],

            [.., parent] if target.height() + 1 == parent.height() => {
                let parent = parent_stack.pop().unwrap();
                match (parent.left(), parent.right()) {
                    (&Some(ref left), &Some(ref right)) if left.hash_value() == target.hash_value() => {
                        let mut result = vec![Proof::Right(right.hash_value())];
                        result.append(&mut self.data_proof(parent, parent_stack));
                        result
                    },
                    (&Some(ref left), &Some(ref right)) if right.hash_value() == target.hash_value() => {
                        let mut result = vec![Proof::Left(left.hash_value())];
                        result.append(&mut self.data_proof(parent, parent_stack));
                        result
                    },
                    (&Some(ref left), _) if left.hash_value() == target.hash_value() => {
                        let mut result = vec![Proof::Right(left.hash_value())];
                        result.append(&mut self.data_proof(parent, parent_stack));
                        result
                    },
                    (_, &Some(ref right)) if right.hash_value() == target.hash_value() => {
                        let mut result = vec![Proof::Left(right.hash_value())];
                        result.append(&mut self.data_proof(parent, parent_stack));
                        result
                    },
                    _ => vec![]
                }
            },

            [.., parent] => {
                match (parent.left(), parent.right()) {
                    (&Some(ref left), &Some(ref right)) => {
                        parent_stack.push(left);
                        let mut result = self.data_proof(target, parent_stack);
                        parent_stack.push(right);
                        result.append(&mut self.data_proof(target, parent_stack));
                        result
                    },
                    (&Some(ref left), _) if left.hash_value() == target.hash_value() => {
                        parent_stack.push(left);
                        self.data_proof(target, parent_stack)
                    },
                    (_, &Some(ref right)) if right.hash_value() == target.hash_value() => {
                        parent_stack.push(right);
                        self.data_proof(target, parent_stack)
                    },
                    _ => vec![]
                }
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