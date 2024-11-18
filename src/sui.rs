use std::{borrow::BorrowMut, str::FromStr};

use anyhow::anyhow;
use base64::{prelude::BASE64_STANDARD, Engine};
use fastcrypto::hash::{Blake2b256, HashFunction};
use shared_crypto::intent::{Intent, IntentMessage};
use sui_json_rpc_types::{
    SuiObjectData, SuiObjectDataOptions, SuiPastObjectResponse, SuiTransactionBlockResponse
};
use sui_sdk::{
    rpc_types::SuiTransactionBlockResponseOptions, types::transaction::TransactionData, SuiClient,
    SuiClientBuilder,
};
use sui_types::base_types::SuiAddress;
use sui_types::crypto::SuiSignature;
use sui_types::crypto::{Signer, SuiKeyPair};
use sui_types::digests::TransactionDigest;
use sui_types::signature::GenericSignature;
use sui_types::sui_serde::BigInt;

use crate::model::balance::{
    AccountHistory, CoinDetail, ComplexTransactionDetails, ReceiveTokenDetails, SentTokenDetails,
    StakeDetail, SwapDetails,
};

use super::sui_lib::{sui_format_volume, sui_get_currency_name, SuiWalletFuncExtension};
pub struct SuiNetwork {
    node_addr: String,
    sui_client: Option<SuiClient>,
    key_pair: Option<SuiKeyPair>,
    public_addr_obj: Option<SuiAddress>,
    public_addr_str: String,
}

impl SuiNetwork {
    pub fn new(node_addr: Option<String>) -> Self {
        let node_addr = match node_addr {
            Some(node_addr) => node_addr,
            None => "https://fullnode.mainnet.sui.io:443".to_string(),
        };
        SuiNetwork {
            node_addr: node_addr,
            sui_client: None,
            key_pair: None,
            public_addr_obj: None,
            public_addr_str: "".to_string(),
        }
    }
    pub async fn connect(&mut self) -> bool {
        match SuiClientBuilder::default().build(&self.node_addr).await {
            Ok(sui_client) => {
                self.sui_client = Some(sui_client);
                true
            }
            Err(_) => false,
        }
    }

    pub async fn transfer(
        &mut self,
        receiver: String,
        volume: u64,
    ) -> Result<String, anyhow::Error> {
        let receiver = SuiAddress::from_str(&receiver.clone());
        if self.sui_client.is_none() {
            return Err(anyhow::Error::msg("not-connected-to-SUI-node"));
        }
        if self.key_pair.is_none() {
            return Err(anyhow::Error::msg("not-assigned-private-key"));
        }
        if receiver.is_err() {
            return Err(anyhow::Error::msg("wrong-receiver-address"));
        }
        let receiver = receiver.unwrap();
        println!("Sender: {:?}", self.public_addr_str.clone());
        println!("receiver: {:?}", receiver.clone());
        println!("---------------------------------------");

        let key_pair = self.key_pair.as_mut().unwrap().borrow_mut().copy();
        let sui_client = self.sui_client.as_mut().unwrap().borrow_mut();
        let gas_coin = sui_client
            .coin_read_api()
            .get_coins(self.public_addr_obj.unwrap(), None, None, None)
            .await
            .unwrap()
            .data
            .into_iter()
            .next()
            .ok_or(anyhow!("No coins found for sender"))
            .unwrap();
        let gas_budget = 5_000_000;
        let gas_price = sui_client
            .read_api()
            .get_reference_gas_price()
            .await
            .unwrap();

        let tx_data = TransactionData::new_transfer_sui(
            receiver,
            self.public_addr_obj.unwrap().clone(),
            Some(volume),
            gas_coin.object_ref(),
            gas_budget,
            gas_price,
        );
        let intent_msg = IntentMessage::new(Intent::sui_transaction(), tx_data.clone());
        let raw_tx = bcs::to_bytes(&intent_msg.clone()).expect("bcs should not fail");

        let mut hasher = Blake2b256::new();
        hasher.update(raw_tx.clone());

        let sui_sig = key_pair.sign(&hasher.finalize().to_vec());
        let res = sui_sig.verify_secure(
            &intent_msg.clone(),
            self.public_addr_obj.unwrap(),
            sui_types::crypto::SignatureScheme::ED25519,
        );
        if res.is_err() {
            return Err(anyhow::Error::msg("verify_secure is ERR".to_string()));
        }
        let transaction_response = sui_client
            .quorum_driver_api()
            .execute_transaction_block(
                sui_types::transaction::Transaction::from_generic_sig_data(
                    intent_msg.value.clone(),
                    vec![GenericSignature::Signature(sui_sig.clone())],
                ),
                SuiTransactionBlockResponseOptions::default(),
                None,
            )
            .await;
        match transaction_response {
            Ok(transaction_response) => {
                println!("gas_coin: {:?}", gas_coin.clone());
                println!("---------------------------------------");

                println!("gas_price: {:?}", gas_price.clone());
                println!("---------------------------------------");

                println!("tx_data: {:?}", tx_data.clone());
                println!("---------------------------------------");

                println!("intent_msg: {:?}", intent_msg.clone());
                println!("---------------------------------------");

                println!("transaction_response: {:?}", transaction_response.clone());
                println!("---------------------------------------");
                let tx_digest = transaction_response.digest.base58_encode();
                println!(
                    "Transaction executed. Transaction digest: {}",
                    tx_digest.clone()
                );
                println!("---------------------------------------");
                println!("{transaction_response}");
                Ok(tx_digest)
            }
            Err(tx_err) => Err(anyhow::Error::msg(tx_err.to_string())),
        }
    }

    pub fn get_my_address(&self) -> String {
        self.public_addr_str.clone()
    }
    pub fn set_my_private_key(&mut self, private_key: String) -> bool {
        self.public_addr_str = "".to_string();
        match BASE64_STANDARD.decode(private_key) {
            Ok(income_64) => match SuiKeyPair::from_bytes(&income_64) {
                Ok(key_pair) => {
                    self.key_pair = Some(key_pair.copy());
                    self.public_addr_obj = Some(SuiAddress::from(&key_pair.public()));
                    self.public_addr_str = self.public_addr_obj.unwrap().to_string();
                    println!("address (hex)   : {}", self.public_addr_str.clone());
                    true
                }
                Err(_) => false,
            },
            Err(_) => false,
        }
    }
    pub fn string_to_address_object(address_str: String) -> Result<SuiAddress, anyhow::Error> {
        match SuiAddress::from_str(&address_str) {
            Ok(addr) => Ok(addr),
            Err(e) => Err(e),
        }
    }

    pub async fn get_latest_checkpoint_no(&mut self) -> Result<u64, anyhow::Error> {
        if self.sui_client.is_none() {
            return Err(anyhow::Error::msg("not-connected-to-SUI-node"));
        }
        let sui_client = self.sui_client.as_mut().unwrap().borrow_mut();
        let result = sui_client.read_api().get_total_transaction_blocks().await;
        if result.is_ok() {
            return Ok(result.unwrap());
        }
        return Err(anyhow::Error::msg("could-not-read"));
    }
    pub async fn get_checkpoint(&mut self, seq_no: u64) -> Result<Vec<String>, anyhow::Error> {
        if self.sui_client.is_none() {
            return Err(anyhow::Error::msg("not-connected-to-SUI-node"));
        }
        let sui_client = self.sui_client.as_mut().unwrap().borrow_mut();
        let seq_no: BigInt<u64> = seq_no.into();
        let checkpoint_result = sui_client
            .read_api()
            .get_checkpoints(Some(seq_no), Some(1), false)
            .await;
        match checkpoint_result {
            Ok(checkpoint_data) => {
                // checkpoint_data.clone()
                dbg!(checkpoint_data.clone());
                // for item in checkpoint_data.data.iter(){
                //     // dbg!(item.)
                // }
                // dbg!(checkpoint_result);
                return Ok(Vec::new());
            }
            Err(_) => {
                return Err(anyhow::Error::msg("not-connected-to-SUI-node"));
            }
        }
    }
    pub async fn get_block_detail(&mut self, seq_no: u64) -> Result<Vec<String>, anyhow::Error> {
        if self.sui_client.is_none() {
            return Err(anyhow::Error::msg("not-connected-to-SUI-node"));
        }
        let sui_client = self.sui_client.as_mut().unwrap().borrow_mut();
        let control_no: BigInt<u64> = seq_no.into();
        match sui_client
            .read_api()
            .get_checkpoints(Some(control_no), Some(1), false)
            .await
        {
            Ok(checkpoint_data) => {
                let mut tx_list = Vec::new();
                for c_item in checkpoint_data.data.iter() {
                    for t_time in c_item.transactions.iter() {
                        tx_list.push(t_time.base58_encode());
                    }
                }
                return Ok(tx_list);
            }
            Err(_) => {
                return Err(anyhow::Error::msg("not-connected-to-SUI-node"));
            }
        }
    }
    pub async fn get_transaction_detail(
        &mut self,
        tx_digest: String,
    ) -> Result<SuiTransactionBlockResponse, anyhow::Error> {
        if self.sui_client.is_none() {
            return Err(anyhow::Error::msg("not-connected-to-SUI-node"));
        }
        let sui_client = self.sui_client.as_mut().unwrap().borrow_mut();

        match TransactionDigest::from_str(&tx_digest) {
            Ok(tx_digest) => {
                match sui_client
                    .read_api()
                    .get_transaction_with_options(
                        tx_digest,
                        SuiTransactionBlockResponseOptions {
                            show_input: true,
                            show_raw_input: true,
                            show_effects: true,
                            show_events: true,
                            show_object_changes: true,
                            show_balance_changes: true,
                            show_raw_effects: true,
                        },
                    )
                    .await
                {
                    Ok(tx_detail_result) => Ok(tx_detail_result),
                    Err(_) => {
                        return Err(anyhow::Error::msg("balance-reading-error"));
                    }
                }
            }
            Err(_) => {
                return Err(anyhow::Error::msg("tx-digest-convert-error"));
            }
        }
    }
    pub async fn get_wallet_balance(
        &mut self,
        wallet_address: SuiAddress,
    ) -> Result<String, anyhow::Error> {
        if self.sui_client.is_none() {
            return Err(anyhow::Error::msg("not-connected-to-SUI-node"));
        }
        let sui_client = self.sui_client.as_mut().unwrap().borrow_mut();

        // match sui_client.coin_read_api().get_balance(wallet_address,Some(coin_type)).await {
        match sui_client
            .coin_read_api()
            .get_balance(wallet_address, None)
            .await
        {
            Ok(total_balance) => {
                println!("wallet balance: {:?}", total_balance);
                Ok(format!("wallet balance: {:?}", total_balance))
            }
            Err(_) => {
                return Err(anyhow::Error::msg("balance-reading-error"));
            }
        }
    }

    pub async fn get_wallet_history_with_wallet_hex(
        &mut self,
        wallet_address: String,
    ) -> Option<Vec<AccountHistory>> {
        match SuiNetwork::string_to_address_object(wallet_address.clone()) {
            Ok(converted_wallet_addr) => {
                match self.get_wallet_history(converted_wallet_addr.clone()).await {
                    Ok(result_list) =>Some(result_list),
                    Err(_) =>None,
                }
            },
            Err(_) =>None,
        }
    }



    // bu fonksiyon henüz tamamlanmadı.!
    // bu fonksiyon henüz tamamlanmadı.!
    // bu fonksiyon henüz tamamlanmadı.!
    // bu fonksiyon henüz tamamlanmadı.!
    // bu fonksiyon henüz tamamlanmadı.!
    pub async fn get_wallet_object(
        &mut self,
        wallet_address: String,
    )->Option<Vec<SuiObjectData>>{
        if self.sui_client.is_none() {
            return None;
            // return Err(anyhow::Error::msg("not-connected-to-SUI-node"));
        }
        let wallet_addr_obj=SuiNetwork::string_to_address_object(wallet_address.clone());
        if wallet_addr_obj.is_err(){
            return None;
        }

        let mut result_list=Vec::new();
        let wallet_addr_obj=wallet_addr_obj.unwrap();
        let limit: usize = 20;
        let mut current_cursor = None;
        // let mut result_list = Vec::new();
        // let sui = SuiClientBuilder::default().build(node_addr).await.unwrap();
        // let key_pair=self.key_pair.as_mut().unwrap().borrow_mut().copy();
        let sui_client = self.sui_client.as_mut().unwrap().borrow_mut();
        let mut counter = 1;
        'inner_loop: loop {
            match sui_client
                .read_api()
                .get_owned_objects(wallet_addr_obj, None, current_cursor, Some(limit))
                .await
            {
                Ok(owned_objects) => {
                    if owned_objects.data.len() == 0 {
                        break 'inner_loop;
                    }
                    for object in owned_objects.data.iter() {
                        counter = counter + 1;
                        let object_data = object.data.as_ref().unwrap_or_else(|| {
                            panic!("No object data for this SuiObjectResponse {:?}", object)
                        });
                        result_list.push(object_data.clone());

                        let object_id = object_data.object_id;
                        current_cursor = Some(object_id.clone());
                        
                        
                        /*
                        let version = object_data.version;
                        let sui_data_options = SuiObjectDataOptions {
                            show_type: true,
                            show_owner: true,
                            show_previous_transaction: true,
                            show_display: true,
                            show_content: true,
                            show_bcs: true,
                            show_storage_rebate: true,
                        };
                        let past_object = sui_client
                            .read_api()
                            .try_get_parsed_past_object(
                                object_id,
                                version,
                                sui_data_options.clone(),
                            )
                            .await;
                        match past_object {
                            Ok(past_object) => {
                                match past_object {
                                    SuiPastObjectResponse::VersionFound(sui_object_data) => {
                                        match sui_object_data.previous_transaction {
                                            Some(prev_tx) => {
                                                match sui_client
                                                    .read_api()
                                                    .get_transaction_with_options(
                                                        prev_tx,
                                                        SuiTransactionBlockResponseOptions {
                                                            show_input: true,
                                                            show_raw_input: true,
                                                            show_effects: true,
                                                            show_events: true,
                                                            show_object_changes: true,
                                                            show_balance_changes: true,
                                                            show_raw_effects: true,
                                                        },
                                                    )
                                                    .await
                                                {
                                                    Ok(tx_result) => match tx_result.status_ok() {
                                                        Some(status) => {
                                                            if status == true {
                                                                let receive_list=SuiNetwork::organize_balance_list(tx_result.clone(),wallet_address.clone());
                                                                for b_item in receive_list.iter() {
                                                                    result_list
                                                                        .push(b_item.clone());
                                                                }
                                                            }
                                                        }
                                                        None => {
                                                            println!("status UNKNOWN");
                                                        }
                                                    },
                                                    Err(_) => {
                                                        println!("tx-error");
                                                    }
                                                }
                                            }
                                            None => {
                                                println!("tx-none");
                                            }
                                        };
                                    }
                                    SuiPastObjectResponse::ObjectNotExists(object_id) => {
                                        dbg!(object_id);
                                    }
                                    SuiPastObjectResponse::ObjectDeleted(sui_object_ref) => {
                                        dbg!(sui_object_ref);
                                    }
                                    SuiPastObjectResponse::VersionNotFound(
                                        object_id,
                                        sequence_number,
                                    ) => {
                                        dbg!(object_id);
                                        dbg!(sequence_number);
                                    }
                                    SuiPastObjectResponse::VersionTooHigh {
                                        object_id,
                                        asked_version,
                                        latest_version,
                                    } => {
                                        dbg!(object_id);
                                        dbg!(asked_version);
                                        dbg!(latest_version);
                                    }
                                };
                            }
                            Err(_) => {
                                println!("past object error");
                            }
                        }
                        
                        */
                    }
                }
                Err(_) => {
                    println!("owned objects error");
                    return Some(result_list);
                }
            }
        }
        Some(result_list)
    }

    pub async fn get_wallet_history(
        &mut self,
        wallet_address: SuiAddress,
    ) -> Result<Vec<AccountHistory>, anyhow::Error> {
        if self.sui_client.is_none() {
            return Err(anyhow::Error::msg("not-connected-to-SUI-node"));
        }

        let limit: usize = 20;
        let mut current_cursor = None;
        let mut result_list = Vec::new();
        // let sui = SuiClientBuilder::default().build(node_addr).await.unwrap();
        // let key_pair=self.key_pair.as_mut().unwrap().borrow_mut().copy();
        let sui_client = self.sui_client.as_mut().unwrap().borrow_mut();
        let mut counter = 1;
        'inner_loop: loop {
            match sui_client
                .read_api()
                .get_owned_objects(wallet_address, None, current_cursor, Some(limit))
                .await
            {
                Ok(owned_objects) => {
                    if owned_objects.data.len() == 0 {
                        println!("End Of data");
                        break 'inner_loop;
                    }
                    for object in owned_objects.data.iter() {
                        println!("counter: {}", counter);
                        counter = counter + 1;
                        let object_data = object.data.as_ref().unwrap_or_else(|| {
                            panic!("No object data for this SuiObjectResponse {:?}", object)
                        });
                        let object_id = object_data.object_id;
                        current_cursor = Some(object_id.clone());
                        let version = object_data.version;
                        let sui_data_options = SuiObjectDataOptions {
                            show_type: true,
                            show_owner: true,
                            show_previous_transaction: true,
                            show_display: true,
                            show_content: true,
                            show_bcs: true,
                            show_storage_rebate: true,
                        };
                        let past_object = sui_client
                            .read_api()
                            .try_get_parsed_past_object(
                                object_id,
                                version,
                                sui_data_options.clone(),
                            )
                            .await;
                        match past_object {
                            Ok(past_object) => {
                                match past_object {
                                    SuiPastObjectResponse::VersionFound(sui_object_data) => {
                                        match sui_object_data.previous_transaction {
                                            Some(prev_tx) => {
                                                match sui_client
                                                    .read_api()
                                                    .get_transaction_with_options(
                                                        prev_tx,
                                                        SuiTransactionBlockResponseOptions {
                                                            show_input: true,
                                                            show_raw_input: true,
                                                            show_effects: true,
                                                            show_events: true,
                                                            show_object_changes: true,
                                                            show_balance_changes: true,
                                                            show_raw_effects: true,
                                                        },
                                                    )
                                                    .await
                                                {
                                                    Ok(tx_result) => match tx_result.status_ok() {
                                                        Some(status) => {
                                                            if status == true {
                                                                let receive_list=SuiNetwork::organize_balance_list(tx_result.clone(),wallet_address.clone());
                                                                for b_item in receive_list.iter() {
                                                                    result_list
                                                                        .push(b_item.clone());
                                                                }
                                                            }
                                                        }
                                                        None => {
                                                            println!("status UNKNOWN");
                                                        }
                                                    },
                                                    Err(_) => {
                                                        println!("tx-error");
                                                    }
                                                }
                                            }
                                            None => {
                                                println!("tx-none");
                                            }
                                        };
                                    }
                                    SuiPastObjectResponse::ObjectNotExists(object_id) => {
                                        dbg!(object_id);
                                    }
                                    SuiPastObjectResponse::ObjectDeleted(sui_object_ref) => {
                                        dbg!(sui_object_ref);
                                    }
                                    SuiPastObjectResponse::VersionNotFound(
                                        object_id,
                                        sequence_number,
                                    ) => {
                                        dbg!(object_id);
                                        dbg!(sequence_number);
                                    }
                                    SuiPastObjectResponse::VersionTooHigh {
                                        object_id,
                                        asked_version,
                                        latest_version,
                                    } => {
                                        dbg!(object_id);
                                        dbg!(asked_version);
                                        dbg!(latest_version);
                                    }
                                };
                            }
                            Err(_) => {
                                println!("past object error");
                            }
                        }
                    }
                }
                Err(_) => {
                    println!("owned objects error");
                    return Ok(result_list);
                }
            }
        }
        Ok(result_list)
    }

    pub fn organize_balance_list(
        tx_details: SuiTransactionBlockResponse,
        my_wallet_address: SuiAddress,
    ) -> Vec<AccountHistory> {
        let block_time = tx_details.timestamp_ms.unwrap_or(0) as u128;
        let block_no = tx_details.checkpoint.unwrap_or(0) as u128;
        let tx_digest = tx_details.digest.clone().to_string();
        let mut result_list = Vec::new();
        let my_sui_addr = my_wallet_address.to_string();
        let chain_currency = "sui::SUI";

        match tx_details.clone().balance_changes {
            Some(balance_list) => {
                if balance_list.my_balanced_changed(my_sui_addr.clone()) == false {
                    return result_list;
                }
                let go_out_currency =
                    balance_list.which_currency_go_out_my_account(my_sui_addr.clone());
                let address_list = balance_list.get_address_list();
                let currency_list = balance_list.get_currency_list();
                let get_in_currency =
                    balance_list.which_currency_get_in_my_account(my_sui_addr.clone());

                //stake işlemi
                if address_list.len() == 1
                    && get_in_currency.len() == 0
                    && go_out_currency.len() >= 1
                {
                    let mut currency = chain_currency.to_string().clone();
                    if currency_list.len() > 1 {
                        for c_item in currency_list.iter() {
                            if c_item.eq(chain_currency) == false {
                                currency = c_item.clone();
                            }
                        }
                    }
                    for i_item in balance_list.iter() {
                        let i_currency = sui_get_currency_name(
                            i_item.coin_type.to_canonical_display(true).to_string(),
                        );
                        if i_currency.eq(&currency.clone()) == false {
                            let volume = i_item.amount.abs() as u128;
                            result_list.push(AccountHistory::Stake(StakeDetail {
                                digest: tx_digest.clone(),
                                time: block_time,
                                block_no: block_no,
                                currency: i_currency.clone(),
                                volume: volume,
                                formated_volume: sui_format_volume(volume, i_currency.clone()),
                            }));
                            return result_list;
                        }
                    }
                }

                //komisyonlu swap işlemi
                if address_list.len() == 2
                    && get_in_currency.len() == 1
                    && go_out_currency.len() == 2
                {
                    for i_item in balance_list.iter() {
                        let commision_account =
                            format!("{}", i_item.owner.get_owner_address().unwrap().to_string());
                        if commision_account.eq(&my_sui_addr.clone()) == false {
                            let currency_name = sui_get_currency_name(
                                i_item.coin_type.to_canonical_display(true).to_string(),
                            );
                            if currency_name.eq(chain_currency) == true && i_item.amount > 0 {
                                // println!("bu hesap swap için komisyon almis")
                                let commision_volume = i_item.amount as u128;
                                let mut gas_total = 0;
                                for i_item in balance_list.iter() {
                                    let account_text = format!(
                                        "{}",
                                        i_item.owner.get_owner_address().unwrap().to_string()
                                    );
                                    if account_text.eq(chain_currency) == true {
                                        if i_item.amount < 0 {
                                            gas_total = i_item.amount.abs() as u128;
                                        } else {
                                            println!("kontrol-noktasi-011");
                                        }
                                    }
                                }
                                let mut input_currency = String::new();
                                let mut output_currency = String::new();
                                let mut input_volume = 0;
                                let mut output_volume = 0;

                                for i_item in balance_list.iter() {
                                    let i_currency = sui_get_currency_name(
                                        i_item.coin_type.to_canonical_display(true).to_string(),
                                    );
                                    if i_currency.eq(&chain_currency) == false {
                                        if i_item.amount > 0 {
                                            input_currency = i_currency.clone();
                                            input_volume = i_item.amount as u128;
                                        } else {
                                            output_currency = i_currency.clone();
                                            output_volume = i_item.amount.abs() as u128;
                                        }
                                    }
                                }
                                result_list.push(AccountHistory::Swap(SwapDetails {
                                    digest: tx_digest.clone(),
                                    time: block_time,
                                    block_no: block_no,
                                    input_currency: input_currency.clone(),
                                    output_currency: output_currency.clone(),
                                    input_volume: input_volume,
                                    output_volume: output_volume,
                                    input_formated_volume: sui_format_volume(
                                        input_volume,
                                        input_currency.clone(),
                                    ),
                                    output_formated_volume: sui_format_volume(
                                        output_volume,
                                        output_currency.clone(),
                                    ),
                                    gas: gas_total,
                                    formated_gas: sui_format_volume(
                                        gas_total,
                                        chain_currency.to_string(),
                                    ),
                                    commision_status: true,
                                    commision_account: commision_account,
                                    commision_volume: commision_volume,
                                    formatted_commision: sui_format_volume(
                                        commision_volume,
                                        chain_currency.to_string(),
                                    ),
                                }));
                                return result_list;
                            }
                        }
                    }
                }

                //token swap işlemi
                if address_list.len() == 1
                    && get_in_currency.len() == 1
                    && go_out_currency.len() == 2
                {
                    let mut gas_total = 0;
                    for i_item in balance_list.iter() {
                        let account_text =
                            format!("{}", i_item.owner.get_owner_address().unwrap().to_string());
                        if account_text.eq(chain_currency) == true {
                            if i_item.amount < 0 {
                                gas_total = i_item.amount.abs() as u128;
                            } else {
                                println!("kontrol-noktasi-011");
                            }
                        }
                    }
                    let mut input_currency = String::new();
                    let mut output_currency = String::new();
                    let mut input_volume = 0;
                    let mut output_volume = 0;

                    for i_item in balance_list.iter() {
                        let i_currency = sui_get_currency_name(
                            i_item.coin_type.to_canonical_display(true).to_string(),
                        );
                        if i_currency.eq(&chain_currency) == false {
                            if i_item.amount > 0 {
                                input_currency = i_currency.clone();
                                input_volume = i_item.amount as u128;
                            } else {
                                output_currency = i_currency.clone();
                                output_volume = i_item.amount.abs() as u128;
                            }
                        }
                    }
                    result_list.push(AccountHistory::Swap(SwapDetails {
                        digest: tx_digest.clone(),
                        time: block_time,
                        block_no: block_no,
                        input_currency: input_currency.clone(),
                        output_currency: output_currency.clone(),
                        input_volume: input_volume,
                        output_volume: output_volume,
                        input_formated_volume: sui_format_volume(
                            input_volume,
                            input_currency.clone(),
                        ),
                        output_formated_volume: sui_format_volume(
                            output_volume,
                            output_currency.clone(),
                        ),
                        gas: gas_total,
                        formated_gas: sui_format_volume(gas_total, chain_currency.to_string()),
                        commision_status: false,
                        commision_account: String::new(),
                        commision_volume: 0,
                        formatted_commision: String::new(),
                    }));
                    // println!("swap : {} {} >> {} {}",
                    //     sui_format_volume(output_volume, output_currency.clone()),output_currency.clone(),
                    //     sui_format_volume(input_volume, input_currency.clone()),input_currency.clone(),
                    // );
                    return result_list;
                }

                //coin swap işlemi
                if address_list.len() == 1
                    && get_in_currency.len() == 1
                    && go_out_currency.len() == 1
                {
                    let input_currency = get_in_currency[0].clone();
                    let output_currency = go_out_currency[0].clone();
                    let mut input_volume = 0;
                    let mut output_volume = 0;

                    for i_item in balance_list.iter() {
                        if i_item.amount > 0 {
                            input_volume = i_item.amount as u128;
                        } else {
                            output_volume = i_item.amount.abs() as u128;
                        }
                    }
                    result_list.push(AccountHistory::Swap(SwapDetails {
                        digest: tx_digest.clone(),
                        time: block_time,
                        block_no: block_no,
                        input_currency: input_currency.clone(),
                        output_currency: output_currency.clone(),
                        input_volume: input_volume,
                        output_volume: output_volume,
                        input_formated_volume: sui_format_volume(
                            input_volume,
                            input_currency.clone(),
                        ),
                        output_formated_volume: sui_format_volume(
                            output_volume,
                            output_currency.clone(),
                        ),
                        gas: 0,
                        formated_gas: String::new(),
                        commision_status: false,
                        commision_account: String::new(),
                        commision_volume: 0,
                        formatted_commision: String::new(),
                    }));
                    // println!("swap : {} {} >> {} {}",
                    //     sui_format_volume(output_volume, output_currency.clone()),output_currency.clone(),
                    //     sui_format_volume(input_volume, input_currency.clone()),input_currency.clone(),
                    // );
                    return result_list;
                }

                //receive coin veya token
                if address_list.len() == 2
                    && get_in_currency.len() == 1
                    && go_out_currency.len() == 0
                {
                    let income_currency = get_in_currency[0].clone();
                    if income_currency.eq(chain_currency) == true {
                        for item in balance_list.iter() {
                            if item.amount > 0 {
                                let receiver = format!(
                                    "{}",
                                    item.owner.get_owner_address().unwrap().to_string()
                                );
                                let currency = sui_get_currency_name(
                                    item.coin_type.to_canonical_display(true).to_string(),
                                );
                                let volume = item.amount.abs() as u128;
                                let mut sender = String::new();
                                for item in balance_list.iter() {
                                    if item.amount < 0 {
                                        sender = format!(
                                            "{}",
                                            item.owner.get_owner_address().unwrap().to_string()
                                        );
                                    }
                                }
                                if receiver.eq(&my_sui_addr.clone()) == true {
                                    result_list.push(AccountHistory::ReceiveCoin(CoinDetail {
                                        digest: tx_digest.clone(),
                                        time: block_time,
                                        block_no: block_no,
                                        sender: sender,
                                        receiver: receiver.clone(),
                                        currency: currency.clone(),
                                        volume: volume,
                                        formated_volume: sui_format_volume(
                                            volume,
                                            currency.clone(),
                                        ),
                                    }));
                                    return result_list;
                                }
                            }
                        }
                    } else {
                        for outer_item in balance_list.iter() {
                            let outer_owner_addr = format!(
                                "{}",
                                outer_item.owner.get_owner_address().unwrap().to_string()
                            );
                            if outer_owner_addr.eq(&my_sui_addr.clone()) {
                                if outer_item.amount > 0 {
                                    let receiver = outer_owner_addr.clone();
                                    let volume = outer_item.amount.abs() as u128;
                                    let currency = sui_get_currency_name(
                                        outer_item.coin_type.to_canonical_display(true).to_string(),
                                    );
                                    for inner_item in balance_list.iter() {
                                        let inner_owner_addr = format!(
                                            "{}",
                                            inner_item
                                                .owner
                                                .get_owner_address()
                                                .unwrap()
                                                .to_string()
                                        );
                                        if inner_owner_addr.eq(&my_sui_addr.clone()) == false {
                                            let sender = inner_owner_addr.clone();
                                            result_list.push(AccountHistory::ReceiveToken(
                                                ReceiveTokenDetails {
                                                    digest: tx_digest.clone(),
                                                    time: block_time,
                                                    block_no: block_no,
                                                    sender: sender.clone(),
                                                    receiver: receiver.clone(),
                                                    currency: currency.clone(),
                                                    volume: volume,
                                                    formated_volume: sui_format_volume(
                                                        volume,
                                                        currency.clone(),
                                                    ),
                                                },
                                            ));
                                            return result_list;
                                        }
                                    }
                                } else {
                                    println!("tx_digest: {}", tx_digest.clone());
                                    dbg!(balance_list.clone());
                                    std::process::exit(99);
                                }
                            }
                        }
                    }
                }

                //transfer coin
                if address_list.len() == 2
                    && get_in_currency.len() == 0
                    && go_out_currency.len() == 1
                {
                    let mut receiver = String::new();
                    let mut real_volume = 0;
                    for item in balance_list.iter() {
                        let volume = item.amount.abs() as u128;
                        if volume > real_volume {
                            real_volume = volume;
                        }
                        if item.amount > 0 {
                            receiver =
                                format!("{}", item.owner.get_owner_address().unwrap().to_string());
                        }
                    }
                    result_list.push(AccountHistory::SentCoin(CoinDetail {
                        digest: tx_digest.clone(),
                        time: block_time,
                        block_no: block_no,
                        sender: my_sui_addr.clone(),
                        receiver: receiver.clone(),
                        currency: chain_currency.to_string(),
                        volume: real_volume,
                        formated_volume: sui_format_volume(real_volume, chain_currency.to_string()),
                    }));
                    return result_list;
                }

                //transfer token
                if address_list.len() == 2
                    && get_in_currency.len() == 0
                    && go_out_currency.len() == 2
                {
                    let mut receiver = String::new();
                    let mut volume = 0;
                    let mut gas_total = 0;
                    let mut token_currency = String::new();
                    for item in balance_list.iter() {
                        let currency = sui_get_currency_name(
                            item.coin_type.to_canonical_display(true).to_string(),
                        );
                        if currency.eq(chain_currency) {
                            gas_total = item.amount.abs() as u128;
                        } else {
                            if item.amount < 0 {
                                volume = item.amount.abs() as u128;
                            } else {
                                token_currency = sui_get_currency_name(
                                    item.coin_type.to_canonical_display(true).to_string(),
                                );
                                receiver = format!(
                                    "{}",
                                    item.owner.get_owner_address().unwrap().to_string()
                                );
                            }
                        }
                    }

                    result_list.push(AccountHistory::SentToken(SentTokenDetails {
                        digest: tx_digest.clone(),
                        time: block_time,
                        block_no: block_no,
                        sender: my_sui_addr.clone(),
                        receiver: receiver.clone(),
                        currency: chain_currency.to_string(),
                        volume: volume,
                        formated_volume: sui_format_volume(volume, token_currency.to_string()),
                        gas: gas_total,
                        formated_gas: sui_format_volume(gas_total, chain_currency.to_string()),
                    }));
                    return result_list;
                }

                result_list.push(AccountHistory::ComplexTransaction(
                    ComplexTransactionDetails {
                        digest: tx_digest.clone(),
                        time: block_time,
                        block_no: block_no,
                        get_in_currency: get_in_currency.clone(),
                        go_out_currency: go_out_currency.clone(),
                        address_list: address_list.clone(),
                        currency_list: currency_list.clone(),
                        balance_list: balance_list.clone(),
                    },
                ));
                return result_list;
            }
            None => {}
        }
        return result_list;
    }
}
