use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{twitter::Response, utils::bearer_auth_header};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StreamRule {
    pub id: String,
    pub value: String,
    #[serde(default)]
    pub tag: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StreamRulesMeta {
    #[allow(dead_code)]
    pub sent: Option<String>,
    #[allow(dead_code)]
    pub result_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct StreamRulesResponse {
    #[serde(default)]
    pub data: Vec<StreamRule>,
    #[allow(dead_code)]
    pub meta: Option<StreamRulesMeta>,
}

#[derive(Debug, Deserialize)]
pub struct StreamRulesError {
    pub message: String,
}

pub struct StreamRules;

impl Display for StreamRulesResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, rule) in self.data.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
                writeln!(f)?;
            }

            write!(f, "Rule Id: {}\nValue: {}", rule.id, rule.value)?;
            if let Some(tag) = &rule.tag {
                write!(f, "\nTag: {}", tag)?;
            }
        }

        Ok(())
    }
}

impl StreamRules {
    fn url(&self) -> &'static str {
        "https://api.x.com/2/tweets/search/stream/rules"
    }

    pub fn fetch(&self) -> Result<Response<StreamRulesResponse>, StreamRulesError> {
        let url = self.url();
        let authorization = bearer_auth_header();

        let response = curl_rest::Client::default()
            .get()
            .header(curl_rest::Header::Authorization(authorization.into()))
            .send(url)
            .map_err(|err| StreamRulesError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let rules_data: StreamRulesResponse =
                serde_json::from_slice(&response.body).map_err(|err| StreamRulesError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: rules_data,
            })
        } else {
            Err(StreamRulesError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_rules_url_uses_rules_endpoint() {
        let endpoint = StreamRules;

        assert_eq!(
            endpoint.url(),
            "https://api.x.com/2/tweets/search/stream/rules"
        );
    }
}
