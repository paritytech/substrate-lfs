use base64;
use codec::{Decode, Encode};

/// helper to decode a base64 encoded string to Lfs
pub fn b64decode<'a, D: Decode>(input: &'a [u8]) -> Option<D> {
	base64::decode_config(input, base64::URL_SAFE)
		.ok()
		.and_then(|input| D::decode(&mut input.as_ref()).ok())
}

/// helper to encode to a base64
pub fn b64encode<'a, E: Encode>(input: E) -> String {
	base64::encode_config(&input.encode(), base64::URL_SAFE)
}
