use bytes::Bytes;
use pingora::http::ResponseHeader;
use pingora::prelude::*;

pub struct Response {
    status: u16,
    headers: Vec<(String, String)>,
    body: Bytes,
}

impl Response {
    pub fn new(status: u16, body: impl Into<Bytes>) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: body.into(),
        }
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    pub async fn send(self, session: &mut Session) -> Result<()> {
        let mut resp = ResponseHeader::build(self.status, Some(self.body.len()))?;

        for (name, value) in self.headers {
            resp.insert_header(name, value)?;
        }

        session.write_response_header(Box::new(resp), false).await?;

        session.write_response_body(Some(self.body), true).await?;

        Ok(())
    }
}
