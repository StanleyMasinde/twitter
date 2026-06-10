use serde::{Deserialize, Serialize};

use crate::{
    twitter::{
        Response,
        params::{
            Pagination, apply_query_params, oauth_param_list, paginated_oauth_entries,
            print_next_page_hint,
        },
    },
    utils::{get_current_user_id, oauth_get_header, oauth_post_header},
};

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

#[derive(Debug, Deserialize)]
pub struct CreateConversationData {
    pub dm_conversation_id: String,
    pub dm_event_id: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateConversationResponse {
    pub data: CreateConversationData,
}

#[derive(Debug, Deserialize)]
pub struct CreateConversationError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct SendWithParticipantMessageData {
    pub dm_conversation_id: String,
    pub dm_event_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SendWithParticipantMessageResponse {
    pub data: SendWithParticipantMessageData,
}

#[derive(Debug, Deserialize)]
pub struct SendWithParticipantMessageError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ConversationDmEvent {
    pub dm_conversation_id: String,
    pub dm_event_id: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct ConversationDmEventsMeta {
    #[allow(dead_code)]
    pub result_count: u32,
    pub next_token: Option<String>,
    #[allow(dead_code)]
    pub previous_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConversationDmEventsResponse {
    #[serde(default)]
    pub data: Vec<ConversationDmEvent>,
    #[allow(dead_code)]
    pub meta: Option<ConversationDmEventsMeta>,
}

#[derive(Debug, Deserialize)]
pub struct ConversationDmEventsError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct UserDmEvent {
    pub dm_conversation_id: String,
    pub dm_event_id: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct UserDmEventsMeta {
    #[allow(dead_code)]
    pub result_count: u32,
    pub next_token: Option<String>,
    #[allow(dead_code)]
    pub previous_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UserDmEventsResponse {
    #[serde(default)]
    pub data: Vec<UserDmEvent>,
    #[allow(dead_code)]
    pub meta: Option<UserDmEventsMeta>,
}

#[derive(Debug, Deserialize)]
pub struct UserDmEventsError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ParticipantDmEvent {
    pub dm_conversation_id: String,
    pub dm_event_id: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct ParticipantDmEventsMeta {
    #[allow(dead_code)]
    pub result_count: u32,
    pub next_token: Option<String>,
    #[allow(dead_code)]
    pub previous_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ParticipantDmEventsResponse {
    #[serde(default)]
    pub data: Vec<ParticipantDmEvent>,
    #[allow(dead_code)]
    pub meta: Option<ParticipantDmEventsMeta>,
}

#[derive(Debug, Deserialize)]
pub struct ParticipantDmEventsError {
    pub message: String,
}

#[derive(Debug)]
pub struct SendConversationMessage {
    conversation_id: String,
    text: String,
}

#[derive(Debug)]
pub struct ConversationDmEvents {
    conversation_id: String,
    max_results: u8,
    pagination: Pagination,
}

#[derive(Debug)]
pub struct UserDmEvents {
    user_id: String,
    max_results: u8,
    pagination: Pagination,
}

#[derive(Debug)]
pub struct ParticipantDmEvents {
    participant_id: String,
    max_results: u8,
    pagination: Pagination,
}

#[derive(Debug)]
pub struct SendWithParticipantMessage {
    participant_id: String,
    text: String,
}

#[derive(Debug)]
pub struct CreateConversation {
    participant_ids: Vec<String>,
    text: String,
}

#[derive(Serialize)]
struct SendConversationMessageBody<'a> {
    text: &'a str,
}

#[derive(Serialize)]
struct CreateConversationBody<'a> {
    conversation_type: &'static str,
    participant_ids: &'a [String],
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

impl ConversationDmEvents {
    pub fn new(conversation_id: impl Into<String>) -> Self {
        Self {
            conversation_id: conversation_id.into(),
            max_results: 10,
            pagination: Pagination::new(),
        }
    }

    pub fn max_results(mut self, max_results: u8) -> Self {
        self.max_results = max_results.clamp(1, 100);
        self
    }

    pub fn pagination_token(mut self, token: impl Into<String>) -> Self {
        self.pagination = self.pagination.pagination_token(token);
        self
    }

    fn url(&self) -> String {
        format!(
            "https://api.x.com/2/dm_conversations/{}/dm_events",
            self.conversation_id
        )
    }

    pub fn fetch(
        &self,
    ) -> Result<Response<ConversationDmEventsResponse>, ConversationDmEventsError> {
        let url = self.url();
        let oauth_entries = paginated_oauth_entries(self.max_results, &[], &self.pagination);
        let auth_header = oauth_get_header(url.as_str(), &oauth_param_list(oauth_entries));

        let max_results_query = self.max_results.to_string();
        let pagination_entries = self.pagination.oauth_entries();
        let mut request = curl_rest::Client::default()
            .get()
            .query_param_kv("max_results", max_results_query.as_str());
        request = apply_query_params(request, &pagination_entries);

        let response = request
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| ConversationDmEventsError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: ConversationDmEventsResponse = serde_json::from_slice(&response.body)
                .map_err(|err| ConversationDmEventsError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            Err(ConversationDmEventsError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl UserDmEvents {
    pub fn current_user() -> Result<Self, UserDmEventsError> {
        let user_id = get_current_user_id().map_err(|message| UserDmEventsError { message })?;
        Ok(Self {
            user_id,
            max_results: 10,
            pagination: Pagination::new(),
        })
    }

    pub fn max_results(mut self, max_results: u8) -> Self {
        self.max_results = max_results.clamp(1, 100);
        self
    }

    pub fn pagination_token(mut self, token: impl Into<String>) -> Self {
        self.pagination = self.pagination.pagination_token(token);
        self
    }

    fn url(&self) -> String {
        format!("https://api.x.com/2/users/{}/dm_events", self.user_id)
    }

    pub fn fetch(&self) -> Result<Response<UserDmEventsResponse>, UserDmEventsError> {
        let url = self.url();
        let oauth_entries = paginated_oauth_entries(self.max_results, &[], &self.pagination);
        let auth_header = oauth_get_header(url.as_str(), &oauth_param_list(oauth_entries));

        let max_results_query = self.max_results.to_string();
        let pagination_entries = self.pagination.oauth_entries();
        let mut request = curl_rest::Client::default()
            .get()
            .query_param_kv("max_results", max_results_query.as_str());
        request = apply_query_params(request, &pagination_entries);

        let response = request
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| UserDmEventsError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: UserDmEventsResponse =
                serde_json::from_slice(&response.body).map_err(|err| UserDmEventsError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            Err(UserDmEventsError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl ParticipantDmEvents {
    pub fn new(participant_id: impl Into<String>) -> Self {
        Self {
            participant_id: participant_id.into(),
            max_results: 10,
            pagination: Pagination::new(),
        }
    }

    pub fn max_results(mut self, max_results: u8) -> Self {
        self.max_results = max_results.clamp(1, 100);
        self
    }

    pub fn pagination_token(mut self, token: impl Into<String>) -> Self {
        self.pagination = self.pagination.pagination_token(token);
        self
    }

    fn url(&self) -> String {
        format!(
            "https://api.x.com/2/dm_conversations/with/{}/dm_events",
            self.participant_id
        )
    }

    pub fn fetch(&self) -> Result<Response<ParticipantDmEventsResponse>, ParticipantDmEventsError> {
        let url = self.url();
        let oauth_entries = paginated_oauth_entries(self.max_results, &[], &self.pagination);
        let auth_header = oauth_get_header(url.as_str(), &oauth_param_list(oauth_entries));

        let max_results_query = self.max_results.to_string();
        let pagination_entries = self.pagination.oauth_entries();
        let mut request = curl_rest::Client::default()
            .get()
            .query_param_kv("max_results", max_results_query.as_str());
        request = apply_query_params(request, &pagination_entries);

        let response = request
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .send(url.as_str())
            .map_err(|err| ParticipantDmEventsError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: ParticipantDmEventsResponse = serde_json::from_slice(&response.body)
                .map_err(|err| ParticipantDmEventsError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            Err(ParticipantDmEventsError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl SendWithParticipantMessage {
    pub fn new(participant_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            participant_id: participant_id.into(),
            text: text.into(),
        }
    }

    fn url(&self) -> String {
        format!(
            "https://api.x.com/2/dm_conversations/with/{}/messages",
            self.participant_id
        )
    }

    pub fn send(
        &self,
    ) -> Result<Response<SendWithParticipantMessageResponse>, SendWithParticipantMessageError> {
        let url = self.url();
        let auth_header = oauth_post_header(url.as_str(), &());
        let body = serde_json::to_string(&SendConversationMessageBody {
            text: self.text.as_str(),
        })
        .map_err(|err| SendWithParticipantMessageError {
            message: err.to_string(),
        })?;

        let response = curl_rest::Client::default()
            .post()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .body_json(body)
            .send(url.as_str())
            .map_err(|err| SendWithParticipantMessageError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: SendWithParticipantMessageResponse = serde_json::from_slice(&response.body)
                .map_err(|err| {
                SendWithParticipantMessageError {
                    message: err.to_string(),
                }
            })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            Err(SendWithParticipantMessageError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

impl CreateConversation {
    pub fn new(participant_ids: Vec<String>, text: impl Into<String>) -> Self {
        Self {
            participant_ids,
            text: text.into(),
        }
    }

    fn url(&self) -> &'static str {
        "https://api.x.com/2/dm_conversations"
    }

    pub fn send(&self) -> Result<Response<CreateConversationResponse>, CreateConversationError> {
        let url = self.url();
        let auth_header = oauth_post_header(url, &());
        let body = serde_json::to_string(&CreateConversationBody {
            conversation_type: "GroupDM",
            participant_ids: self.participant_ids.as_slice(),
            text: self.text.as_str(),
        })
        .map_err(|err| CreateConversationError {
            message: err.to_string(),
        })?;

        let response = curl_rest::Client::default()
            .post()
            .header(curl_rest::Header::Authorization(auth_header.into()))
            .body_json(body)
            .send(url)
            .map_err(|err| CreateConversationError {
                message: err.to_string(),
            })?;

        if (200..300).contains(&response.status.as_u16()) {
            let data: CreateConversationResponse =
                serde_json::from_slice(&response.body).map_err(|err| CreateConversationError {
                    message: err.to_string(),
                })?;
            Ok(Response {
                status: response.status.as_u16(),
                content: data,
            })
        } else {
            Err(CreateConversationError {
                message: String::from_utf8_lossy(&response.body).to_string(),
            })
        }
    }
}

fn print_dm_events(conversation_id: &str, event_id: &str, text: &str) {
    println!(
        "Conversation Id: {}\nMessage Id: {}\nText: {}",
        conversation_id, event_id, text
    );
}

pub fn print_conversation_dm_events(response: &ConversationDmEventsResponse) {
    if response.data.is_empty() {
        println!("No DM events found.");
        return;
    }

    for (index, event) in response.data.iter().enumerate() {
        if index > 0 {
            println!();
            println!();
        }

        print_dm_events(&event.dm_conversation_id, &event.dm_event_id, &event.text);
    }

    print_next_page_hint(
        response
            .meta
            .as_ref()
            .and_then(|meta| meta.next_token.as_deref()),
    );
}

pub fn print_user_dm_events(response: &UserDmEventsResponse) {
    if response.data.is_empty() {
        println!("No DM events found.");
        return;
    }

    for (index, event) in response.data.iter().enumerate() {
        if index > 0 {
            println!();
            println!();
        }

        print_dm_events(&event.dm_conversation_id, &event.dm_event_id, &event.text);
    }

    print_next_page_hint(
        response
            .meta
            .as_ref()
            .and_then(|meta| meta.next_token.as_deref()),
    );
}

pub fn print_participant_dm_events(response: &ParticipantDmEventsResponse) {
    if response.data.is_empty() {
        println!("No DM events found.");
        return;
    }

    for (index, event) in response.data.iter().enumerate() {
        if index > 0 {
            println!();
            println!();
        }

        print_dm_events(&event.dm_conversation_id, &event.dm_event_id, &event.text);
    }

    print_next_page_hint(
        response
            .meta
            .as_ref()
            .and_then(|meta| meta.next_token.as_deref()),
    );
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

impl std::fmt::Display for CreateConversationResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Conversation Id: {}\nMessage Id: {}\nText: {}",
            self.data.dm_conversation_id, self.data.dm_event_id, self.data.text
        )
    }
}

impl std::fmt::Display for SendWithParticipantMessageResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Conversation Id: {}\nMessage Id: {}",
            self.data.dm_conversation_id, self.data.dm_event_id
        )
    }
}

impl std::fmt::Display for ConversationDmEventsResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, event) in self.data.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
                writeln!(f)?;
            }

            write!(
                f,
                "Conversation Id: {}\nMessage Id: {}\nText: {}",
                event.dm_conversation_id, event.dm_event_id, event.text
            )?;
        }

        Ok(())
    }
}

impl std::fmt::Display for UserDmEventsResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, event) in self.data.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
                writeln!(f)?;
            }

            write!(
                f,
                "Conversation Id: {}\nMessage Id: {}\nText: {}",
                event.dm_conversation_id, event.dm_event_id, event.text
            )?;
        }

        Ok(())
    }
}

impl std::fmt::Display for ParticipantDmEventsResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, event) in self.data.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
                writeln!(f)?;
            }

            write!(
                f,
                "Conversation Id: {}\nMessage Id: {}\nText: {}",
                event.dm_conversation_id, event.dm_event_id, event.text
            )?;
        }

        Ok(())
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

    #[test]
    fn test_create_conversation_uses_collection_url() {
        let endpoint = CreateConversation::new(vec!["1".to_string(), "2".to_string()], "hello");

        assert_eq!(endpoint.url(), "https://api.x.com/2/dm_conversations");
    }

    #[test]
    fn test_send_with_participant_message_url_uses_participant_id() {
        let endpoint = SendWithParticipantMessage::new("123", "hello");

        assert_eq!(
            endpoint.url(),
            "https://api.x.com/2/dm_conversations/with/123/messages"
        );
    }

    #[test]
    fn test_conversation_dm_events_url_uses_conversation_id() {
        let endpoint = ConversationDmEvents::new("123");

        assert_eq!(
            endpoint.url(),
            "https://api.x.com/2/dm_conversations/123/dm_events"
        );
    }

    #[test]
    fn test_user_dm_events_url_uses_current_user_id() {
        let endpoint = UserDmEvents {
            user_id: "123".to_string(),
            max_results: 10,
            pagination: Pagination::new(),
        };

        assert_eq!(endpoint.url(), "https://api.x.com/2/users/123/dm_events");
    }

    #[test]
    fn test_participant_dm_events_url_uses_participant_id() {
        let endpoint = ParticipantDmEvents::new("123");

        assert_eq!(
            endpoint.url(),
            "https://api.x.com/2/dm_conversations/with/123/dm_events"
        );
    }
}
