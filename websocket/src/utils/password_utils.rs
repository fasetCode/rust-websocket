use argon2::{Argon2, PasswordHasher};
use password_hash::{SaltString, PasswordHash, PasswordVerifier};
use password_hash::rand_core::OsRng as CoreOsRng;

pub fn hash_password(password: &str) -> Result<String, password_hash::Error> {
    let mut rng = CoreOsRng;            // ✅ 安全 RNG
    let salt = SaltString::generate(&mut rng);

    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(hash)
}



pub fn verify_password(hash: &str, password: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash).unwrap();
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}


#[cfg(test)]
mod tests {
    use crate::utils::password_utils::{hash_password, verify_password};

    #[test]
    fn it_works() {

        // $argon2id$v=19$m=19456,t=2,p=1$e3lXKJWuoOJIcesYH85TCQ$8Xe89OS2eCoa63YEw2OtzWJdmv+83gVVtyDpFJ0LIPI
        // $argon2id$v=19$m=19456,t=2,p=1$EF00lWP6ViHb40X5JpWPJg$OAQp9aTatBWuRlTK2I7aQ4eFA6jyGtx7LkAHY4J4jxM
        // let string = hash_password("123456").expect("TODO: panic message");
        // println!("加密后的密码 --> {}", string);

        let res = verify_password(
            "$argon2id$v=19$m=19456,t=2,p=1$e3lXKJWuoOJIcesYH85TCQ$8Xe89OS2eCoa63YEw2OtzWJdmv+83gVVtyDpFJ0LIPI",
            "123456"
        );
        println!("验证结果 --> {}", res);
    }
}