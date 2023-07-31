#[cfg(test)]
mod block_tests {
    // use crate::queue::{Task, WorkQueue};
    use crate::block::Block;
    // use std::sync::atomic::{AtomicUsize, Ordering};
    // use std::time::{Duration, Instant};
    // use std::{sync, thread, time};
    use std::fmt::Write;

    #[test]
    fn example() {
        assert_eq!(1 + 1, 2);
    }

    #[test]
    fn create_hash_for_proof() {
        let mut b0 = Block::initial(16);
        let b0_hash_string = b0.hash_string_for_proof(56231);
        assert_eq!("0000000000000000000000000000000000000000000000000000000000000000:0:16::56231", b0_hash_string);
        let b0_hash_for_proof = b0.hash_for_proof(56231);
        let mut b0_hashed_string = String::new();
        write!(&mut b0_hashed_string, "{:02x}", b0_hash_for_proof).unwrap();
        assert_eq!("6c71ff02a08a22309b7dbbcee45d291d4ce955caa32031c50d941e3e9dbd0000", b0_hashed_string);
        println!("Take 2");
        b0.set_proof(56231);
        let b1 = Block::next(&b0, String::from("message"));
        println!("{:?}", b0);
        println!("{:?}", b1);
        let b1_hash_string = b1.hash_string_for_proof(2159);
        assert_eq!("6c71ff02a08a22309b7dbbcee45d291d4ce955caa32031c50d941e3e9dbd0000:1:16:message:2159", b1_hash_string);
        let b1_hash_for_proof = b1.hash_for_proof(2159);
        let mut b1_hashed_string = String::new();
        write!(&mut b1_hashed_string, "{:02x}", b1_hash_for_proof).unwrap();
        assert_eq!("9b4417b36afa6d31c728eed7abc14dd84468fdb055d8f3cbe308b0179df40000", b1_hashed_string);
    }

    #[test]
    fn is_valid_for_proof() {
        let mut b0 = Block::initial(19);
        b0.set_proof(87745);
        assert_eq!(true, b0.is_valid_for_proof(87745));
        let mut b1 = Block::next(&b0, String::from("hash example 1234"));
        b1.set_proof(1407891);
        assert_eq!(true, b1.is_valid_for_proof(1407891));
        b1.set_proof(346082);
        assert_eq!(false, b1.is_valid_for_proof(346082));
    }

    #[test]
    fn basic_mine() {
        let mut b0 = Block::initial(7);
        b0.mine(1);
        println!("{}", b0.hash_string());
        println!("{:02x}", b0.hash());
        let mut b1 = Block::next(&b0, String::from("this is an interesting message"));
        b1.mine(1);
        println!("{}", b1.hash_string());
        println!("{:02x}", b1.hash());
        let mut b2 = Block::next(&b1, String::from("this is not interesting"));
        b2.mine(1);
        println!("{}", b2.hash_string());
        println!("{:02x}", b2.hash());
    }
}
