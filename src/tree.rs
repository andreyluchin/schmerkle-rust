use std::hash::Hash;
use std::collections::VecDeque;
use std::iter::FromIterator;

use hash::{BuildMerkleHasher, MerkleHasher};
use node::{Node, Branch, Leaf, Child};


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

    pub fn insert_items<T>(&mut self, hashbuilder: S, items: T)
    where
        T: IntoIterator<Item=V>
    {
        for item in items {
            let leaf = Box::new(Node::new_leaf(item, self.hasher_builder.clone()));
            self.nodes.push_back(leaf)
        } 
    }

    pub fn root_hash(&self) -> Option<Box<[u8]>> {
        if let Some(ref root) = self.root {
            let mut hasher = self.hasher_builder.build_hasher();
            root.hash(&mut hasher);
            Some(hasher.finish_full())
        } else {
            None
        }
    }

    fn rebuild_tree(&mut self) {
        let nodes_len = self.nodes_len();
        match self.root {
            _ if self.nodes.is_empty() => self.root = None,
            Some(ref node) if node.len() == self.nodes_len() => (),
            Some(ref node) if node.len() > self.nodes_len() => unimplemented!(),
            _ => {
                let root = self.build_tree(f64::log2(nodes_len as f64) as usize);
                if let Some(ref unwrapped_root) = root {
                    self.recycle(unwrapped_root)
                };
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
        self.nodes = VecDeque::new();
        if let &box Node::Branch(ref branch) = root {
            match branch.left() {
                &Some(box Node::Branch(ref left)) if left.is_final() =>
                    self.nodes.push_back(Box::new(Node::Branch((*left).clone()))),
                &Some(box Node::Branch(ref left)) =>
                    self.recycle(&Box::new(Node::Branch((*left).clone()))),
                &Some(ref leaf) => self.nodes.push_back((*leaf).clone()),
                _ => ()
            };
            match branch.right() {
                &Some(box Node::Branch(ref right)) if right.is_final() =>
                    self.nodes.push_back(Box::new(Node::Branch((*right).clone()))),
                &Some(box Node::Branch(ref right)) =>
                    self.recycle(&Box::new(Node::Branch((*right).clone()))),
                &Some(ref leaf) => self.nodes.push_back((*leaf).clone()),
                _ => ()           
            };
        } else {
            self.nodes.push_back((*root).clone())
        }
        
    }

    fn nodes_len(&self) -> usize {
        self.nodes.iter()
            .fold(0, |acc, child| acc + child.len())
    }
}
