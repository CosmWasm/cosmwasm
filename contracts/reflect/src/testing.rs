use crate::msg::{CustomQuery, CustomResponse};

use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{to_binary, Binary, Coin, Extern, HumanAddr, StdResult};

/// A drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies_with_custom_querier(
    canonical_length: usize,
    contract_balance: &[Coin],
) -> Extern<MockStorage, MockApi, MockQuerier<CustomQuery>> {
    let contract_addr = HumanAddr::from(MOCK_CONTRACT_ADDR);
    let custom_querier: MockQuerier<CustomQuery> =
        MockQuerier::new(&[(&contract_addr, contract_balance)])
            .with_custom_handler(|query| Ok(execute(&query)));
    Extern {
        storage: MockStorage::default(),
        api: MockApi::new(canonical_length),
        querier: custom_querier,
    }
}

fn execute(query: &CustomQuery) -> StdResult<Binary> {
    let msg = match query {
        CustomQuery::Ping {} => "pong".to_string(),
        CustomQuery::Capital { text } => text.to_uppercase(),
    };
    to_binary(&CustomResponse { msg })
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::{from_binary, Querier, QueryRequest};

    #[test]
    fn custom_query_ping() {
        let res = execute(&CustomQuery::Ping {}).unwrap();
        let msg: CustomResponse = from_binary(&res).unwrap();
        assert_eq!(msg.msg, "pong".to_string());
    }

    #[test]
    fn custom_query_capitalize() {
        let res = execute(&CustomQuery::Capital {
            text: "fOObaR".to_string(),
        })
        .unwrap();
        let msg: CustomResponse = from_binary(&res).unwrap();
        assert_eq!(msg.msg, "FOOBAR".to_string());
    }

    #[test]
    fn custom_querier() {
        let deps = mock_dependencies_with_custom_querier(20, &[]);
        let req: QueryRequest<_> = CustomQuery::Capital {
            text: "food".to_string(),
        }
        .into();
        let res: CustomResponse = deps.querier.custom_query(&req).unwrap();
        assert_eq!(res.msg, "FOOD".to_string());
    }
}
