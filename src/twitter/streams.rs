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

#[derive(Debug, Deserialize)]
pub struct StreamRulesUpdateMeta {
    #[allow(dead_code)]
    pub summary: Option<serde_json::Value>,
    #[allow(dead_code)]
    pub sent: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StreamRulesUpdateResponse {
    #[serde(default)]
    pub data: Vec<StreamRule>,
    #[allow(dead_code)]
    pub meta: Option<StreamRulesUpdateMeta>,
}

#[derive(Debug, Deserialize)]
pub struct StreamRulesUpdateError {
    pub message: String,
}

pub struct StreamRules;

#[derive(Debug)]
pub struct AddStreamRule {
    value: String,
    tag: Option<String>,
}

#[derive(Debug)]
pub struct DeleteStreamRules {
    ids: Vec<String>,
}

#[derive(Debug, Serialize)]
struct AddStreamRulePayload<'a> {
    value: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct DeleteStreamRulesPayload<'a> {
    ids: &'a [String],
}

#[derive(Debug, Serialize)]
struct StreamRulesUpdatePayload<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    add: Option<Vec<AddStreamRulePayload<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delete: Option<DeleteStreamRulesPayload<'a>>,
}

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

impl AddStreamRule {
    pub fn new(value: impl Into<String>, tag: Option<String>) -> Self {
        Self {
            value: value.into(),
            tag,
        }
    }

    fn url(&self) -> &'static str {
        "https://api.x.com/2/tweets/search/stream/rules"
    }

    pub fn send(&self) -> Result<Response<StreamRulesUpdateResponse>, StreamRulesUpdateError> {
        let url = self.url();
        let authorization = bearer_auth_header();
        let body = serde_json::to_string(&StreamRulesUpdatePayload {
            add: Some(vec![AddStreamRulePayload {
                value: self.value.as_str(),
                tag: self.tag.as_deref(),
            }]),
            delete: None,
        })
        .map_err(|err| StreamRulesUpdateError {
            message: err.to_string(),
        })?;

        let response = curl_rest::Client::default()
            .post()
            .header(curl_rest::Header::Authorization(authorization.into()))
            .body_json(body)
            .send(url)
            .map_err(|err| StreamRulesUpdateError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let rules_data: StreamRulesUpdateResponse = serde_json::from_slice(&response.body)
                .map_err(|err| StreamRulesUpdateError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: rules_data,
            })
        } else {
            Err(StreamRulesUpdateError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl DeleteStreamRules {
    pub fn new(ids: Vec<String>) -> Self {
        Self { ids }
    }

    fn url(&self) -> &'static str {
        "https://api.x.com/2/tweets/search/stream/rules"
    }

    pub fn send(&self) -> Result<Response<StreamRulesUpdateResponse>, StreamRulesUpdateError> {
        let url = self.url();
        let authorization = bearer_auth_header();
        let body = serde_json::to_string(&StreamRulesUpdatePayload {
            add: None,
            delete: Some(DeleteStreamRulesPayload {
                ids: self.ids.as_slice(),
            }),
        })
        .map_err(|err| StreamRulesUpdateError {
            message: err.to_string(),
        })?;

        let response = curl_rest::Client::default()
            .post()
            .header(curl_rest::Header::Authorization(authorization.into()))
            .body_json(body)
            .send(url)
            .map_err(|err| StreamRulesUpdateError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let rules_data: StreamRulesUpdateResponse = serde_json::from_slice(&response.body)
                .map_err(|err| StreamRulesUpdateError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: rules_data,
            })
        } else {
            Err(StreamRulesUpdateError {
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

    #[test]
    fn test_add_stream_rule_url_uses_rules_endpoint() {
        let endpoint = AddStreamRule::new("from:openai", Some("openai".to_string()));

        assert_eq!(
            endpoint.url(),
            "https://api.x.com/2/tweets/search/stream/rules"
        );
    }

    #[test]
    fn test_delete_stream_rules_url_uses_rules_endpoint() {
        let endpoint = DeleteStreamRules::new(vec!["1".to_string(), "2".to_string()]);

        assert_eq!(
            endpoint.url(),
            "https://api.x.com/2/tweets/search/stream/rules"
        );
    }
}
