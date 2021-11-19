pub struct ResponseBuilder {
    status: String,
    content: Vec<u8>,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        Self {
            status: "".to_string(),
            content: Vec::new(),
        }
    }

    pub fn set_status(&mut self, status: String) -> &mut Self {
        self.status = status;
        self
    }

    pub fn set_content(&mut self, content: Vec<u8>) -> &mut Self {
        self.content = content;
        self
    }

    pub fn build(&self) -> Response {
        Response {
            status: self.status.clone(),
            content: self.content.clone(),
        }
    }
}

pub struct Response {
    status: String,
    content: Vec<u8>,
}

impl Response {
    pub fn into_vec(self) -> Vec<u8> {
        let mut ans = vec![];
        let response = format!(
            "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n",
            self.status,
            self.content.len(),
        );
        ans.extend_from_slice(response.as_bytes());
        ans.extend_from_slice(&self.content);
        ans
    }
}
