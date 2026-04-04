pub mod flate;
pub mod ascii85;
pub mod asciihex;
pub mod lzw;
pub mod runlength;

#[cfg(feature = "c-codecs")]
pub mod dct;
#[cfg(feature = "c-codecs")]
pub mod jpx;

use crate::error::{BotlError, Result};
use crate::parser::objects::{PdfDict, PdfObject, PdfStream};

/// Decode stream data using the filter(s) specified in the stream dictionary.
pub fn decode_stream_data(stream: &PdfStream) -> Result<Vec<u8>> {
    let filter = match stream.filter() {
        Some(f) => f,
        None => return Ok(stream.data.clone()),
    };

    let mut data = stream.data.clone();

    match filter {
        PdfObject::Name(name) => {
            let filter_str = std::str::from_utf8(name)
                .map_err(|_| BotlError::CodecError("Invalid filter name encoding".into()))?;
            let params = stream.decode_parms().and_then(|p| p.as_dict());
            data = apply_filter(filter_str, &data, params)?;
        }
        PdfObject::Array(filters) => {
            // Chained filters: apply in order
            let params_array = stream.decode_parms().and_then(|p| p.as_array());

            for (i, f) in filters.iter().enumerate() {
                let filter_str = f
                    .as_name_str()
                    .ok_or_else(|| BotlError::CodecError("Filter name not a string".into()))?;
                let params = params_array
                    .and_then(|arr| arr.get(i))
                    .and_then(|p| p.as_dict());
                data = apply_filter(filter_str, &data, params)?;
            }
        }
        _ => return Err(BotlError::CodecError("Invalid Filter type".into())),
    }

    Ok(data)
}

fn apply_filter(filter: &str, data: &[u8], params: Option<&PdfDict>) -> Result<Vec<u8>> {
    match filter {
        "FlateDecode" => flate::decompress(data),
        "ASCII85Decode" => ascii85::decode(data),
        "ASCIIHexDecode" => asciihex::decode(data),
        "LZWDecode" => lzw::decode(data, params),
        "RunLengthDecode" => runlength::decode(data),
        "DCTDecode" => {
            #[cfg(feature = "c-codecs")]
            {
                dct::decompress(data)
            }
            #[cfg(not(feature = "c-codecs"))]
            {
                Err(BotlError::UnsupportedFeature(
                    "DCTDecode (JPEG) — requires c-codecs feature".into(),
                ))
            }
        }
        "JPXDecode" => {
            #[cfg(feature = "c-codecs")]
            {
                jpx::decompress(data)
            }
            #[cfg(not(feature = "c-codecs"))]
            {
                Err(BotlError::UnsupportedFeature(
                    "JPXDecode (JPEG2000) — requires c-codecs feature".into(),
                ))
            }
        }
        "CCITTFaxDecode" => Err(BotlError::UnsupportedFeature(
            "CCITTFaxDecode — not yet implemented".into(),
        )),
        "JBIG2Decode" => Err(BotlError::UnsupportedFeature(
            "JBIG2Decode — not yet implemented".into(),
        )),
        _ => Err(BotlError::CodecError(format!(
            "Unknown filter: {}",
            filter
        ))),
    }
}
