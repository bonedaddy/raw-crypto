use super::consts::*;

extern "C" {
  fn check_scalar(scalar: *const u8) -> bool;
  fn random_scalar(secret_key: *mut u8);
  fn hash_to_scalar(data: *const u8, length: usize, hash: *mut u8);
}

pub struct EllipticCurveScalar {
  pub data: [u8; CHACHA_IV_SIZE],
}

impl EllipticCurveScalar {
  pub fn check(scalar: &[u8; 32]) -> bool {
    unsafe { return check_scalar(scalar[..].as_ptr()) }
  }

  pub fn random(secret_key: &mut [u8; 32]) {
    unsafe {
      random_scalar(secret_key.as_mut_ptr());
    }
  }
  pub fn to_hash(plain: &[u8]) -> [u8; 32] {
    let mut hash: [u8; 32] = [0; 32];
    unsafe { hash_to_scalar(plain.as_ptr(), plain.len(), hash.as_mut_ptr()) }
    hash
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs::{canonicalize, File};
  use std::io::{prelude::*, BufReader};
  use std::path::PathBuf;
  extern crate hex;
  use super::super::key::Key;

  extern "C" {
    fn setup_random(value: i32);
  }

  #[test]
  fn should_to_hash() {
    let bytes = hex::decode("2ace").expect("Error parse scalar");
    let hash = EllipticCurveScalar::to_hash(bytes.as_slice());
    let expected = hex::decode("427f5090283713a2a8448285f2a22cc8cf5374845766b6370425e2319e40f50d")
      .expect("Error parse scalar");
    // println!("{:0x?}", hash);
    // println!("expected: 427f5090283713a2a8448285f2a22cc8cf5374845766b6370425e2319e40f50d");
    assert!(hash == expected.as_slice());
  }

  #[test]
  fn should_test_scalar() {
    let path = PathBuf::from("./tests/tests.txt");
    let str = canonicalize(path);
    // println!("{:?}", &str);
    let f = File::open(str.unwrap()).unwrap();
    let file = BufReader::new(&f);
    let mut last = String::from("");
    let mut executed = false;
    for (_num, line) in file.lines().enumerate() {
      let l = line.unwrap();
      let split: Vec<&str> = l.split_whitespace().collect();
      let name = split[0];
      if last != name {
        // println!("{:?}", split[0]);
        last = split[0].to_string();
      }
      match name {
        "check_scalar" => {
          let plain = hex::decode(split[1]).expect("Error parse scalar");
          let expected = split[2] == String::from("true");
          let mut scalar: [u8; 32] = [0; 32];
          for i in 0..32 {
            scalar[i] = plain[i];
          }
          let actual = EllipticCurveScalar::check(&scalar);
          assert!(expected == actual)
        }
        "random_scalar" => {
          if !executed {
            unsafe {
              setup_random(42);
            }
            executed = true;
          }
          let expected = hex::decode(split[1]).expect("Error parse expected");
          let mut ec_scalar: [u8; 32] = [0; 32];
          EllipticCurveScalar::random(&mut ec_scalar);
          for i in 0..32 {
            assert!(expected[i] == ec_scalar[i]);
          }
        }
        "hash_to_scalar" => {
          let mut bytes: Vec<u8>;
          if split[1] == "x" {
            bytes = hex::decode("").expect("Error parse scalar");
          } else {
            bytes = hex::decode(split[1]).expect("Error parse scalar");
          }
          let hash = EllipticCurveScalar::to_hash(bytes.as_slice());
          let expected = hex::decode(split[2]).expect("Error parse expected");
          assert!(hash == expected.as_slice());
        }
        "generate_keys" => {
          // println!("{}", split[1]);
          // println!("{}", split[2]);
          let public_key = hex::decode(split[1]).expect("Error parse expected");
          let private_key = hex::decode(split[2]).expect("Error parse expected");
          let mut generated_public_key: [u8; 32] = [0; 32];
          let mut generated_private_key: [u8; 32] = [0; 32];
          Key::generate_key_pair(&mut generated_public_key, &mut generated_private_key);
          // println!("generated public key: {:0x?}", generated_public_key);
          // println!("generated private key: {:0x?}", generated_private_key);
          assert!(public_key.as_slice() == generated_public_key);
          assert!(private_key.as_slice() == generated_private_key);
        }
        "check_key" => {
          let public_key = hex::decode(split[1]).expect("Error parse expected");
          assert!(public_key.len() == 32);
          let mut fixed_public_key: [u8; 32] = [0; 32];
          for i in 0..32 {
            fixed_public_key[i] = public_key[i];
          }
          let expected = split[2] == "true";
          assert!(Key::check_public_key(&fixed_public_key) == expected);
        }
        "secret_key_to_public_key" => {
          let secret_key = hex::decode(split[1]).expect("Error parse expected");
          let mut fixed_secret_key: [u8; 32] = [0; 32];
          for i in 0..32 {
            fixed_secret_key[i] = secret_key[i];
          }
          let expected1 = split[2] == "true";
          let mut public_key: [u8; 32] = [0; 32];
          let actual1 = Key::secret_to_public(&fixed_secret_key, &mut public_key);
          assert!(expected1 == actual1);
          if expected1 == true {
            let expected2 = hex::decode(split[3]).expect("Error parse expected");
            assert!(public_key == expected2.as_slice());
          }
        }
        "generate_key_derivation" => {
          let public_key = hex::decode(split[1]).expect("Error parse expected");
          let mut fixed_public_key: [u8; 32] = [0; 32];
          for i in 0..32 {
            fixed_public_key[i] = public_key[i];
          }
          let secret_key = hex::decode(split[2]).expect("Error parse expected");
          let mut fixed_secret_key: [u8; 32] = [0; 32];
          for i in 0..32 {
            fixed_secret_key[i] = secret_key[i];
          }
          let expected1 = split[3] == "true";
          let derived = Key::generate_key_derivation(&fixed_public_key, &fixed_secret_key);
          if expected1 {
            let expected2 = hex::decode(split[4]).expect("Error parse expected");
            assert!(derived == expected2.as_slice());
          } else {
            assert!(derived == [0; 32]);
          }
        }
        "derive_public_key" => {
          let derivation = hex::decode(split[1]).expect("Error parse derivation");
          let out_index = split[2].parse::<u32>().unwrap();
          let public_key = hex::decode(split[3]).expect("Error parse public key");
          let expected1 = split[4] == "true";
          let mut fixed_derivation: [u8; 32] = [0; 32];
          for i in 0..32 {
            fixed_derivation[i] = derivation[i];
          }

          let mut fixed_base: [u8; 32] = [0; 32];
          for i in 0..32 {
            fixed_base[i] = public_key[i];
          }
          let derived = Key::derive_public_key(&fixed_derivation, out_index as u64, &fixed_base);

          if expected1 {
            let expected2 = hex::decode(split[5]).expect("Error parse expected derived");
            assert!(expected2.as_slice() == derived);
          } else {
            assert!(derived == [0; 32]);
          }
        }
        "derive_secret_key" => {
          let derivation = hex::decode(split[1]).expect("Error parse derivation");
          let out_index = split[2].parse::<u32>().unwrap();

          let private_key = hex::decode(split[3]).expect("Error parse public key");
          let expected = hex::decode(split[4]).expect("Error parse public key");
          let mut fixed_derivation: [u8; 32] = [0; 32];
          for i in 0..32 {
            fixed_derivation[i] = derivation[i];
          }

          let mut fixed_base: [u8; 32] = [0; 32];
          for i in 0..32 {
            fixed_base[i] = private_key[i];
          }
          let derived = Key::derive_secret_key(&fixed_derivation, out_index as u64, &fixed_base);
          assert!(derived == expected.as_slice());
        }
        "underive_public_key" => {
          let derivation = hex::decode(split[1]).expect("Error parse derivation");
          let out_index = split[2].parse::<u32>().unwrap();
          let public_key = hex::decode(split[3]).expect("Error parse public key");
          let expected1 = split[4] == "true";
          let mut fixed_derivation: [u8; 32] = [0; 32];
          for i in 0..32 {
            fixed_derivation[i] = derivation[i];
          }

          let mut fixed_base: [u8; 32] = [0; 32];
          for i in 0..32 {
            fixed_base[i] = public_key[i];
          }
          let derived = Key::underive_public_key(&fixed_derivation, out_index as u64, &fixed_base);

          if expected1 {
            let expected2 = hex::decode(split[5]).expect("Error parse expected derived");
            assert!(expected2.as_slice() == derived);
          } else {
            assert!(derived == [0; 32]);
          }
        }
        "check_ring_signature" => {
          let pre_hash = hex::decode(split[1]).expect("Error parse pre hash!");
          // println!("pre hash = {}", split[1]);
          let key_image = hex::decode(split[2]).expect("Error parse key image!");
          // println!("key image = {}", split[2]);

          let pubs_count = split[3].parse::<u64>().expect("Error parse integer!");
          // println!("pubs count = {}", split[3]);

          let mut pubs: Vec<u8> = vec![];
          for n in 0..pubs_count {
            // println!("{}", n);
            // println!("{}", split[4 + n as usize]);
            let key = hex::decode(split[4 + n as usize]).expect("Error parse public key!");
            // println!("{:x?}", key);

            let mut converted_key: [u8; 32] = [0; 32];
            for i in 0..32 {
              // pubs.push(key[i]);
              converted_key[i] = key[i];
            }
            // println!("n = {}", n);
            // println!("n = {:x?}", converted_key);
            pubs.extend(&converted_key);
          }

          // println!("pubs.len() = {}", pubs.len());
          // println!("{}", 32 * pubs_count);

          // assert!(pubs.len() == (32 * pubs_count) as usize);

          // let sig = hex::decode(split[4 + pubs_count as usize]).expect("Error parse siginatures!");
          // println!("{}", sig.len());
          // println!("{}", pubs_count);
          // let mut siginatures : Vec<[u8; 64]> = vec![];
          // for n in 0..pubs_count {
          //   let mut n_sig : [u8;64] = [0; 64];
          //   for i in 0..64 {
          //     let idx = n * 64 + (i as u64);
          //     n_sig[i] = sig[idx as usize];
          //   }
          //   siginatures
          // }

          // println!("{:?}", split);
          // let expected = split[5 + pubs_count as usize] == "true";
          // println!("expected = {}", expected);
          // let actual = is_ring_signature(pre_hash.as_slice(), key_image.as_slice(), pubs.as_slice(), pubs_count as usize, sig.as_slice());
          // assert!(expected == actual);
        }
        _ => {}
      }
    }
  }
}
