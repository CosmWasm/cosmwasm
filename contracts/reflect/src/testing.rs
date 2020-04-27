use crate::msg::{CustomQuery, CustomResponse};

use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    from_slice, to_binary, Binary, Coin, Extern, Querier, QuerierResult, QueryRequest, StdResult,
    SystemError,
};

#[derive(Clone)]
pub struct CustomQuerier {
    base: MockQuerier,
}

impl CustomQuerier {
    pub fn new(base: MockQuerier) -> Self {
        CustomQuerier { base }
    }
}

impl Querier for CustomQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // parse into our custom query class
        let request: QueryRequest<CustomQuery> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return Err(SystemError::InvalidRequest {
                    error: format!("Parsing QueryRequest: {}", e),
                })
            }
        };
        if let QueryRequest::Custom(custom_query) = &request {
            Ok(execute(&custom_query))
        } else {
            self.base.handle_query(&request)
        }
    }
}

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies(
    canonical_length: usize,
    contract_balance: &[Coin],
) -> Extern<MockStorage, MockApi, CustomQuerier> {
    let base = cosmwasm_std::testing::mock_dependencies(canonical_length, contract_balance);
    base.change_querier(CustomQuerier::new)
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
    use cosmwasm_std::from_binary;
    use cosmwasm_std::testing::mock_dependencies;

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
        let base = mock_dependencies(20, &[]).querier;
        let querier = CustomQuerier::new(base);
        let req: QueryRequest<_> = CustomQuery::Capital {
            text: "food".to_string(),
        }
        .into();
        let res: CustomResponse = querier.custom_query(&req).unwrap();
        assert_eq!(res.msg, "FOOD".to_string());
    }
}
