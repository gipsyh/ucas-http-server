use std::collections::HashMap;

#[derive(Debug)]
pub enum Method {
    Get,
    Post,
}

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Request {
    pub fn parse(raw: Vec<u8>) -> Self {
        let headers = String::from_utf8(raw).unwrap();
        let headers = headers.trim_matches(char::from(0));
        let mut headers: Vec<&str> = headers
            .split(&['\n', '\r'])
            .filter(|str| !str.is_empty())
            .collect();
        let t: Vec<&str> = headers[0].split(' ').collect();
        let method = match t[0] {
            "GET" => Method::Get,
            "POST" => Method::Post,
            _ => {
                panic!()
            }
        };
        let path = t[1].to_string();
        headers.remove(0);
        let mut ans = Self {
            method,
            path,
            headers: HashMap::new(),
            body: Vec::new(),
        };
        for header in headers {
            let param: Vec<&str> = header.split(": ").collect();
            if param.len() != 2 {
                dbg!(param);
                panic!();
            }
            ans.headers
                .insert(param[0].to_string(), param[1].to_string());
        }
        dbg!(&ans);
        ans
    }
}
