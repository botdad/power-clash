use clap::Parser;
use rayon::prelude::*;
use std::str;
use std::time::Instant;
use tiny_keccak::{Hasher, Keccak};

// from ethers-rs
// https://github.com/gakonst/ethers-rs/blob/2b94066cd5884c6a97ee9ab56042e7148182cf46/ethers-core/src/utils/hash.rs#L40
pub fn get_signature_hash(signature: &str) -> [u8; 4] {
    let mut output = [0u8; 4];

    let mut hasher = Keccak::v256();
    hasher.update(signature.as_bytes());
    hasher.finalize(&mut output);

    output
}

/// Finds matching function signatures for a given 4 byte signature hash in the format
/// PrefixRndChars(args)
#[derive(Parser, Debug)]
#[clap(
    name = "power-clash",
    version = "1.0",
    author = "github.com/botdad twitter.com/MEV_Dad"
)]
struct PowerClash {
    /// Target 4 byte signature hash. Ex: fa461e33
    #[clap(short, long, default_value = "00000000")]
    sighash: String,

    /// Arguments string from the target function signature. Ex: addres,address,bytes
    #[clap(short, long)]
    arg_signature: String,

    /// Method name prefix before random string. Ex: LolSwap (for computed LolSwapAd75(address,address,bytes))
    #[clap(short, long)]
    prefix: String,

    /// Character set to use for random string
    #[clap(
        short,
        long,
        default_value = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
    )]
    char_set: String,

    /// Length of random string
    #[clap(short, long, default_value = "6")]
    rnd_len: u32,
}

fn main() {
    let opts = PowerClash::parse();
    let sighash_string = opts.sighash.to_string();
    let sighash_bytes = hex::decode(sighash_string).expect("Decoding failed");

    let char_set_bytes = opts.char_set.as_bytes();

    let num_possible_chars = opts.char_set.len();
    let max_permutations = num_possible_chars.pow(opts.rnd_len);

    println!(
        "Attempting to find {}*****({}) match for 0x{} in {} max permutations",
        opts.prefix, opts.arg_signature, opts.sighash, max_permutations
    );

    let start = Instant::now();
    (0..max_permutations).into_par_iter().find_any(|&j| {
        let mut test_rnd_string: Vec<u8> = Vec::with_capacity(opts.rnd_len as usize);
        let mut i: u32 = 0;

        while i < opts.rnd_len {
            let char_index_to_use = j / num_possible_chars.pow(i) % num_possible_chars;
            test_rnd_string.push(char_set_bytes[char_index_to_use]);
            i = i + 1;
        }

        let test_function_signature = format!(
            "{}{}({})",
            opts.prefix,
            str::from_utf8(&test_rnd_string).unwrap(),
            opts.arg_signature
        );

        let test_signature_hash = get_signature_hash(&test_function_signature);

        if sighash_bytes.eq(&test_signature_hash) {
            let duration = start.elapsed();
            println!("FOUND match in {:?}", duration);

            println!(
                "{} should match 0x{}",
                test_function_signature, opts.sighash
            );
            return true;
        }
        return false;
    });
}
