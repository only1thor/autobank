use reqwest::{Error, blocking::Client, blocking::Response};
use std::fs;

use crate::models::{AccountData, TransactionResponse};

pub fn get_accounts(access_token: String) -> AccountData {
    let client = Client::new();

    let account_response = client
        .get("https://api.sparebank1.no/personal/banking/accounts?includeCreditCardAccounts=true")
        .header("Authorization", format!("Bearer {}", access_token))
        .header(
            "Accept",
            "application/vnd.sparebank1.v1+json; charset=utf-8",
        )
        .send();

    let data: AccountData = match account_response {
        Ok(response) => {
            let text = response.text().expect("Failed to get response text");
            //fs::write("accounts.json", &text).expect("Failed to write JSON to file");
            serde_json::from_str(&text).expect("Failed to parse accounts JSON")
        }
        Err(err) => {
            panic!("Paniced: {}", err)
        }
    };

    data
}

pub fn get_transactions(access_token: String, account_key: &String) -> TransactionResponse {
    let client = Client::new();

    let url = format!(
        "https://api.sparebank1.no/personal/banking/transactions?accountKey={}",
        account_key
    );

    let transactions_respose = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header(
            "Accept",
            "application/vnd.sparebank1.v1+json; charset=utf-8",
        )
        .send();

    let data: TransactionResponse = match transactions_respose {
        Ok(response ) => {
            let text = response.text().expect("Failed to get response text");
            //fs::write("transactions.json", &text).expect("Failed to write JSON to file");
            serde_json::from_str(&text).expect("Fucky wucky")
        },
        Err(err ) => {
            panic!("Shieeet: {}", err)
        }
    };
    data
}

pub fn hello_world(access_token: String) -> Result<Response, Error> {
    let client = Client::new();
    client
        .get("https://api.sparebank1.no/common/helloworld")
        .header("Authorization", format!("Bearer {}", access_token))
        .header(
            "Accept",
            "application/vnd.sparebank1.v1+json; charset=utf-8",
        )
        .send()
}
