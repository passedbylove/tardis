use crate::basic::error::TardisError;
use crate::basic::result::TardisResult;
pub struct TardisCryptoBase64;

impl TardisCryptoBase64 {
    pub fn decode(&self, data: &str) -> TardisResult<String> {
        match base64::decode(data) {
            Ok(result) => Ok(String::from_utf8(result)?),
            Err(e) => Err(TardisError::format_error(
                &format!("[Tardis.Crypto] Base64 decode error:{}", e),
                "406-tardis-crypto-base64-decode-error",
            )),
        }
    }

    pub fn encode(&self, data: &str) -> String {
        base64::encode(data)
    }
}
