pub mod actix_web_request_wrapper;
pub mod request_wrapper;
pub mod response_wrapper;

pub type ActixWebRequestWrapper = crate::reqwest_bridge::actix_web_request_wrapper::ActixWebRequestWrapper;
pub type RequestWrapper = crate::reqwest_bridge::request_wrapper::RequestWrapper;
pub type ResponseWrapper = crate::reqwest_bridge::response_wrapper::ResponseWrapper;