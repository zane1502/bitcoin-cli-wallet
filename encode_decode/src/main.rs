use serde::Serialize;
use std::marker::PhantomData;
use tracing::*;

fn main() {
    trait Encode {
        fn encode<T: Serialize>(val: T) -> String;
    }

    struct Json;
    struct Toml;
    struct Cbor;
    struct Yaml;

    impl Encode for Json {
        fn encode<T: Serialize>(val: T) -> String {
            serde_json::to_string(&val).unwrap()
        }
    }
    impl Encode for Toml {
        fn encode<T: Serialize>(val: T) -> String {
            toml::to_string(&val).unwrap()
        }
    }
    // impl Encode for Cbor {
    //     fn encode<T: Serialize>(val: T) -> String {
    //         serde_cbor::to_vec(&val).unwrap()
    //     }
    // }
    impl Encode for Yaml {
        fn encode<T: Serialize>(val: T) -> String {
            serde_yaml::to_string(&val).unwrap()
        }
    }

    struct User<T: Encode> {
        name: String,
        age: u32,
        _marker: PhantomData<T>,
    }

    impl<T> User<T>
    where
        T: Encode,
    {
        fn new(name: String, age: u32) -> Self {
            User {
                name,
                age,
                _marker: PhantomData,
            }
        }
    }

    let user = User::new("Samuel".to_string(), 23);
    let encode = T::encode(&user);
    println!("{:?}", encode);

    let numbers = vec![1, 2, 3, 4, 5];
    // Start a new thread using a move closure
    let handle = thread::spawn(move || {
        let sum: i32 = numbers.iter().sum();
        println!("The sum is: {}", sum);
    });
    // Wait for the thread to complete
    handle.join().unwrap();
}
