use crate::msg::{SpecialQuery, SpecialResponse};

use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{to_binary, Binary, Coin, ContractResult, Extern, HumanAddr, SystemResult};

/// A drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies_with_custom_querier(
    contract_balance: &[Coin],
) -> Extern<MockStorage, MockApi, MockQuerier<SpecialQuery>> {
    let contract_addr = HumanAddr::from(MOCK_CONTRACT_ADDR);
    let custom_querier: MockQuerier<SpecialQuery> =
        MockQuerier::new(&[(&contract_addr, contract_balance)])
            .with_custom_handler(|query| SystemResult::Ok(custom_query_execute(&query)));
    Extern {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
    }
}

pub fn custom_query_execute(query: &SpecialQuery) -> ContractResult<Binary> {
    let msg = match query {
        SpecialQuery::Ping {} => "pong".to_string(),
        SpecialQuery::Capitalized { text } => text.to_uppercase(),
    };
    to_binary(&SpecialResponse { msg }).into()
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::{from_binary, Querier, QueryRequest};

    #[test]
    fn custom_query_execute_ping() {
        let res = custom_query_execute(&SpecialQuery::Ping {}).unwrap();
        let response: SpecialResponse = from_binary(&res).unwrap();
        assert_eq!(response.msg, "pong");
    }

    #[test]
    fn custom_query_execute_capitalize() {
        let res = custom_query_execute(&SpecialQuery::Capitalized {
            text: "fOObaR".to_string(),
        })
        .unwrap();
        let response: SpecialResponse = from_binary(&res).unwrap();
        assert_eq!(response.msg, "FOOBAR");
    }

    #[test]
    fn custom_querier() {
        let deps = mock_dependencies_with_custom_querier(&[]);
        let req: QueryRequest<_> = SpecialQuery::Capitalized {
            text: "food".to_string(),
        }
        .into();
        let response: SpecialResponse = deps.querier.custom_query(&req).unwrap();
        assert_eq!(response.msg, "FOOD");
    }
}
