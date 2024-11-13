use sui_json_rpc_types::BalanceChange;

use crate::sui_lib::sui_format_volume;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AccountHistory {
    ReceiveCoin(CoinDetail),
    SentCoin(CoinDetail),
    ReceiveToken(ReceiveTokenDetails),
    SentToken(SentTokenDetails),
    Stake(StakeDetail),
    Swap(SwapDetails),
    ComplexTransaction(ComplexTransactionDetails),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AccountPrintOption {
    All,
    Receive,
    Send,
}

pub trait AccountHistoryExtension {
    fn print(&self, options: AccountPrintOption);
}
impl AccountHistoryExtension for Vec<AccountHistory> {
    fn print(&self, options: AccountPrintOption) {
        let chain_currency = "sui::SUI";
        for item in self.iter() {
            match item {
                AccountHistory::ReceiveCoin(coin_detail) => {
                    if options == AccountPrintOption::All || options == AccountPrintOption::Receive
                    {
                        println!(
                            "Received {} {} => {}",
                            sui_format_volume(
                                coin_detail.volume,
                                chain_currency.to_string().clone()
                            ),
                            chain_currency,
                            coin_detail.receiver.clone(),
                        );
                    }
                }
                AccountHistory::SentCoin(coin_detail) => {
                    if options == AccountPrintOption::All || options == AccountPrintOption::Send {
                        println!(
                            "Sent To {} {} => {}",
                            sui_format_volume(
                                coin_detail.volume,
                                chain_currency.to_string().clone()
                            ),
                            chain_currency,
                            coin_detail.receiver.clone(),
                        );
                    }
                }
                AccountHistory::ReceiveToken(receive_token_details) => {
                    if options == AccountPrintOption::All || options == AccountPrintOption::Receive
                    {
                        println!(
                            "Received {} {} => {}",
                            sui_format_volume(
                                receive_token_details.volume,
                                receive_token_details.currency.clone()
                            ),
                            receive_token_details.currency.clone(),
                            receive_token_details.receiver.clone(),
                        );
                    }
                }
                AccountHistory::SentToken(sent_token_details) => {
                    if options == AccountPrintOption::All || options == AccountPrintOption::Send {
                        println!(
                            "Sent To {} {} => {}",
                            sui_format_volume(
                                sent_token_details.volume,
                                sent_token_details.currency.clone()
                            ),
                            sent_token_details.currency.clone(),
                            sent_token_details.receiver.clone(),
                        );
                    }
                }
                AccountHistory::Stake(stake_detail) => {
                    if options == AccountPrintOption::All {
                        println!(
                            "Staked {} {}",
                            sui_format_volume(stake_detail.volume, stake_detail.currency.clone()),
                            stake_detail.currency.clone()
                        );
                    }
                }
                AccountHistory::Swap(swap_details) => {
                    if options == AccountPrintOption::All {
                        println!(
                            "Swap : {} {} >> {} {}",
                            sui_format_volume(
                                swap_details.output_volume,
                                swap_details.output_currency.clone()
                            ),
                            swap_details.output_currency.clone(),
                            sui_format_volume(
                                swap_details.input_volume,
                                swap_details.input_currency.clone()
                            ),
                            swap_details.input_currency.clone(),
                        );
                    }
                }
                AccountHistory::ComplexTransaction(complex_transaction_details) => {
                    if options == AccountPrintOption::All {
                        println!(
                            "Complex Tx Digest : {}",
                            complex_transaction_details.digest.clone()
                        );
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StakeDetail {
    pub digest: String,
    pub time: u128,
    pub block_no: u128,
    pub currency: String,
    pub volume: u128,
    pub formated_volume: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ComplexTransactionDetails {
    pub digest: String,
    pub time: u128,
    pub block_no: u128,
    pub get_in_currency: Vec<String>,
    pub go_out_currency: Vec<String>,
    pub address_list: Vec<String>,
    pub currency_list: Vec<String>,
    pub balance_list: Vec<BalanceChange>,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct CoinDetail {
    pub digest: String,
    pub time: u128,
    pub block_no: u128,
    pub sender: String,
    pub receiver: String,
    pub currency: String,
    pub volume: u128,
    pub formated_volume: String,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct ReceiveTokenDetails {
    pub digest: String,
    pub time: u128,
    pub block_no: u128,
    pub sender: String,
    pub receiver: String,
    pub currency: String,
    pub volume: u128,
    pub formated_volume: String,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct SentTokenDetails {
    pub digest: String,
    pub time: u128,
    pub block_no: u128,
    pub sender: String,
    pub receiver: String,
    pub currency: String,
    pub volume: u128,
    pub formated_volume: String,
    pub gas: u128,
    pub formated_gas: String,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct SwapDetails {
    pub digest: String,
    pub time: u128,
    pub block_no: u128,
    pub input_currency: String,
    pub output_currency: String,
    pub input_volume: u128,
    pub output_volume: u128,
    pub input_formated_volume: String,
    pub output_formated_volume: String,
    pub gas: u128,
    pub formated_gas: String,
    pub commision_status: bool,
    pub commision_account: String,
    pub commision_volume: u128,
    pub formatted_commision: String,
}

/*
impl Default for Balance {
    fn default() -> Self {
        Balance {
            time: 0,
            block_no: 0,
            sender: String::new(),
            receiver: String::new(),
            input: false,
            volume: 0,
            currency: "[UNKNOWN]".to_string(),
            formated_volume: String::new(),
            r#type: BalanceType::Unknown,
        }
    }
}
*/
