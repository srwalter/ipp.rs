//!
//! IPP client
//!
use hyper::client::request::Request;
use hyper::method::Method;
use hyper::Url;
use hyper::status::StatusCode;
use std::io::{BufWriter, BufReader};

use request::IppRequest;
use response::IppResponse;
use ::{IppError, Result};
use attribute::{IppAttributeList};
use parser::IppParser;

/// IPP client.
///
/// IPP client is responsible for sending requests to IPP server.
pub struct IppClient {}

impl IppClient {
    /// Create new instance of the client
    pub fn new() -> IppClient {
        IppClient {}
    }

    /// Send request and return response
    pub fn send_raw<'a>(&self, request: &'a mut IppRequest<'a>) -> Result<IppResponse> {
        match Url::parse(request.uri()) {
            Ok(url) => {
                // create request and set headers
                let mut http_req_fresh = Request::new(Method::Post, url)?;
                http_req_fresh.headers_mut().set_raw("Content-Type", vec![b"application/ipp".to_vec()]);

                // connect and send headers
                let mut http_req_stream = http_req_fresh.start()?;

                // send IPP request using buffered writer.
                // NOTE: unbuffered output will cause issues on many IPP implementations including CUPS
                request.write(&mut BufWriter::new(&mut http_req_stream))?;

                // get the response
                let http_resp = http_req_stream.send()?;

                if http_resp.status == StatusCode::Ok {
                    // HTTP 200 assumes we have IPP response to parse
                    let mut reader = BufReader::new(http_resp);
                    let mut parser = IppParser::new(&mut reader);
                    let resp = IppResponse::from_parser(&mut parser)?;

                    Ok(resp)
                } else {
                    error!("HTTP error: {}", http_resp.status);
                    Err(IppError::RequestError(
                        if let Some(reason) = http_resp.status.canonical_reason() {
                            reason.to_string()
                        } else {
                            format!("{}", http_resp.status)
                        }))
                }
            }
            Err(err) => {
                error!("Invalid URI: {}", request.uri());
                Err(IppError::RequestError(err.to_string()))
            }
        }
    }

    /// Send request and return list of attributes if it succeeds
    pub fn send<'a>(&self, request: &'a mut IppRequest<'a>) -> Result<IppAttributeList> {
        match self.send_raw(request) {
            Ok(resp) => {
                if resp.header().status > 3 {
                    // IPP error
                    Err(IppError::StatusError(resp.header().status))
                } else {
                    Ok(resp.attributes().clone())
                }
            }
            Err(err) => Err(err)
        }
    }
}