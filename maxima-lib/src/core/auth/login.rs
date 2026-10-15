use regex::Regex;
use std::sync::LazyLock;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpListener;
use url::form_urlencoded;

use crate::core::{auth::storage::AuthError, clients::JUNO_PC_CLIENT_ID};

use super::context::AuthContext;

static HTTP_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([A-Za-z]+) +(.*) +(HTTP/[0-9][.][0-9])")
        .expect("HTTP pattern regex should be valid")
});

pub async fn begin_oauth_login_flow<'a>(context: &mut AuthContext<'a>) -> Result<(), AuthError> {
    open::that(context.nucleus_auth_url(JUNO_PC_CLIENT_ID, "code")?)?;
    let listener = TcpListener::bind("127.0.0.1:31033").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        let (read, _) = socket.split();
        let mut reader = BufReader::new(read);

        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let captures = match HTTP_PATTERN.captures(&line) {
            Some(cap) => cap,
            None => continue,
        };

        let path_and_query = captures.get(2).ok_or(AuthError::Query)?.as_str();
        if path_and_query.starts_with("/auth") {
            let query = path_and_query
                .split_once("?")
                .map(|(_, qs)| qs.trim())
                .map(|qs| {
                    let normalized = form_urlencoded::Serializer::new(String::new())
                        .extend_pairs(form_urlencoded::parse(qs.as_bytes()))
                        .finish();

                    form_urlencoded::parse(normalized.as_bytes())
                        .into_owned()
                        .collect::<Vec<(String, String)>>()
                })
                .ok_or(AuthError::Query)?;

            for (key, value) in query {
                if key == "code" {
                    context.set_code(value.as_str());
                    return Ok(());
                }
            }

            return Err(AuthError::NoAuthCode);
        }
    }
}
