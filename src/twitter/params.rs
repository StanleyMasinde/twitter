/// Pagination for list/search endpoints.
#[derive(Debug, Default, Clone)]
pub struct Pagination {
    pub pagination_token: Option<String>,
}

impl Pagination {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pagination_token(mut self, token: impl Into<String>) -> Self {
        self.pagination_token = Some(token.into());
        self
    }

    pub fn oauth_entries(&self) -> Vec<(&'static str, String)> {
        let mut entries = Vec::new();
        if let Some(token) = &self.pagination_token {
            entries.push(("pagination_token", token.clone()));
        }
        entries
    }
}

/// Time window filters for search and timeline endpoints.
#[derive(Debug, Default, Clone)]
pub struct TimeParams {
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub since_id: Option<String>,
    pub until_id: Option<String>,
}

impl TimeParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_time(mut self, value: impl Into<String>) -> Self {
        self.start_time = Some(value.into());
        self
    }

    pub fn end_time(mut self, value: impl Into<String>) -> Self {
        self.end_time = Some(value.into());
        self
    }

    pub fn since_id(mut self, value: impl Into<String>) -> Self {
        self.since_id = Some(value.into());
        self
    }

    pub fn until_id(mut self, value: impl Into<String>) -> Self {
        self.until_id = Some(value.into());
        self
    }

    pub fn oauth_entries(&self) -> Vec<(&'static str, String)> {
        let mut entries = Vec::new();
        if let Some(value) = &self.start_time {
            entries.push(("start_time", value.clone()));
        }
        if let Some(value) = &self.end_time {
            entries.push(("end_time", value.clone()));
        }
        if let Some(value) = &self.since_id {
            entries.push(("since_id", value.clone()));
        }
        if let Some(value) = &self.until_id {
            entries.push(("until_id", value.clone()));
        }
        entries
    }
}

/// Extra parameters for tweet search endpoints.
#[derive(Debug, Default, Clone)]
pub struct SearchParams {
    pub sort_order: Option<String>,
    pub granularity: Option<String>,
    pub time: TimeParams,
    pub pagination: Pagination,
}

impl SearchParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sort_order(mut self, value: impl Into<String>) -> Self {
        self.sort_order = Some(value.into());
        self
    }

    pub fn granularity(mut self, value: impl Into<String>) -> Self {
        self.granularity = Some(value.into());
        self
    }

    pub fn oauth_entries(&self) -> Vec<(&'static str, String)> {
        let mut entries = self.time.oauth_entries();
        entries.extend(self.pagination.oauth_entries());
        if let Some(value) = &self.sort_order {
            entries.push(("sort_order", value.clone()));
        }
        if let Some(value) = &self.granularity {
            entries.push(("granularity", value.clone()));
        }
        entries
    }
}

pub fn oauth_param_list(
    entries: Vec<(&'static str, String)>,
) -> oauth::ParameterList<&'static str, String> {
    oauth::ParameterList::new(entries)
}

pub fn apply_query_params<'a>(
    mut request: curl_rest::Client<'a>,
    entries: &'a [(&'static str, String)],
) -> curl_rest::Client<'a> {
    for (key, value) in entries {
        request = request.query_param_kv(*key, value.as_str());
    }
    request
}

pub fn print_next_page_hint(next_token: Option<&str>) {
    if let Some(token) = next_token {
        println!("\nNext page: --pagination-token {token}");
    }
}

pub fn collect_oauth_entries(
    base: Vec<(&'static str, String)>,
    extra: &[(&'static str, String)],
) -> Vec<(&'static str, String)> {
    let mut entries = base;
    entries.extend_from_slice(extra);
    entries
}

pub fn max_results_entry(max_results: u8) -> (&'static str, String) {
    ("max_results", max_results.to_string())
}

pub fn tweet_field_entries() -> Vec<(&'static str, String)> {
    vec![
        ("tweet.fields", super::TWEET_FIELDS.to_string()),
        ("user.fields", super::USER_FIELDS.to_string()),
        ("expansions", super::EXPANSIONS.to_string()),
    ]
}

pub fn user_field_entries() -> Vec<(&'static str, String)> {
    vec![("user.fields", super::USER_FIELDS.to_string())]
}

pub fn paginated_oauth_entries(
    max_results: u8,
    field_entries: &[(&'static str, String)],
    pagination: &Pagination,
) -> Vec<(&'static str, String)> {
    collect_oauth_entries(
        collect_oauth_entries(vec![max_results_entry(max_results)], field_entries),
        &pagination.oauth_entries(),
    )
}
