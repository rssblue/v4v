/// Keysend address.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct KeysendAddress {
    /// Node's public key.
    pub pubkey: String,
    /// Custom data (key -> value), usually used to identify a wallet hosted at a node.
    #[serde(default)]
    pub custom_data: Option<(String, String)>,
}
