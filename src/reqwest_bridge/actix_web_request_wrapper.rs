use actix_web::{web::Bytes, HttpRequest};

#[derive(Debug)]
pub struct ActixWebRequestWrapper {
    pub req: HttpRequest,
    pub body: Bytes,
}
impl ActixWebRequestWrapper {
    pub fn new(req: HttpRequest, body: Bytes) -> Self {
        Self { req, body }
    }
}

#[cfg(test)]
mod tests {
    use actix_web::{http::{header::{HeaderValue}, Method}, test::TestRequest};
    use pretty_assertions::assert_eq;
    use super::*;

    #[test]
    fn test_new() {
        let head_val = HeaderValue::from_static("application/json");
        let req = TestRequest::with_uri("/").append_header(("Content-Type", "application/json")).to_http_request();
        let body = Bytes::from_static(b"test");
        let wrapper = ActixWebRequestWrapper::new(req, body.clone());
        assert_eq!(wrapper.req.method(), Method::GET);
        assert_eq!(wrapper.req.uri().path(), "/");
        assert_eq!(wrapper.body, body);
        assert_eq!(wrapper.req.headers().get("Content-Type"), Some(head_val).as_ref());
    }
}