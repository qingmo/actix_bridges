use std::vec;

use actix_web::{HttpResponse, body::BoxBody, http::StatusCode};
use futures::{io::ErrorKind, prelude::*};
use reqwest::header::HeaderMap;

#[derive(Debug)]
pub struct ResponseWrapper {
    pub response: Result<reqwest::Response, reqwest::Error>,
}

impl ResponseWrapper {
    pub fn new(response: Result<reqwest::Response, reqwest::Error>) -> Self {
        Self { response }
    }

    async fn handle_reqwest_response(response: reqwest::Response) -> HttpResponse {
        let headers = response.headers().clone();
        let status = response.status();
        log::trace!("headers: {:#?}", headers);
        if !status.is_success() {
            log::trace!("status: {}", status);
            return HttpResponse::new(StatusCode::from_u16(status.as_u16()).unwrap());
        }
        Self::into_actix_response(headers, status, response).await
    }

    async fn into_actix_response(
        headers: HeaderMap,
        status: reqwest::StatusCode,
        response: reqwest::Response,
    ) -> HttpResponse {
        let headers = Box::new(headers);
        let mut reader = response
            .bytes_stream()
            .map_err(|e| std::io::Error::new(ErrorKind::Other, e))
            .into_async_read();
        let mut output = vec![];
        let _bytes = reader.read_to_end(&mut output).await.unwrap();
        let mut ret = HttpResponse::new(StatusCode::from_u16(status.as_u16()).unwrap())
            .set_body(BoxBody::new(output));
        let response_headers = ret.headers_mut();
        headers.iter().for_each(|(key, value)| {
            response_headers.insert(
                key.as_str().parse().unwrap(),
                value.to_str().unwrap().parse().unwrap(),
            );
        });
        ret
    }

    pub async fn into(self) -> HttpResponse {
        match self.response {
            Err(err) => {
                log::error!("request error: {}", err);
                return HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR);
            }
            Ok(response) => Self::handle_reqwest_response(response).await,
        }
    }
}

#[cfg(test)]
mod tests {

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
        assert_eq!(headers.get("X-Custom-Foo").unwrap(), "bar");
        assert_eq!(headers.get("content-length").unwrap(), "0");
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
