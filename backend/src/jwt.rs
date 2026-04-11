use serde::Serialize;

use crate::UserId;

pub struct Authenticator;

#[derive(Serialize)]
#[serde(transparent)]
pub struct Cookie(String);

use axum_extra::extract::{CookieJar, cookie::Cookie as Kjeks};

impl From<Cookie> for Kjeks<'static> {
    fn from(value: Cookie) -> Self {
        Kjeks::new("session_id", value.0)
    }
}

impl Authenticator {
    pub(crate) fn authenticate(&self, user_id: UserId) -> Cookie {
        Cookie(format!("{user_id}"))
    }

    pub(crate) fn validate(&self, cookie: &Cookie) -> Option<UserId> {
        cookie.0.parse().ok()
    }

    pub(crate) fn new() -> Self {
        Self
    }
}

impl Cookie {
    pub(crate) fn from_jar(jar: &CookieJar) -> Option<Cookie> {
        jar.get("session_id").map(|x| Cookie(x.value().to_owned()))
    }
}
