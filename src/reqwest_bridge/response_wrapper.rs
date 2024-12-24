use actix_web::{
    HttpResponse,
    body::BoxBody,
    http::{
        StatusCode,
    },
};
use async_compression::futures::bufread::GzipDecoder;
use futures::{
    io::{ErrorKind},
    prelude::*,
};

#[derive(Debug)]
pub struct ResponseWrapper {
    pub response: Result<reqwest::Response, reqwest::Error>,
}

impl ResponseWrapper {
    pub fn new(response: Result<reqwest::Response, reqwest::Error>) -> Self {
        Self { response }
    }

    pub async fn into(self) -> HttpResponse {
        if let Ok(response) = self.response {
            let headers = response.headers().clone();
            let status = response.status();
            log::info!("headers: {:#?}", headers);
            if status.is_success() {
                // let text = response.text().await.unwrap();
                let content_type_opt = headers.get("content-type");
                let content_type = match content_type_opt {
                    Some(content_type) => content_type.to_str().unwrap_or("application/text"),
                    None => "application/text",
                };
                let content_encoding_opt = headers.get("content-encoding");
                let content_encoding = match content_encoding_opt {
                    Some(content_encoding) => content_encoding.to_str().unwrap_or(""),
                    None => "",
                };
                let mut reader = response
                    .bytes_stream()
                    .map_err(|e| std::io::Error::new(ErrorKind::Other, e))
                    .into_async_read();
                let mut data = String::new();
                if "gzip" == content_encoding || "application/gzip" == content_type {
                    let mut decoder = GzipDecoder::new(reader);
                    decoder.read_to_string(&mut data).await.unwrap();
                } else {
                    reader.read_to_string(&mut data).await.unwrap();
                }
                HttpResponse::new(StatusCode::from_u16(status.as_u16()).unwrap())
                        .set_body(BoxBody::new(data))
            } else {
                HttpResponse::new(StatusCode::from_u16(status.as_u16()).unwrap())
            }
        } else {
            log::error!("response failed: {}", self.response.err().unwrap());
            HttpResponse::InternalServerError().body("error")
        }
    }

    // async fn read_gzip_body(response: reqwest::Response) -> Result<String, reqwest::Error> {
    //     // let bytes = response.bytes().await?;
    //     let bytesx = response.bytes().await?.split();
    //     let mut decoder = GzDecoder::new(bytes);
    //     let mut s = String::new();

    //     decoder.read_exact(&mut bytes[..])?;
    //     s.push_str(&decoder.finish().unwrap());

    //     Ok(s)
    // }
}

#[cfg(test)]
mod tests {
    use std::io::{self, BufReader, ErrorKind};

    use actix_web::{HttpResponse, body::MessageBody, http::StatusCode};
    use http::response::Builder;
    use pretty_assertions::assert_eq;
    use reqwest::Response;

    use super::*;

    #[actix_web::test]
    async fn test_into_success() {
        let response = Builder::new()
            .status(200)
            .header("Content-Type", "text/html")
            .header("X-Custom-Foo", "bar")
            .header("content-length", 0)
            .body("foo")
            .unwrap();
        let response = Response::from(response);
        let wrapper = ResponseWrapper::new(Ok(response));

        let http_response: HttpResponse<BoxBody> = wrapper.into().await;
        let (http_response, body) = http_response.into_parts();
        let headers = http_response.headers();
        assert_eq!(http_response.status(), StatusCode::OK);
        assert_eq!(headers.get("Content-Type").unwrap(), "text/html");
        // assert_eq!(headers.get("X-Custom-Foo").unwrap(), "bar");
        // assert_eq!(headers.get("content-length").unwrap(), "0");
        assert_eq!(body.try_into_bytes().unwrap(), "foo".as_bytes());
    }

    #[actix_web::test]
    async fn test_into_not_found() {
        let response = Builder::new().status(404).body("Not Found").unwrap();
        let response = Response::from(response);
        let wrapper = ResponseWrapper::new(Ok(response));

        let http_response = wrapper.into().await;
        let (http_response, body) = http_response.into_parts();
        assert_eq!(http_response.status(), StatusCode::NOT_FOUND);
        assert_eq!(body.try_into_bytes().unwrap(), "");
    }
}
