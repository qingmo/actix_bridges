use actix_web::{HttpResponse, body::BoxBody, http::StatusCode};

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
            if response.status().is_success() {
                HttpResponse::new(StatusCode::from_u16(response.status().as_u16()).unwrap())
                    .set_body(BoxBody::new(response.text().await.unwrap()))
            } else {
                HttpResponse::new(StatusCode::from_u16(response.status().as_u16()).unwrap())
            }
        } else {
            log::error!("response failed: {}", self.response.err().unwrap());
            HttpResponse::InternalServerError().body("error")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        borrow::Borrow,
        error,
        fmt::Result,
        io::{self, ErrorKind},
        rc::Rc,
    };

    use actix_web::{
        HttpResponse,
        body::{MessageBody, to_bytes},
        http::StatusCode,
        test::TestRequest,
    };
    use http::response::Builder;
    use reqwest::{Error, RequestBuilder, Response};
    use url::Url;

    use super::*;

    #[actix_web::test]
    async fn test_into_success() {
        let response = Builder::new().status(200).body("foo").unwrap();
        let response = Response::from(response);
        let wrapper = ResponseWrapper::new(Ok(response));

        let http_response: HttpResponse<BoxBody> = wrapper.into().await;
        let (http_response, body) = http_response.into_parts();
        assert_eq!(http_response.status(), StatusCode::OK);
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
