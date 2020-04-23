use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    from_slice, to_binary, Binary, Querier, QuerierResult, QueryRequest, StdResult, SystemError,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// CustomQuery is an override of QueryRequest::Custom to show this works and can be extended in the contract
pub enum CustomQuery {
    Ping {},
    Capital { text: String },
}

// TODO: do we want to standardize this somehow for all?
impl Into<QueryRequest<CustomQuery>> for CustomQuery {
    fn into(self) -> QueryRequest<CustomQuery> {
        QueryRequest::Custom(self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
// All return values of CustomQuery are CustomResponse
pub struct CustomResponse {
    pub msg: String,
}

impl CustomQuery {
    fn execute(&self) -> StdResult<Binary> {
        let msg = match self {
            CustomQuery::Ping {} => "pong".to_string(),
            CustomQuery::Capital { text } => text.to_uppercase(),
        };
        to_binary(&CustomResponse { msg })
    }
}

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
            QueryRequest::Custom(custom_query) => Ok(custom_query.execute().map_err(|e| e.into())),
            _ => Err(SystemError::InvalidRequest {
                error: "Mock only supports custom queries".to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::from_binary;

    #[test]
    fn custom_query_ping() {
        let res = CustomQuery::Ping {}.execute().unwrap();
        let msg: CustomResponse = from_binary(&res).unwrap();
        assert_eq!(msg.msg, "pong".to_string());
    }

    #[test]
    fn custom_query_capitalize() {
        let res = CustomQuery::Capital {
            text: "fOObaR".to_string(),
        }
        .execute()
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
        let res: CustomResponse = querier.parse_query(&req).unwrap();
        assert_eq!(res.msg, "FOOD".to_string());
    }
}
