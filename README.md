# actix_bridges

provide libs to convert from/to actix_web `HttpRequest` or `HttpResponse`

# Examples

## reqwest_bridge 

**convert actix_web `HttpRequest` to reqwest `Request`**
```rust
    let url = Url::parse("https://echo.free.beeceptor.com").unwrap();
    let mut request_wrapper: RequestWrapper = RequestWrapper::from(ActixWebRequestWrapper::new(req, bytes));
    log::info!("reqwest_req: {:#?}", request_wrapper);
    request_wrapper.change_url(url);
    let RequestWrapper { request, .. } = request_wrapper;
    log::info!("reqwest_req after set: {:#?}", request);
```

**convert reqwest `Response` to actix_web `HttpResponse`**
```rust
    let ret: HttpResponse<BoxBody> = ResponseWrapper::new(response).into().await;
```