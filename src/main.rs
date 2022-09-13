use std::env;
use web3::contract::Options;
use web3::{types, ethabi::TopicFilter, contract::{Contract}};
use std::fmt;


#[derive(Debug)]
struct Token {
    hash: types::H256,
    address: types::Address,
    name: String,
    supply: f64,
    
}

impl Token {
    fn new(hash:types::H256, address:types::Address, name:String, supply:f64) -> Self{
        Self{
            hash,
            address,
            name,
            supply
        }
    }
}

impl fmt::Display for Token{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Transaction Hash: {:?}\nContract Address: {:?}\nToken Name: {}\nTotal Supply: {}\n", self.hash, self.address, self.name, self.supply)
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let ws = web3::transports::WebSocket::new(&env::var("NODE").unwrap())
        .await
        .unwrap();

    let web3_client = web3::Web3::new(ws);

    let mut block = types::U64::zero();

    loop {
        let last_block = web3_client
            .eth()
            .block(types::BlockId::Number(types::BlockNumber::Latest))
            .await
            .unwrap()
            .unwrap();

        let block_filter = types::FilterBuilder::default()
            .block_hash(last_block.hash.unwrap())
            .topic_filter(TopicFilter{
                topic1: web3::ethabi::Topic::This(types::H256::zero()),
                ..Default::default()
            })
            .build();

        let block_logs = match web3_client.eth().logs(block_filter).await {
            Ok(logs) => logs,
            _ => continue, 
        };

        if last_block.number.unwrap() != block {
            block = last_block.number.unwrap();

            for log_tx in block_logs{

                let tx = match web3_client.eth().transaction(types::TransactionId::Hash(log_tx.transaction_hash.unwrap())).await{
                    Ok(Some(tx)) => tx,
                    _ => continue,
                };

                if tx.to == None{
                    let smart_contract = match Contract::from_json(web3_client.eth(), log_tx.address, include_bytes!("erc20_abi.json")){
                        Ok(ca) => ca,
                        _ => continue,
                    };

                    let name_token: String = match smart_contract
                        .query("name", (), None, Options::default(), None)
                        .await
                    {
                        Ok(name_token) => name_token,
                        _ => break,
                    };

                    let supply_token: types::U256 = match smart_contract
                    .query("totalSupply", (), None, Options::default(), None)
                    .await
                    {
                    Ok(sypply_token) => sypply_token,
                    _ => break,
                    };

                    let token = Token::new(
                        log_tx.transaction_hash.unwrap(), 
                        types::Address::from(log_tx.address), 
                        name_token, 
                        convert_wei(supply_token)
                    );

                    println!("{}", token);
               
                    break;
                }
            }
        }
    }   
}

fn convert_wei(wei: types::U256) -> f64{
    let wei = wei.as_u128() as f64;
    wei / 10_f64.powf(18.0)
}