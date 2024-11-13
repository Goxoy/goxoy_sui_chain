use base64::{prelude::BASE64_STANDARD, Engine};
use sui_sdk::{
    types::{
        base_types::SuiAddress,
        crypto::{get_key_pair_from_rng, SuiKeyPair},
    },
    SuiClientBuilder,
};

pub async fn get_wallet_all_balance(node_addr: String, wallet_address: SuiAddress) {
    match SuiClientBuilder::default().build(node_addr.clone()).await {
        Ok(sui) => {
            let coin_read = sui.coin_read_api();
            match coin_read.get_all_balances(wallet_address).await {
                Ok(total_balance) => {
                    println!("wallet balance: {:?}", total_balance);
                }
                Err(_) => {
                    println!("balance error");
                }
            };
        }
        Err(_) => {
            println!("client connection error");
        }
    }
}

pub async fn get_wallet_balance(node_addr: String, wallet_address: SuiAddress, coin_type: String) {
    match SuiClientBuilder::default().build(node_addr.clone()).await {
        Ok(sui) => {
            match sui
                .coin_read_api()
                .get_balance(wallet_address, Some(coin_type))
                .await
            {
                Ok(total_balance) => {
                    println!("wallet balance: {:?}", total_balance);
                }
                Err(_) => {
                    println!("balance error");
                }
            };
        }
        Err(_) => {
            println!("client connection error");
        }
    }
}

pub async fn get_wallet_owned_object(node_addr: String, wallet_address: SuiAddress) {
    match SuiClientBuilder::default().build(node_addr.clone()).await {
        Ok(sui) => {
            match sui
                .read_api()
                .get_owned_objects(wallet_address.clone(), None, None, Some(5))
                .await
            {
                Ok(owned_objects) => {
                    println!("// ************ READ API ************ //\n");
                    println!("owned_objects {:?}", owned_objects);
                }
                Err(_) => {
                    println!("object read error");
                }
            }
        }
        Err(_) => {
            println!("client connection error");
        }
    }
}

pub fn generate_keypair() -> SuiKeyPair {
    SuiKeyPair::Ed25519(get_key_pair_from_rng(&mut rand::rngs::OsRng).1)
}

pub fn private_key_string_to_keypair(keypair_bytes: Vec<u8>) -> SuiKeyPair {
    SuiKeyPair::from_bytes(&keypair_bytes).unwrap()
}

pub fn base64_private_key_to_wallet_address(base64_private_key: String) -> SuiAddress {
    let private_key_vec = BASE64_STANDARD.decode(base64_private_key).unwrap();
    let key_pair = private_key_string_to_keypair(private_key_vec);
    SuiAddress::from(&key_pair.public())
}
