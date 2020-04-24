use crate::msg::{CustomQuery, CustomResponse};

use cosmwasm_std::{
    from_slice, to_binary, Binary, Querier, QuerierResult, QueryRequest, StdResult, SystemError,
};

#[derive(Clone)]
pub struct CustomQuerier {}

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
        match &request {
            QueryRequest::Custom(custom_query) => Ok(execute(&custom_query).map_err(|e| e.into())),
            _ => Err(SystemError::UnsupportedRequest {
                kind: "non-custom".to_string(),
            }),
        }
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
    use cosmwasm_std::from_binary;

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
        let querier = CustomQuerier {};
        let req: QueryRequest<_> = CustomQuery::Capital {
            text: "food".to_string(),
        }
        .into();
        let res: CustomResponse = querier.custom_query(&req).unwrap();
        assert_eq!(res.msg, "FOOD".to_string());
    }
}
