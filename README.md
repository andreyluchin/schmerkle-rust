# schmerkle-rust
Merkle Tree implemented in Rust.

## Usage
1. Implement MerkleHasher
2. Implement BuildMerkleHasher
3. Implement Hash and Clone for your data
4. Ready to use

```rust
use std::mem::transmute;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{BuildHasherDefault, Hasher, Hash};

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
    
    fn main() {
        let mut tree = MerkleTree::with_hasher(BuildHasherDefault::default());
        tree.insert_items(vec![TestStruct(0), TestStruct(1), TestStruct(2), TestStruct(3), TestStruct(4), TestStruct(5), TestStruct(6)]);
        let proof = tree.value_proof(TestStruct(3));
    }
 ```
 
 ## Domain
 Merkle trees are mostly used in blockchains and some databases for data verification and consistency verification.
 Despite the general concept being roughly the same in all implementation, there are few things that change from one specification to another.
 This project addresses one of them: Custom Hashing.
 Other things like single node hashing (which are hashed twice in Schmerkle) cannot be changed.
 
 ## Philosophy
 Schmerkle embraces two concepts:
      1. Hashing interface akin to standard library (via Hash, Hasher, HasherBuilder traits), which makes it agnostic to hashing crates chosen by the user.
      2. The idea of `final nodes`
      
 ## Final Nodes
 Before explaining final nodes and why are they used in Schmerkle it is important to emphasize on the fact that values inserted in Merkle Tree preserve insertion order.
 This is done to make consistency verification possible.
 
 Final node is a node that has no siblingless leaves:
 ```
    01  
    /\    -- Final Node
   0  1  
   
   0123  
    /\  
   01 23     -- Final Node
  /\  /\  
  0 1 2 3  
  
  0 -- Single value (leaf) is a final node itself
  ```
  
  When insertion means rebuilding a tree with bigger height, final nodes don't change, so it would be wise to reuse them.
  This saves us from composing rehashing nodes over and over when it is not necessary.
