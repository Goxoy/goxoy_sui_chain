use std::usize;

use sui_json_rpc_types::{BalanceChange, SuiTransactionBlockData, SuiTransactionBlockResponse};
use sui_types::{
    base_types::SuiAddress,
    crypto::{get_key_pair_from_rng, SuiKeyPair},
};

pub trait SuiTransactionFuncExtension {
    fn who_paid_gas(&self) -> String;
    fn get_module_list(&self) -> Vec<String>;
    fn get_function_list(&self) -> Vec<String>;
}
impl SuiTransactionFuncExtension for SuiTransactionBlockResponse {
    fn who_paid_gas(&self) -> String {
        match self.transaction.clone() {
            Some(tx_obj) => match tx_obj.data {
                SuiTransactionBlockData::V1(sui_transaction_block_data_v1) => format!(
                    "{}",
                    sui_transaction_block_data_v1.gas_data.owner.to_string()
                ),
            },
            None => "".to_string(),
        }
    }
    fn get_module_list(&self) -> Vec<String> {
        let mut module_list: Vec<String> = Vec::new();
        match self.transaction.clone() {
            Some(inner_tx_obj) => {
                for inner_item in inner_tx_obj.data.move_calls().iter() {
                    if inner_item.module.len() == 0 {
                        dbg!(inner_item);
                    } else {
                        let module_name_str = inner_item.module.clone();
                        // println!("module: {} / {}",inner_item.module,inner_item.function);
                        let mut module_found = false;
                        for a_item in module_list.iter() {
                            if a_item.clone().eq(&module_name_str.clone()) {
                                module_found = true;
                            }
                        }
                        if module_found == false {
                            module_list.push(module_name_str);
                        }
                    }
                }
            }
            None => {}
        }
        module_list
    }
    fn get_function_list(&self) -> Vec<String> {
        let mut result_list: Vec<String> = Vec::new();
        match self.transaction.clone() {
            Some(inner_tx_obj) => {
                for inner_item in inner_tx_obj.data.move_calls().iter() {
                    if inner_item.module.len() == 0 {
                        dbg!(inner_item);
                    } else {
                        let module_name_str = inner_item.function.clone();
                        let mut item_found = false;
                        for a_item in result_list.iter() {
                            if a_item.clone().eq(&module_name_str.clone()) {
                                item_found = true;
                            }
                        }
                        if item_found == false {
                            result_list.push(module_name_str);
                        }
                    }
                }
            }
            None => {}
        }
        result_list
    }
}

pub trait SuiWalletFuncExtension {
    fn which_currency_go_out_my_account(&self, my_wallet_addr: String) -> Vec<String>;
    fn which_currency_get_in_my_account(&self, my_wallet_addr: String) -> Vec<String>;
    fn get_currency_list(&self) -> Vec<String>;
    fn get_address_list(&self) -> Vec<String>;
    fn my_balanced_changed(&self, my_wallet_addr: String) -> bool;
    fn remove_dont_belong_to_me(&self, my_wallet_addr: String) -> Vec<BalanceChange>;
}

impl SuiWalletFuncExtension for Vec<BalanceChange> {
    fn which_currency_go_out_my_account(&self, my_wallet_addr: String) -> Vec<String> {
        let mut result_list = Vec::new();
        for item in self.iter() {
            let control_addr = format!("{}", item.owner.get_owner_address().unwrap().to_string());
            if control_addr.eq(&my_wallet_addr.clone()) {
                if item.amount < 0 {
                    result_list.push(sui_get_currency_name(
                        item.coin_type.to_canonical_display(true).to_string(),
                    ));
                }
            }
        }
        result_list
    }
    fn which_currency_get_in_my_account(&self, my_wallet_addr: String) -> Vec<String> {
        let mut result_list = Vec::new();
        for item in self.iter() {
            let control_addr = format!("{}", item.owner.get_owner_address().unwrap().to_string());
            if control_addr.eq(&my_wallet_addr.clone()) {
                if item.amount > 0 {
                    result_list.push(sui_get_currency_name(
                        item.coin_type.to_canonical_display(true).to_string(),
                    ));
                }
            }
        }
        result_list
    }
    fn get_currency_list(&self) -> Vec<String> {
        let mut currency_list: Vec<String> = Vec::new();
        for item in self.iter() {
            let c_text =
                sui_get_currency_name(item.coin_type.to_canonical_display(true).to_string());
            let mut addr_found = false;
            for a_item in currency_list.iter() {
                if a_item.clone().eq(&c_text.clone()) {
                    addr_found = true;
                }
            }
            if addr_found == false {
                currency_list.push(c_text);
            }
        }
        currency_list
    }
    fn get_address_list(&self) -> Vec<String> {
        let mut address_list: Vec<String> = Vec::new();
        for item in self.iter() {
            let c_addr = format!("{}", item.owner.get_owner_address().unwrap().to_string());
            let mut addr_found = false;
            for a_item in address_list.iter() {
                if a_item.clone().eq(&c_addr.clone()) {
                    addr_found = true;
                }
            }
            if addr_found == false {
                address_list.push(c_addr);
            }
        }
        address_list
    }
    fn my_balanced_changed(&self, my_wallet_addr: String) -> bool {
        for item in self.iter() {
            let control_addr = format!("{}", item.owner.get_owner_address().unwrap().to_string());
            if control_addr.eq(&my_wallet_addr.clone()) {
                return true;
            }
        }
        false
    }
    fn remove_dont_belong_to_me(&self, my_wallet_addr: String) -> Vec<BalanceChange> {
        let mut balance_list = self.clone();
        loop {
            let mut found_index = usize::MAX;
            for (index, item) in balance_list.iter().enumerate() {
                let control_addr =
                    format!("{}", item.owner.get_owner_address().unwrap().to_string());
                if control_addr.eq(&my_wallet_addr.clone()) == false {
                    found_index = index;
                }
            }
            if found_index == usize::MAX {
                break;
            } else {
                balance_list.remove(found_index);
            }
        }
        balance_list
    }
}

pub fn sui_get_currency_name(raw_currency: String) -> String {
    if raw_currency
        .eq("0x0000000000000000000000000000000000000000000000000000000000000002::sui::SUI")
        || raw_currency.eq("0x2::sui::SUI")
    {
        return "sui::SUI".to_string();
    } else if raw_currency
        .eq("0xfa7ac3951fdca92c5200d468d31a365eb03b2be9936fde615e69f0c1274ad3a0::blub::BLUB")
    {
        return "blub::BLUB".to_string();
    } else if raw_currency
        .eq("0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC")
    {
        return "usdc::USDC".to_string();
    } else if raw_currency
        .eq("0xdeeb7a4662eec9f2f3def03fb937a663dddaa2e215b8078a284d026b7946c270::deep::DEEP")
    {
        return "deep::DEEP".to_string();
    } else if raw_currency
        .eq("0x1fc50c2a9edf1497011c793cb5c88fd5f257fd7009e85a489392f388b1118f82::tusk::TUSK")
    {
        return "tusk::TUSK".to_string();
    } else if raw_currency
        .eq("0x5d4b302506645c37ff133b98c4b50a5ae14841659738d6d733d59d0d217a93bf::coin::COIN")
    {
        return "wUSDC::wUSDC".to_string();
    } else if raw_currency
        .eq("0xf325ce1300e8dac124071d3152c5c5ee6174914f8bc2161e88329cf579246efc::afsui::AFSUI")
    {
        return "afsui::AFSUI".to_string();
    } else if raw_currency.eq(
        "0xb2040456be6b1b16835cc32b2fe2b1dc4b55c8a9b3cab6fb962f06b570f4645c::SuiReward::SUIREWARD",
    ) {
        return "SuiReward::SUIREWARD".to_string();
    } else {
        let collection = raw_currency.split("::").collect::<Vec<&str>>();
        format!("[{}::{}]", collection[1], collection[2])
        // "[UNKNOWN]".to_string()
    }
}

pub fn sui_format_volume(volume: u128, currency: String) -> String {
    if currency.eq("usdc::USDC") {
        format!(
            "{}.{}",
            (volume / 1_000_000).to_string(),
            format!("{:0>6}", (volume % 1_000_000).to_string())
        )
    } else {
        format!(
            "{}.{}",
            (volume / 1_000_000_000).to_string(),
            format!("{:0>9}", (volume % 1_000_000_000).to_string())
        )
    }
}

#[allow(dead_code)]
pub fn key_pair_func() {
    let random_key_pair = SuiKeyPair::Ed25519(get_key_pair_from_rng(&mut rand::rngs::OsRng).1);
    println!(
        "random address : {}",
        SuiAddress::from(&random_key_pair.public())
    );
    let to_bytes = random_key_pair.to_bytes();

    let regenerated = SuiKeyPair::from_bytes(&to_bytes).unwrap();
    // let regenerated = SuiKeyPair::Ed25519(get_key_pair_from_bytes(&to_bytes).unwrap().1);
    println!(
        "regenerate address : {}",
        SuiAddress::from(&regenerated.public())
    );
}

/*

pub async fn fetch_sorted_gas_coins(
    rpc_client: &SuiClient,
    sender: &SuiAddress,
) -> anyhow::Result<Vec<(SuiObjectData, u64)>> {

    let mut gas_objects: Vec<(SuiObjectData, u64)> = vec![];
    let mut cursor = None;
    loop {
        let response = rpc_client
            .read_api()
            .get_owned_objects(
                sender.clone(),
                Some(SuiObjectResponseQuery {
                    filter: Some(SuiObjectDataFilter::MatchAll(vec![
                        SuiObjectDataFilter::StructType(GasCoin::type_()),
                    ])),
                    options: Some(SuiObjectDataOptions::full_content()),
                }),
                cursor,
                None,
            )
            .await?;

        let new_gas_objects: Vec<_> = response
            .data
            .into_iter()
            .filter_map(|maybe_object| {
                if let Some(object) = maybe_object.data {
                    let gas_coin = GasCoin::try_from(&object).unwrap();
                    let gas_balance = gas_coin.value();
                    if gas_balance > 0 {
                        Some((object, gas_balance))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        gas_objects.extend(new_gas_objects);

        if !response.has_next_page {
            break;
        };
        cursor = response.next_cursor;
    }

    gas_objects.sort_by(|(_, a), (_, b)| b.cmp(a));

    Ok(gas_objects)
}


*/
