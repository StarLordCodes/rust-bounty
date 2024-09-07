use std::{collections::HashMap, fmt::Display, str::FromStr};

use super::response::HttpResponse;

#[derive(Debug)]
pub struct HttpRequest {
    method: Method,
    pub resource: Resource,
    version: Version,
    headers: HttpHeader,
    pub request_body: String,
}
impl HttpRequest {
    pub fn response(&self) -> std::io::Result<HttpResponse> {
        HttpResponse::new(self)
    }
    pub fn new(request: &str) -> std::io::Result<HttpRequest> {
        let method = Method::new(request);
        let resource = if let Some(resource) = Resource::new(request) {
            resource
        } else {
            Resource {
                path: "".to_string(),
            }
        };

        let version: Version = Version::new(request)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.msg))?;

        let headers = if let Some(headers) = HttpHeader::new(request) {
            headers
        } else {
            HttpHeader {
                headers: HashMap::new(),
            }
        };
        let request_body = if let Some((_header, body)) = request.split_once("\r\n\r\n") {
            body.to_string()
        } else {
            "".to_string()
        };

        Ok(HttpRequest {
            method,
            resource,
            version,
            headers,
            request_body,
        })
    }
}
#[derive(Debug)]
pub enum Version {
    V1_1,
    V2_0,
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Version::V1_1 => "HTTP/1.1",
            Version::V2_0 => "HTTP/2",
        };
        write!(f, "{}", msg)
    }
}

#[derive(Debug)]
pub struct VersionError {
    msg: String,
}

impl Display for VersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(request: &str) -> std::result::Result<Self, Self::Err> {
        let request_split = request.split_once("\r\n");
        if let Some((method_line, _rest)) = request_split {
            let method_line = method_line.split_ascii_whitespace();
            for line in method_line {
                if line == "HTTP/1.1" {
                    return Ok(Version::V1_1);
                } else if line == "HTTP/2" || line == "HTTP/2.0" {
                    return Ok(Version::V2_0);
                }
            }
        };
        let invalid = format! {"Unknown protocol version in {}", request};
        let version_error = VersionError { msg: invalid };
        Err(version_error)
    }
}

impl Version {
    pub fn new(request: &str) -> std::result::Result<Self, VersionError> {
        Version::from_str(request)
    }
}

#[derive(Debug)]
enum Method {
    Get,
    Post,
    Uninitialized,
}

impl Method {
    pub fn new(request: &str) -> Method {
        let request_split = request.split_once("\r\n");
        if let Some((method_line, _rest)) = request_split {
            let method_line = method_line.split_once(" ");
            if let Some((method, _rest)) = method_line {
                return match method {
                    "GET" => Method::Get,
                    "POST" => Method::Post,
                    _ => Method::Uninitialized,
                };
            }
        };
        Method::Uninitialized
    }
    pub fn identify(s: &str) -> Method {
        match s {
            "GET" => Method::Get,
            "POST" => Method::Post,
            _ => Method::Uninitialized,
        }
    }
}

#[derive(Debug)]
pub struct Resource {
    pub path: String,
}
impl Resource {
    pub fn new(request: &str) -> Option<Resource> {
        if let Some((request_method, _)) = request.split_once("\r\n") {
            let (method, rest) = request_method.split_once(' ')?;
            return match Method::identify(method) {
                Method::Get | Method::Post => {
                    let (resource, _protocol_version) = rest.split_once(' ')?;
                    let resource = resource.trim();
                    let resource = resource.trim_start_matches('/');
                    return Some(Resource {
                        path: resource.to_string(),
                    });
                }
                Method::Uninitialized => None,
            };
        };
        None
    }
}

#[derive(Debug)]
struct HttpHeader {
    headers: HashMap<String, String>,
}

impl HttpHeader {
    pub fn new(request: &str) -> Option<HttpHeader> {
        let mut httpheader = HttpHeader {
            headers: HashMap::new(),
        };
        let (_, header_str) = request.split_once("\r\n")?;
        for line in header_str.split_terminator("\r\n") {
            if line.is_empty() {
                break;
            }
            let (header, value) = line.split_once(":")?;
            httpheader
                .headers
                .insert(header.trim().to_string(), value.trim().to_string());
        }

        Some(httpheader)
    }
}
