//!
//! IPP request
//!
use std::io::{self, Read, Write};

use attribute::{IppAttribute, IppAttributeList};
use ::{Result, IPP_VERSION, IppHeader};
use consts::tag::*;
use consts::attribute::*;
use value::IppValue;

/// IPP request struct
pub struct IppRequest<'a> {
    /// Operation ID
    operation: u16,
    /// IPP server URI
    uri: String,
    /// IPP attributes
    attributes: IppAttributeList,
    /// Optional payload to send after IPP-encoded stream (for example Print-Job operation)
    payload: Option<&'a mut Read>
}

impl<'a> IppRequest<'a> {
    /// Create new IPP request for the operation and uri
    pub fn new(operation: u16, uri: &str) -> IppRequest<'a> {
        let mut retval = IppRequest {
            operation: operation,
            uri: uri.to_string(),
            attributes: IppAttributeList::new(),
            payload: None };

        retval.set_attribute(
            OPERATION_ATTRIBUTES_TAG,
            IppAttribute::new(ATTRIBUTES_CHARSET,
                              IppValue::Charset("utf-8".to_string())));
        retval.set_attribute(
            OPERATION_ATTRIBUTES_TAG,
            IppAttribute::new(ATTRIBUTES_NATURAL_LANGUAGE,
                              IppValue::NaturalLanguage("en".to_string())));

        retval.set_attribute(
            OPERATION_ATTRIBUTES_TAG,
            IppAttribute::new(PRINTER_URI,
                              IppValue::Uri(uri.replace("http", "ipp").to_string())));
        retval
    }

    /// Get uri
    pub fn uri(&self) -> &String {
        &self.uri
    }

    /// Set payload
    pub fn set_payload(&mut self, payload: &'a mut Read) {
        self.payload = Some(payload)
    }

    /// Set attribute
    pub fn set_attribute(&mut self, group: u8, attribute: IppAttribute) {
        self.attributes.add(group, attribute);
    }

    /// Serialize request into the binary stream (TCP)
    pub fn write(&'a mut self, writer: &mut Write) -> Result<usize> {
        let hdr = IppHeader::new(IPP_VERSION, self.operation, 1);
        let mut retval = hdr.write(writer)?;

        retval += self.attributes.write(writer)?;

        debug!("Wrote {} bytes IPP stream", retval);

        if let Some(ref mut payload) = self.payload {
            let size = io::copy(payload, writer)? as usize;
            debug!("Wrote {} bytes payload", size);
            retval += size;
        }

        Ok(retval)
    }
}