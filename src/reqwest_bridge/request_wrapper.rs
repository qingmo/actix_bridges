use reqwest::{header::{HeaderName, HeaderValue}, Body, Method, Request};

use crate::reqwest_bridge::ActixWebRequestWrapper;
#[derive(Debug)]
pub struct RequestWrapper {
    pub request: reqwest::Request,
}

impl RequestWrapper {
    pub fn change_url(&mut self, url: reqwest::Url) {
        self.request.url_mut().set_scheme(url.scheme()).unwrap();
        self.request.url_mut().set_host(url.host_str()).unwrap();
        self.request.url_mut().set_port(url.port()).unwrap();
        self.request.headers_mut().insert("host", url.host_str().unwrap().parse().unwrap());
    }
}

impl From<ActixWebRequestWrapper> for RequestWrapper {
    fn from(value: ActixWebRequestWrapper) -> Self {
        let parsed_method = Method::from_bytes(value.req.method().as_str().as_bytes()).unwrap();
        let mut request = Request::new(parsed_method, value.req.full_url());
        let mut_headers = request.headers_mut();
        for (name, value) in value.req.headers() {
            let header_name = HeaderName::try_from(name.as_str()).unwrap();
            let header_value: HeaderValue = value.to_str().unwrap().parse().unwrap();
            mut_headers.insert(header_name, header_value);
        }
        let _ = request.body_mut().insert(Body::from(value.body));
        Self { request }
    }
}

#[cfg(test)]
mod tests {
    use actix_web::test::TestRequest;
    use actix_web::web::Bytes;
    use reqwest::{Method, Url};
    use reqwest::header::{HeaderValue};
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_change_url() {
        let mut wrapper = RequestWrapper {
            request: Request::new(Method::GET, Url::parse("http://example.com:8080").unwrap()),
        };

        assert_eq!(wrapper.request.url().port(), Some(8080));
        let new_url = Url::parse("https://new-example.com").unwrap();
        wrapper.change_url(new_url);

        assert_eq!(wrapper.request.url().scheme(), "https");
        assert_eq!(wrapper.request.url().host_str(), Some("new-example.com"));
        assert_eq!(wrapper.request.url().port(), None);
    }

    #[test]
    fn test_from_actix_web_request_wrapper_with_headers() {
        let head_val = HeaderValue::from_static("application/json");
        let req = TestRequest::with_uri("/").method(actix_web::http::Method::POST).append_header(("Content-Type", "application/json")).to_http_request();
        let body = Bytes::from_static(b"test");
        let actix_wrapper = ActixWebRequestWrapper::new(req, body.clone());
        
        let request_wrapper: RequestWrapper = actix_wrapper.into();

        assert_eq!(request_wrapper.request.method(), Method::POST);
        assert_eq!(request_wrapper.request.url().path(), "/");
        let new_body = request_wrapper.request.body().unwrap().as_bytes().unwrap();
        assert_eq!(new_body, body);
        assert_eq!(request_wrapper.request.headers().get("Content-Type"), Some(head_val).as_ref());
    }
}