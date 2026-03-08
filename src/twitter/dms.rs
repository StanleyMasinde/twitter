use serde::{Deserialize, Serialize};

use crate::{twitter::Response, utils::oauth_post_header};

#[derive(Debug, Deserialize)]
pub struct SendConversationMessageData {
    pub dm_conversation_id: String,
    pub dm_event_id: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct SendConversationMessageResponse {
    pub data: SendConversationMessageData,
}

#[derive(Debug, Deserialize)]
pub struct SendConversationMessageError {
    pub message: String,
}

#[derive(Debug)]
pub struct SendConversationMessage {
    conversation_id: String,
    text: String,
}

#[derive(Serialize)]
struct SendConversationMessageBody<'a> {
    text: &'a str,
}

impl SendConversationMessage {
    pub fn new(conversation_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            conversation_id: conversation_id.into(),
            text: text.into(),
        }
    }

    fn url(&self) -> String {
        format!(
            "https://api.x.com/2/dm_conversations/{}/messages",
            self.conversation_id
        )
    }

    pub fn send(
        &self,
    ) -> Result<Response<SendConversationMessageResponse>, SendConversationMessageError> {
        let url = self.url();
        let auth_header = oauth_post_header(url.as_str(), &());
        let body = serde_json::to_string(&SendConversationMessageBody {
            text: self.text.as_str(),
        })
        .map_err(|err| SendConversationMessageError {
            message: err.to_string(),
        })?;

        let response = curl_rest::Client::default()
            .post()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .body_json(body)
            .send(url.as_str())
            .map_err(|err| SendConversationMessageError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: SendConversationMessageResponse = serde_json::from_slice(&response.body)
                .map_err(|err| SendConversationMessageError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            Err(SendConversationMessageError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl std::fmt::Display for SendConversationMessageResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Conversation Id: {}\nMessage Id: {}\nText: {}",
            self.data.dm_conversation_id, self.data.dm_event_id, self.data.text
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_conversation_message_url_uses_conversation_id() {
        let endpoint = SendConversationMessage::new("123", "hello");

        assert_eq!(
            endpoint.url(),
            "https://api.x.com/2/dm_conversations/123/messages"
        );
    }
}
