use contract_analysis::*;
use ethereum_types::U256;
pub enum Transaction {
    NewContract {
        creation_address: U256,
        bytecode: String,
    },
    MethodCall {
        target_address: U256,
        calldata: String,
    },
}

impl TransactionDataProvider for Transaction {
    fn get_target_contract(&self) -> U256 {
        match self {
            Transaction::NewContract {
                creation_address: a,
                ..
            } => *a,
            Transaction::MethodCall {
                target_address: a, ..
            } => *a,
        }
    }
    fn get_target_method(&self) -> MethodType {
        match self {
            Transaction::NewContract { .. } => MethodType::Constructor,
            Transaction::MethodCall { calldata: d, .. } => {
                MethodType::Method(U256::from(u128::from_str_radix(d, 16).unwrap()))
            }
        }
    }
}

pub struct TransactionDependency(pub usize, pub usize);
