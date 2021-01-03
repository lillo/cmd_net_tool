extern crate contract_analysis;
extern crate ethereum_types;
extern crate json;
extern crate ructe;

mod transaction;

use contract_analysis::*;
use ethereum_types::U256;
use json::JsonValue;
use std::env::args;
use std::fs;
use std::sync::{Arc, Mutex};
use transaction::{Transaction, TransactionDependency};
include!(concat!(env!("OUT_DIR"), "/templates.rs"));
use crate::templates::*;
fn main() -> std::io::Result<()> {
    //args().next();
    let file = args().skip(1).next();
    if let Some(f) = file {
        let content = &fs::read_to_string(&f);
        match content {
            Ok(c) => match json::parse(c) {
                Ok(j) => create_net(&parse_transactions(j)),
                Err(e) => panic!("Could not parse JSON: {}", e),
            },
            Err(e) => panic!("File Not found: {}", e),
        }
    } else {
        println!("Usage: cldb <file>.json");
    }
    Ok(())
}
fn parse_transactions(object: JsonValue) -> Vec<Transaction> {
    let mut ret = vec![];
    if let JsonValue::Array(a) = object {
        for transaction_json in a {
            if let JsonValue::Object(o) = transaction_json {
                ret.push(parse_transaction(o));
            } else {
                println!("JSON transactions should be objects");
            }
        }
    } else {
        println!("The JSON file should contain an array");
    }
    ret
}
fn parse_transaction(o: json::object::Object) -> Transaction {
    let t_type_js = o
        .get("type")
        .expect("Transactions should have a field 'type'");
    let t_type = if let JsonValue::Short(s) = t_type_js {
        s
    } else {
        panic!("field type should be a string")
    };
    let address_json = o.get("called_address");
    let address: U256;
    if let Some(JsonValue::Short(address_string)) = address_json {
        address = U256::from_dec_str(address_string).expect("numbers should be in decimal");
    } else {
        panic!("JSON transactions should have a field named 'called_address' containing a string");
    }
    let data: String = o
        .get("data")
        .expect("Transactions should have a field 'data'")
        .as_str()
        .unwrap()
        .to_owned();

    if t_type == "constructor" {
        Transaction::NewContract {
            creation_address: address,
            bytecode: data,
        }
    } else {
        Transaction::MethodCall {
            target_address: address,
            calldata: data,
        }
    }
}

fn create_net(transactions: &[Transaction]) {
    let mut netbuilder = NetBuilder::new();

    for transaction in transactions {
        if let Transaction::NewContract {
            creation_address: ca,
            bytecode: bc,
        } = transaction
        {
            let analyzed = analyze_contract_default(&parse_bytecode(bc)).unwrap();
            netbuilder.register_contract(*ca, analyzed);
        }
        netbuilder.new_transaction(transaction, Box::from(|| {}));
    }
    create_graphical_net(netbuilder)
}

fn parse_bytecode(bc: &String) -> Vec<u8> {
    let mut buffer = "".to_owned();
    let mut ret = vec![];
    for chr in bc.chars() {
        buffer.push(chr);
        if buffer.len() == 2 {
            ret.push(u8::from_str_radix(&buffer, 16).unwrap());
            buffer = "".to_owned();
        }
    }

    ret
}

fn create_graphical_net(nb: NetBuilder) {
    let start_trans = nb.finalize();
    let buffer = String::new();
    let mut dep_list = vec![];
    print_trans(&start_trans, &mut dep_list);
    let mut out_file = std::fs::File::create("./output.html").unwrap();
    output_html(&mut out_file, &dep_list).unwrap();
}
fn print_trans(
    trans: &Vec<Arc<Mutex<contract_analysis::net::transaction::Transaction>>>,
    dep_list: &mut Vec<TransactionDependency>,
) {
    for t in trans {
        println!("Transaction #{}", t.lock().unwrap().id);
        let lock = t.lock().unwrap();
        let deps = &lock.dependencies;
        for d in deps {
            println!("{} => {}", lock.id, d.lock().unwrap().id);
            dep_list.push(TransactionDependency(lock.id, d.lock().unwrap().id));
        }
        print_trans(deps, dep_list);
    }
}
