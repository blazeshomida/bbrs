use std::time::Instant;

use super::attacks::*;

pub struct MagicPRNG {
    state: u32,
}

impl MagicPRNG {
    pub fn new() -> Self {
        Self { state: 1804289383 }
    }

    fn rand_32(&mut self) -> u32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;
        self.state
    }

    fn rand_64(&mut self) -> u64 {
        let (n1, n2, n3, n4) = (
            (self.rand_32() & 0xFFFF) as u64,
            (self.rand_32() & 0xFFFF) as u64,
            (self.rand_32() & 0xFFFF) as u64,
            (self.rand_32() & 0xFFFF) as u64,
        );

        n1 | (n2 << 16) | (n3 << 32) | (n4 << 48)
    }

    pub fn rand_magic(&mut self) -> u64 {
        self.rand_64() & self.rand_64() & self.rand_64()
    }
}

#[allow(dead_code)]
fn find_magic_number(rng: &mut MagicPRNG, square: u8, is_bishop: bool) -> Result<u64, &str> {
    let (mask, bits) = if is_bishop {
        (
            mask_bishop_attacks(square),
            BISHOP_RELEVANT_BITS[square as usize],
        )
    } else {
        (
            mask_rook_attacks(square),
            ROOK_RELEVANT_BITS[square as usize],
        )
    };
    let variations = 1 << bits;
    let mut occupancies = vec![0; variations];
    let mut attacks = vec![0; variations];
    (0..variations).for_each(|index| {
        occupancies[index] = create_occupancy(index, mask, bits);
        attacks[index] = if is_bishop {
            generate_bishop_attacks(square, occupancies[index])
        } else {
            generate_rook_attacks(square, occupancies[index])
        };
    });
    for _ in 0..1_000_000_000 {
        let magic = rng.rand_magic();

        if count_bits!((mask.wrapping_mul(magic)) & 0xFF00000000000000) < 6 {
            continue;
        };

        let mut used = vec![0; variations];

        let mut fail = false;
        for index in 0..variations {
            let magic_index = ((occupancies[index].wrapping_mul(magic)) >> (64 - bits)) as usize;
            if used[magic_index] == 0 {
                used[magic_index] = attacks[index];
            }
            if used[magic_index] != attacks[index] {
                fail = true;
                break;
            }
        }
        if !fail {
            println!("{:#X},", magic);
            return Ok(magic);
        }
    }

    Err("failed to find magic number")
}

#[allow(dead_code)]
fn find_magic_numbers() {
    let mut rng = MagicPRNG::new();
    let now = Instant::now();
    println!("Rook magics:");
    (0..64).for_each(|square| {
        find_magic_number(&mut rng, square, false).unwrap();
    });
    println!();
    println!("Bishop magics:");
    (0..64).for_each(|square| {
        find_magic_number(&mut rng, square, true).unwrap();
    });
    println!("Total time: {:?}", now.elapsed());
}
