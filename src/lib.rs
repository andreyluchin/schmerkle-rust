#![feature(box_patterns)]
#![allow(dead_code)]
#![feature(slice_patterns)]
#![feature(advanced_slice_patterns)]
mod hash;
mod node;
mod tree;

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
        assert!(first_hash == second_hash);
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
        assert!(first_hash != second_hash);
    }

    #[test]
    fn test_value_proof() {
        let value = TestStruct(3);
        let hasher_builder = BuildHasherDefault::default();

        let tree = make_tree();
        let proof = tree.value_proof(&value);
        println!("Proof length: {}", proof.len());
        assert!(proof.len() == tree.height());

        print!("Proof: ");
        for piece in proof.iter() {
            if let &Proof::Left(ref hash) = piece {
                for &byte in hash.as_ref() {
                    print!("{:X}", byte);
                }
            } else if let &Proof::Right(ref hash) = piece {
                for &byte in hash.as_ref() {
                    print!("{:X}", byte);
                }
            };
            println!();
        }
        println!("\n{}\n", tree);

        let mut hasher = hasher_builder.build_hasher();
        value.hash(&mut hasher);
        let mut current_hash = hasher.finish_full();

        for piece in proof {
            match piece {
                Proof::Left(hash) => {
                    println!("Left: ");
                    for &byte in hash.as_ref() {
                        print!("{:X}", byte);
                    }
                    print!(" vs : ");
                    for &byte in current_hash.as_ref() {
                        print!("{:X}", byte);
                    }
                    println!();
                    let mut hasher = hasher_builder.build_hasher();
                    hasher.write(&hash);
                    hasher.write(&current_hash);
                    current_hash = hasher.finish_full();
                    println!("Current: ");
                    for &byte in current_hash.as_ref() {
                        print!("{:X}", byte);
                    }
                    println!();
                },
                Proof::Right(hash) => {
                    println!("Right: ");
                    for &byte in hash.as_ref() {
                        print!("{:X}", byte);
                    }
                    print!(" vs : ");
                    for &byte in current_hash.as_ref() {
                        print!("{:X}", byte);
                    }
                    println!();
                    let mut hasher = hasher_builder.build_hasher();
                    hasher.write(&current_hash);
                    hasher.write(&hash);                    
                    current_hash = hasher.finish_full();
                    println!("Current: ");
                    for &byte in current_hash.as_ref() {
                        print!("{:X}", byte);
                    }
                    println!();                    
                },
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
        assert!(current_hash == root_hash)
    }
    
}