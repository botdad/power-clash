use clap::Parser;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use std::{str, sync::atomic::AtomicUsize};
use tiny_keccak::{Hasher, Keccak};

#[derive(Debug)]
struct NotFoundError;

/// Finds matching function signatures for a given 4 byte signature hash in the format
/// PrefixRndChars(args)
#[derive(Parser, Debug)]
#[clap(name = "power-clash", version = "1.0", author = "github.com/botdad")]
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

fn build_signature(
    permutation_index: usize,
    capacity: usize,
    rnd_len: u32,
    char_set_bytes: &[u8],
    prefix: &str,
    arg_signature: &str,
) -> String {
    let mut test_rnd_string: Vec<u8> = Vec::with_capacity(rnd_len as usize);
    let num_possible_chars = char_set_bytes.len();

    let mut i: u32 = 0;
    while i < rnd_len {
        let char_index_to_use = permutation_index / num_possible_chars.pow(i) % num_possible_chars;
        test_rnd_string.push(char_set_bytes[char_index_to_use]);
        i += 1;
    }

    let mut test_function_signature = String::with_capacity(capacity);
    test_function_signature.push_str(prefix);
    test_function_signature.push_str(str::from_utf8(&test_rnd_string).unwrap());
    test_function_signature.push('(');
    test_function_signature.push_str(arg_signature);
    test_function_signature.push(')');

    test_function_signature
}

// from ethers-rs
// https://github.com/gakonst/ethers-rs/blob/2b94066cd5884c6a97ee9ab56042e7148182cf46/ethers-core/src/utils/hash.rs#L40
pub fn hash_signature(signature: &String) -> [u8; 4] {
    let mut output = [0u8; 4];

    let mut hasher = Keccak::v256();
    hasher.update(signature.as_bytes());
    hasher.finalize(&mut output);

    output
}

fn main() -> Result<(), NotFoundError> {
    let opts = PowerClash::parse();
    let sighash_bytes = hex::decode(&opts.sighash).expect("Decoding failed");
    let char_set_bytes = opts.char_set.as_bytes();
    let max_permutations = opts.char_set.len().pow(opts.rnd_len);
    let capacity = opts.prefix.len() + opts.rnd_len as usize + opts.arg_signature.len() + 2;

    println!(
        "Attempting to find {}{}({}) match for 0x{} in {} max permutations",
        opts.prefix,
        "*".repeat(opts.rnd_len as usize),
        opts.arg_signature,
        opts.sighash,
        max_permutations
    );

    let start = Instant::now();
    let permutation_count = AtomicUsize::new(0);

    let benchmark_shown = AtomicBool::new(false);

    let result = (0..max_permutations).into_par_iter().find_any(|&j| {
        let test_signature_hash = hash_signature(&build_signature(
            j,
            capacity,
            opts.rnd_len,
            char_set_bytes,
            &opts.prefix,
            &opts.arg_signature,
        ));

        if !benchmark_shown.load(Ordering::Relaxed) {
            permutation_count.fetch_add(1, Ordering::Relaxed);

            if 0 == rayon::current_thread_index().unwrap() && start.elapsed().as_secs() > 2 {
                benchmark_shown.store(true, Ordering::Relaxed);
                println!(
                    "Calculating {} permutations per second",
                    permutation_count.load(Ordering::Relaxed) as f32
                        / start.elapsed().as_secs_f32(),
                );
            }
        }

        sighash_bytes.eq(&test_signature_hash)
    });

    match result {
        Some(permutation_index) => {
            let function_signature = build_signature(
                permutation_index,
                capacity,
                opts.rnd_len,
                char_set_bytes,
                &opts.prefix,
                &opts.arg_signature,
            );

            println!("Found match in {:?}", start.elapsed());
            println!("{} hashes to 0x{}", function_signature, opts.sighash);
            Ok(())
        }
        None => Err(NotFoundError),
    }
}
