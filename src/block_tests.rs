#[cfg(test)]
mod block_tests {
    use crate::block::Block;
    use std::{fmt::Write, time::Instant};

    // Test correctness of Block::initial and Block::next
    // Test Block.hash_string_for_proof and Block.hash_for_proof
    #[test]
    fn create_hash_for_proof() {
        let mut b0 = Block::initial(16);
        let b0_hash_string = b0.hash_string_for_proof(56231);
        assert_eq!("0000000000000000000000000000000000000000000000000000000000000000:0:16::56231", b0_hash_string);
        let b0_hash_for_proof = b0.hash_for_proof(56231);
        let mut b0_hashed_string = String::new();
        write!(&mut b0_hashed_string, "{:02x}", b0_hash_for_proof).unwrap();
        assert_eq!("6c71ff02a08a22309b7dbbcee45d291d4ce955caa32031c50d941e3e9dbd0000", b0_hashed_string);
        b0.set_proof(56231);
        let b1 = Block::next(&b0, String::from("message"));
        let b1_hash_string = b1.hash_string_for_proof(2159);
        assert_eq!("6c71ff02a08a22309b7dbbcee45d291d4ce955caa32031c50d941e3e9dbd0000:1:16:message:2159", b1_hash_string);
        let b1_hash_for_proof = b1.hash_for_proof(2159);
        let mut b1_hashed_string = String::new();
        write!(&mut b1_hashed_string, "{:02x}", b1_hash_for_proof).unwrap();
        assert_eq!("9b4417b36afa6d31c728eed7abc14dd84468fdb055d8f3cbe308b0179df40000", b1_hashed_string);
    }

    // Test Block.is_valid_for_proof
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

    // Test MiningTask Implementation
    #[test]
    fn mining_task() {
        let mut b0_1 = Block::initial(10);
        b0_1.mine_serial();
        let mut b0_2 = Block::initial(10);
        b0_2.mine_serial_using_task();
        assert_eq!(b0_1.hash(), b0_2.hash());

    }

    // Test Block.mine_range with difficult = 7
    #[test]
    fn basic_mine() {
        let mut b0 = Block::initial(7);
        b0.mine(1);
        assert_eq!("0000000000000000000000000000000000000000000000000000000000000000:0:7::385", format!("{}", b0.hash_string()));
        assert_eq!("379bf2fb1a558872f09442a45e300e72f00f03f2c6f4dd29971f67ea4f3d5300", format!("{:02x}", b0.hash()));
        let mut b1 = Block::next(&b0, String::from("this is an interesting message"));
        b1.mine(1);
        assert_eq!("379bf2fb1a558872f09442a45e300e72f00f03f2c6f4dd29971f67ea4f3d5300:1:7:this is an interesting message:20", format!("{}", b1.hash_string()));
        assert_eq!("4a1c722d8021346fa2f440d7f0bbaa585e632f68fd20fed812fc944613b92500", format!("{:02x}", b1.hash()));
        let mut b2 = Block::next(&b1, String::from("this is not interesting"));
        b2.mine(1);
        assert_eq!("4a1c722d8021346fa2f440d7f0bbaa585e632f68fd20fed812fc944613b92500:2:7:this is not interesting:40", format!("{}", b2.hash_string()));
        assert_eq!("ba2f9bf0f9ec629db726f1a5fe7312eb76270459e3f5bfdc4e213df9e47cd380", format!("{:02x}", b2.hash()));
    }

    // Test Block.mine_range with difficult = 20
    #[test]
    fn difficult_mine() {
        let mut b0 = Block::initial(20);
        b0.mine(1);
        assert_eq!("0000000000000000000000000000000000000000000000000000000000000000:0:20::1209938", format!("{}", b0.hash_string()));
        assert_eq!("19e2d3b3f0e2ebda3891979d76f957a5d51e1ba0b43f4296d8fb37c470600000", format!("{:02x}", b0.hash()));
        let mut b1 = Block::next(&b0, String::from("this is an interesting message"));
        b1.mine(1);
        assert_eq!("19e2d3b3f0e2ebda3891979d76f957a5d51e1ba0b43f4296d8fb37c470600000:1:20:this is an interesting message:989099", format!("{}", b1.hash_string()));
        assert_eq!("a42b7e319ee2dee845f1eb842c31dac60a94c04432319638ec1b9f989d000000", format!("{:02x}", b1.hash()));
        let mut b2 = Block::next(&b1, String::from("this is not interesting"));
        b2.mine(1);
        assert_eq!("a42b7e319ee2dee845f1eb842c31dac60a94c04432319638ec1b9f989d000000:2:20:this is not interesting:1017262", format!("{}", b2.hash_string()));
        assert_eq!("6c589f7a3d2df217fdb39cd969006bc8651a0a3251ffb50470cbc9a0e4d00000", format!("{:02x}", b2.hash()));
    }

    // Test if assigning additional worker threads speeds up mining
    #[test]
    fn multiple_workers() {
        let start1 = Instant::now();
        let mut b0 = Block::initial(20);
        b0.mine(1);
        let mut b1 = Block::next(&b0, String::from("1 worker is far too slow"));
        b1.mine(1);
        let mut b2 = Block::next(&b1, String::from("we ought to have at least 5"));
        b2.mine(1);
        let end1 = Instant::now();

        let start2 = Instant::now();
        let mut b3 = Block::initial(20);
        b3.mine(5);
        let mut b4 = Block::next(&b3, String::from("1 worker is far too slow"));
        b4.mine(5);
        let mut b5 = Block::next(&b4, String::from("we ought to have at least 5"));
        b5.mine(5);
        let end2 = Instant::now();

        let time_taken_1_worker = end1.duration_since(start1).as_millis();
        let time_taken_5_workers = end2.duration_since(start2).as_millis();

        assert!(time_taken_5_workers < time_taken_1_worker);
    }

}
