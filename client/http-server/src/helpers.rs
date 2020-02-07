use base64;
use codec::Decode;

/// helper to decode a base64 encoded string to Lfs
pub fn b64decode<'a, D: Decode>(id: &'a str) -> Option<D> {
	base64::decode_config(id, base64::URL_SAFE)
		.ok()
		.map(|id| D::decode(&mut id.as_ref()).ok())
		.flatten()
}
