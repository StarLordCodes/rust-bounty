use std::io::Read;
use std::{fmt::Display, io};

use url_escape::{decode, encode, NON_ALPHANUMERIC};

use super::request::{HttpRequest, Version};

#[derive(Debug)]
pub struct HttpResponse {
    version: Version,
    status: ResponseStatus,
    content_length: usize,
    accept_ranges: AcceptRanges,
    pub response_body: Vec<u8>,
    pub current_path: String,
    pub content_type: String,
}

impl HttpResponse {
    pub fn new(request: &HttpRequest) -> io::Result<HttpResponse> {
        let version = Version::V1_1;
        let mut status = ResponseStatus::NotFound;
        let mut content_length = 0;
        let mut content_type = "text/html".to_string();
        let mut accept_ranges = AcceptRanges::None;
        let resource_path = request.resource.path.clone();
        let current_path = decode(&resource_path).into_owned();
        let trimmed_path = current_path.trim_start_matches('/');
        let rootcwd = std::env::current_dir()?;
        let rootcwd_canonical = rootcwd.canonicalize()?;
        let new_path = rootcwd.join(trimmed_path);
        let new_path_canonical = new_path.canonicalize()?;

        // Calculate depth
        let rootcwd_len = rootcwd_canonical.components().count();
        let new_path_len = new_path_canonical.components().count();

        let mut response_body = Vec::new();

        if new_path.exists() {
            if new_path.is_file() {
                let file_type_result = infer::get_from_path(&new_path)?;
                if let Some(file_type) = file_type_result {
                    content_type = file_type.mime_type().to_string();
                } else {
                    content_type = "text/plain".to_string();
                }

                // Read file as binary
                let mut file = std::fs::File::open(&new_path)?;
                let mut content = Vec::new();
                file.read_to_end(&mut content)?;
                content_length = content.len();
                status = ResponseStatus::OK;
                accept_ranges = AcceptRanges::Bytes;

                // Create the response header
                let header = format!(
                    "{} {}\n{}\ncontent-type: {}\ncontent-length: {}\r\n\r\n",
                    version, status, accept_ranges, content_type, content_length
                );
                response_body.extend_from_slice(header.as_bytes());
                response_body.extend_from_slice(&content);
            } else if new_path.is_dir() {
                status = ResponseStatus::OK;
                accept_ranges = AcceptRanges::None;

                let mut listing = Vec::new();
                listing.extend_from_slice(b"<html><head><meta charset=\"utf-8\"/></head><body>");

                // Display the current directory
                let current_dir_display = to_unix_style(new_path.to_str().unwrap_or(""));
                listing.extend_from_slice(b"<h1>Directory Listing</h1>");
                listing.extend_from_slice(b"<p>Current directory: ");
                listing.extend_from_slice(current_dir_display.as_bytes());
                listing.extend_from_slice(b"</p>");

                // Option to go up one directory shown only if not at root directory
                if rootcwd_canonical != new_path_canonical {
                    let parent_path = new_path.parent().unwrap_or(&rootcwd).to_path_buf();
                    let parent_path_str = parent_path.to_str().unwrap_or("");
                    let parent_encoded = encode(parent_path_str, NON_ALPHANUMERIC).into_owned();
                    listing.extend_from_slice(b"<p><a href=\"");
                    listing.extend_from_slice(parent_encoded.as_bytes());
                    listing.extend_from_slice(b"\">Up One Level</a></p>");
                }

                listing.extend_from_slice(b"<ul>");
                for entry in std::fs::read_dir(&new_path)? {
                    let entry = entry?;
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_str().expect("invalid unicode");
                    let full_path = format!("{}/{}", current_path, file_name_str);
                    let file_name_bytes = file_name.as_encoded_bytes();
                    let encoded_path = encode(&full_path, NON_ALPHANUMERIC).into_owned();

                    listing.extend_from_slice(b"<li><a href=\"");
                    listing.extend_from_slice(encoded_path.as_bytes());
                    listing.extend_from_slice(b"\">");
                    listing.extend_from_slice(&file_name_bytes);
                    listing.extend_from_slice(b"</a></li>");
                }

                listing.extend_from_slice(b"</ul></body></html>");
                content_length = listing.len();
                response_body.extend_from_slice(
                    format!(
                        "{} {}\n{}\ncontent-type: {}\ncontent-length: {}\r\n\r\n",
                        version, status, accept_ranges, content_type, content_length
                    )
                    .as_bytes(),
                );
                response_body.extend_from_slice(&listing);
            } else {
                // Handle 404 not found
                let four_o_four = "
                <html>
                <body>
                <h1>404 NOT FOUND</h1>
                </body>
                </html>";
                content_length = four_o_four.len();
                let content = format!(
                    "{} {}\n{}\ncontent-type: {}\ncontent-length: {}\r\n\r\n{}",
                    version, status, accept_ranges, content_type, content_length, four_o_four
                );
                response_body.extend_from_slice(content.as_bytes());
            }
        }

        Ok(HttpResponse {
            version,
            status,
            content_length,
            content_type,
            accept_ranges,
            response_body,
            current_path,
        })
    }
}
#[derive(Debug)]
enum ResponseStatus {
    OK = 200,
    NotFound = 404,
}

impl Display for ResponseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ResponseStatus::OK => "200 OK",
            ResponseStatus::NotFound => "404 NOT FOUND",
        };
        write!(f, "{}", msg)
    }
}

#[derive(Debug)]
enum AcceptRanges {
    Bytes,
    None,
}

impl Display for AcceptRanges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            AcceptRanges::Bytes => "accept-ranges: bytes",
            AcceptRanges::None => "accept-ranges: none",
        };
        write!(f, "{}", msg)
    }
}

// Function to convert path to Unix-style
fn to_unix_style(path: &str) -> String {
    path.replace("\\", "/")
}
