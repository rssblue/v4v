/// Action for Podcasting 2.0 payment.
#[derive(
    Debug, Default, serde::Deserialize, PartialEq, Clone, serde::Serialize, strum_macros::Display,
)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    /// One-time payment.
    #[default]
    Boost,
    /// Stream payment.
    Stream,
    /// Auto payment.
    Auto,
}

use std::collections::HashMap;

use chrono::Duration;
use serde_json::Value;
use url::Url;
use uuid::Uuid;

use crate::alby::{
    api::{
        payments::{MultiKeysendItemArgs, MultiKeysendResponse},
        RequestError,
    },
    types::KeysendAddress,
};

/// bLIP-10 TLV record coming from an untrusted source.
///
/// Apps may not conform to
/// <https://github.com/Podcastindex-org/podcast-namespace/blob/main/value/blip-0010.md#fields>
/// standard so will make it as generic as possible.
#[derive(Debug, serde::Deserialize)]
pub struct UntrustedTlvRecord {
    /// ACTION
    /// "boost", "stream" or "auto"
    action: Value,

    /// FEED IDENTIFIER
    ///
    ///  The `<podcast:guid>` tag.
    #[serde(rename = "guid", default)]
    feed_guid: Value,
    /// title of the feed
    #[serde(rename = "podcast", default)]
    feed_name: Value,
    /// ID of podcast in PodcastIndex.org directory
    #[serde(rename = "feedID", default)]
    feed_pi_id: Value,
    /// RSS feed URL of podcast
    #[serde(rename = "url", default)]
    feed_url: Value,

    /// ITEM IDENTIFIER
    ///
    /// The `<item:guid>` tag.
    #[serde(rename = "episode_guid", default)]
    item_guid: Value,
    /// title of the item
    #[serde(rename = "episode", default)]
    item_name: Value,
    /// ID of the item in PodcastIndex.org directory
    #[serde(rename = "itemID", default)]
    item_pi_id: Value,

    /// PLAYBACK INFO
    ///
    ///  Timestamp of when the payment was sent, in seconds, as an offset from zero (i.e. - playback position).
    #[serde(rename = "ts", default)]
    timestamp_seconds: Value,
    /// Timestamp of when the payment was sent, in HH:MM:SS notation, as an offset from 00:00:00 (i.e. - playback position).
    #[serde(rename = "time", default)]
    timestamp_hhmmss: Value,
    /// Speed in which the podcast was playing, in decimal notation at the time the payment was sent. So 0.5 is half speed and 2 is double speed.
    #[serde(default)]
    speed: Value,

    /// APP INFO
    ///
    /// Name of sending app
    #[serde(default)]
    app_name: Value,
    /// Version of sending app
    #[serde(default)]
    app_version: Value,

    /// SENDER INFO
    ///
    /// Name of the sender (free text, not validated in any way)
    #[serde(default)]
    sender_name: Value,
    /// Static random identifier for users, not displayed by apps to prevent abuse. Apps can set this per-feed or app-wide. A GUID-like random identifier or a hash works well. Max 32 bytes (64 ascii characters). This can be a Nostr hex encoded pubkey (not NIP-19) for purposes of sender attribution.
    #[serde(default)]
    sender_id: Value,

    /// RECEIVER INFO
    ///
    /// Name for this split in value tag
    #[serde(rename = "name", default)]
    receiver_name: Value,

    /// PAYMENT INFO
    ///
    /// Total number of millisats for the payment before any fees are subtracted. This should be the number the listener entered into the app. Preserving this value is important for numerology reasons. Certain numeric values can have significance to the sender and/or receiver, so giving a way to show this is critical.
    #[serde(default)]
    value_msat_total: Value,
    /// Text message to add to the payment. When this field is present, the payment is known as a "boostagram".
    #[serde(default)]
    message: Value,
    /// App-specific URL containing route to podcast, episode, and/or timestamp at time of the action. The use case for this is sending a link along with the payment that will take the recipient to the exact playback position within the episode where the payment was sent.
    #[serde(default)]
    boost_link: Value,
    /// Optionally, this field can contain a signature for the payment, to be able to verify that the user who sent it is actually who they claim in the sender_id field. If the sender_id contains a Nostr public key, this field should contain a Nostr sig value as a 64-byte encoded hex string. For the purpose of generating the Nostr signature, the following data should be serialized: [0,sender_id,ts,1,[],message] to conform to the NIP-01 specification. The resulting serialized string should be hashed with sha256 to obtain the value.
    #[serde(rename = "signature", default)]
    payment_signature: Value,
    /// UUID of a payment sent out to a single recipient.
    #[serde(rename = "uuid", default)]
    payment_id: Value,
    /// UUID for the boost/stream/auto payment. If there are several recipients, the same identifier should be sent to all of them.
    #[serde(rename = "boost_uuid", default)]
    boost_id: Value,

    /// REMOTE INFO
    ///
    /// Sometimes a payment will be sent to a feed's value block because a different feed referenced it in a <podcast:valueTimeSplit> tag. When that happens, this field will contain the guid of the referencing feed.
    #[serde(default)]
    remote_feed_guid: Value,
    /// The Split Kit does weird stuff.
    #[serde(default, rename = "remoteFeedGuid")]
    remote_feed_guid_camelcase: Value,
    /// Sometimes a payment will be sent to an episode's value block because a different feed referenced it in a <podcast:valueTimeSplit> tag. When that happens, this field will contain the guid of the referencing feed's <item>.
    #[serde(default)]
    remote_item_guid: Value,
    /// The Split Kit does weird stuff.
    #[serde(default, rename = "remoteItemGuid")]
    remote_item_guid_camelcase: Value,

    /// REPLY INFO
    ///
    /// The pubkey of the lightning node that can receive payments for the sender. The node given must be capable of receiving keysend payments. If this field contains an "@" symbol, it should be interpreted as a "lightning address".
    #[serde(default)]
    reply_address: Value,
    /// The custom key for routing a reply payment to the sender. This field should not be present if it is not required for payment routing.
    #[serde(default)]
    reply_custom_key: Value,
    /// The custom value for routing a reply payment to the sender. This field should not be present if it is not required for payment routing.
    #[serde(default)]
    reply_custom_value: Value,
}

/// Well-formed bLIP-10 TLV record.
#[derive(Debug, serde::Serialize)]
pub struct TlvRecord {
    /// ACTION
    action: Action,

    /// FEED IDENTIFIER
    ///
    ///  The `<podcast:guid>` tag.
    #[serde(rename = "guid", skip_serializing_if = "Option::is_none")]
    feed_guid: Option<Uuid>,
    /// title of the feed
    #[serde(rename = "podcast", skip_serializing_if = "Option::is_none")]
    feed_name: Option<String>,
    /// ID of podcast in PodcastIndex.org directory
    #[serde(rename = "feedID", skip_serializing_if = "Option::is_none")]
    feed_pi_id: Option<u64>,
    /// RSS feed URL of podcast
    #[serde(rename = "url", skip_serializing_if = "Option::is_none")]
    feed_url: Option<Url>,

    /// ITEM IDENTIFIER
    ///
    /// The `<item:guid>` tag.
    #[serde(rename = "episode_guid", skip_serializing_if = "Option::is_none")]
    item_guid: Option<String>,
    /// title of the item
    #[serde(rename = "episode", skip_serializing_if = "Option::is_none")]
    item_name: Option<String>,
    /// ID of the item in PodcastIndex.org directory
    #[serde(rename = "itemID", skip_serializing_if = "Option::is_none")]
    item_pi_id: Option<u64>,

    /// PLAYBACK INFO
    ///
    ///  Timestamp of when the payment was sent, in seconds, as an offset from zero (i.e. - playback position).
    #[serde(rename = "ts", skip_serializing_if = "Option::is_none")]
    timestamp_seconds: Option<f64>,
    /// Timestamp of when the payment was sent, in HH:MM:SS notation, as an offset from 00:00:00 (i.e. - playback position).
    #[serde(rename = "time", skip_serializing_if = "Option::is_none")]
    timestamp_hhmmss: Option<String>,
    /// Speed in which the podcast was playing, in decimal notation at the time the payment was sent. So 0.5 is half speed and 2 is double speed.
    #[serde(skip_serializing_if = "Option::is_none")]
    speed: Option<f64>,

    /// APP INFO
    ///
    /// Name of sending app
    #[serde(skip_serializing_if = "Option::is_none")]
    app_name: Option<String>,
    /// Version of sending app
    #[serde(skip_serializing_if = "Option::is_none")]
    app_version: Option<String>,

    /// SENDER INFO
    ///
    /// Name of the sender (free text, not validated in any way)
    #[serde(skip_serializing_if = "Option::is_none")]
    sender_name: Option<String>,
    /// Static random identifier for users, not displayed by apps to prevent abuse. Apps can set this per-feed or app-wide. A GUID-like random identifier or a hash works well. Max 32 bytes (64 ascii characters). This can be a Nostr hex encoded pubkey (not NIP-19) for purposes of sender attribution.
    #[serde(skip_serializing_if = "Option::is_none")]
    sender_id: Option<String>,

    /// RECEIVER INFO
    ///
    /// Name for this split in value tag
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    receiver_name: Option<String>,

    /// PAYMENT INFO
    ///
    /// Total number of millisats for the payment before any fees are subtracted. This should be the number the listener entered into the app. Preserving this value is important for numerology reasons. Certain numeric values can have significance to the sender and/or receiver, so giving a way to show this is critical.
    #[serde(skip_serializing_if = "Option::is_none")]
    value_msat_total: Option<u64>,
    /// Text message to add to the payment. When this field is present, the payment is known as a "boostagram".
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    /// App-specific URL containing route to podcast, episode, and/or timestamp at time of the action. The use case for this is sending a link along with the payment that will take the recipient to the exact playback position within the episode where the payment was sent.
    #[serde(skip_serializing_if = "Option::is_none")]
    boost_link: Option<Url>,
    /// Optionally, this field can contain a signature for the payment, to be able to verify that the user who sent it is actually who they claim in the sender_id field. If the sender_id contains a Nostr public key, this field should contain a Nostr sig value as a 64-byte encoded hex string. For the purpose of generating the Nostr signature, the following data should be serialized: [0,sender_id,ts,1,[],message] to conform to the NIP-01 specification. The resulting serialized string should be hashed with sha256 to obtain the value.
    #[serde(rename = "signature", skip_serializing_if = "Option::is_none")]
    payment_signature: Option<String>,
    /// UUID of a payment sent out to a single recipient.
    #[serde(rename = "uuid", skip_serializing_if = "Option::is_none")]
    payment_id: Option<String>,
    /// UUID for the boost/stream/auto payment. If there are several recipients, the same identifier should be sent to all of them.
    #[serde(rename = "boost_uuid", skip_serializing_if = "Option::is_none")]
    boost_id: Option<String>,

    /// REMOTE INFO
    ///
    /// Sometimes a payment will be sent to a feed's value block because a different feed referenced it in a <podcast:valueTimeSplit> tag. When that happens, this field will contain the guid of the referencing feed.
    #[serde(skip_serializing_if = "Option::is_none")]
    remote_feed_guid: Option<Uuid>,
    /// Sometimes a payment will be sent to an episode's value block because a different feed referenced it in a <podcast:valueTimeSplit> tag. When that happens, this field will contain the guid of the referencing feed's <item>.
    #[serde(skip_serializing_if = "Option::is_none")]
    remote_item_guid: Option<String>,

    /// REPLY INFO
    ///
    /// The pubkey of the lightning node that can receive payments for the sender. The node given must be capable of receiving keysend payments. If this field contains an "@" symbol, it should be interpreted as a "lightning address".
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_address: Option<String>,
    /// The custom key for routing a reply payment to the sender. This field should not be present if it is not required for payment routing.
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_custom_key: Option<String>,
    /// The custom value for routing a reply payment to the sender. This field should not be present if it is not required for payment routing.
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_custom_value: Option<String>,
}

fn json_value_to_string(value: Value) -> Option<String> {
    match value {
        Value::String(string) => {
            let string = string.trim();
            if string.is_empty() {
                None
            } else {
                Some(string.to_string())
            }
        }
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

fn json_value_to_u64(value: Value) -> Option<u64> {
    match value {
        Value::String(string) => string.parse().ok(),
        Value::Number(number) => number.as_u64(),
        _ => None,
    }
}

fn json_value_to_url(value: Value) -> Option<Url> {
    match value {
        Value::String(string) => Url::parse(&string).ok(),
        _ => None,
    }
}

fn json_value_to_uuid(value: Value) -> Option<Uuid> {
    match value {
        Value::String(string) => Uuid::parse_str(&string).ok(),
        _ => None,
    }
}

fn json_value_to_f64(value: Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        _ => None,
    }
}

impl From<UntrustedTlvRecord> for TlvRecord {
    fn from(record: UntrustedTlvRecord) -> Self {
        Self {
            action: match record.action {
                Value::String(string) => match string.as_str() {
                    "boost" => Action::Boost,
                    "stream" => Action::Stream,
                    "auto" => Action::Auto,
                    _ => Action::Boost,
                },
                _ => Action::Boost,
            },
            feed_guid: json_value_to_uuid(record.feed_guid),
            feed_name: json_value_to_string(record.feed_name),
            feed_pi_id: json_value_to_u64(record.feed_pi_id),
            feed_url: json_value_to_url(record.feed_url),
            item_guid: json_value_to_string(record.item_guid),
            item_name: json_value_to_string(record.item_name),
            item_pi_id: json_value_to_u64(record.item_pi_id),
            timestamp_seconds: json_value_to_f64(record.timestamp_seconds),
            timestamp_hhmmss: json_value_to_string(record.timestamp_hhmmss),
            speed: json_value_to_f64(record.speed),
            app_name: json_value_to_string(record.app_name),
            app_version: json_value_to_string(record.app_version),
            sender_name: json_value_to_string(record.sender_name),
            sender_id: json_value_to_string(record.sender_id),
            receiver_name: json_value_to_string(record.receiver_name),
            value_msat_total: json_value_to_u64(record.value_msat_total),
            message: json_value_to_string(record.message),
            boost_link: json_value_to_url(record.boost_link),
            payment_signature: json_value_to_string(record.payment_signature),
            payment_id: json_value_to_string(record.payment_id),
            boost_id: json_value_to_string(record.boost_id),
            remote_feed_guid: match (
                json_value_to_uuid(record.remote_feed_guid),
                json_value_to_uuid(record.remote_feed_guid_camelcase),
            ) {
                (Some(guid), _) => Some(guid),
                (_, Some(guid)) => Some(guid),
                _ => None,
            },
            remote_item_guid: match (
                json_value_to_string(record.remote_item_guid),
                json_value_to_string(record.remote_item_guid_camelcase),
            ) {
                (Some(guid), _) => Some(guid),
                (_, Some(guid)) => Some(guid),
                _ => None,
            },
            reply_address: json_value_to_string(record.reply_address),
            reply_custom_key: json_value_to_string(record.reply_custom_key),
            reply_custom_value: json_value_to_string(record.reply_custom_value),
        }
    }
}

/// Recipient arguments for [keysend_payment].
#[derive(Debug)]
pub struct KeysendPaymentRecipientArgs {
    /// Recipient's keysend address.
    pub address: KeysendAddress,
    /// Number of sats to send.
    pub num_sats: u64,
    /// UUID of a payment sent out to a single recipient.
    pub payment_id: Option<String>,
    /// Optionally, this field can contain a signature for the payment, to be able to verify that the user who sent it is actually who they claim in the sender_id field. If the sender_id contains a Nostr public key, this field should contain a Nostr sig value as a 64-byte encoded hex string. For the purpose of generating the Nostr signature, the following data should be serialized: [0,sender_id,ts,1,[],message] to conform to the NIP-01 specification. The resulting serialized string should be hashed with sha256 to obtain the value.
    pub payment_signature: Option<String>,
    /// Recipient's name.
    pub name: Option<String>,
}

/// Arguments for [keysend_payment].
#[derive(Debug, Default)]
pub struct KeysendPaymentArgs<'a> {
    /// User agent.
    pub user_agent: &'a str,
    /// Token.
    pub token: &'a str,

    /// ACTION
    pub action: Action,

    /// FEED IDENTIFIER
    ///
    ///  The `<podcast:guid>` tag.
    pub feed_guid: Option<Uuid>,
    /// Title of the feed.
    pub feed_name: Option<String>,
    /// ID of podcast in PodcastIndex.org directory.
    pub feed_pi_id: Option<u64>,
    /// RSS feed URL of podcast.
    pub feed_url: Option<Url>,

    /// ITEM IDENTIFIER
    ///
    /// The `<item:guid>` tag.
    pub item_guid: Option<String>,
    /// Title of the item.
    pub item_name: Option<String>,
    /// ID of the item in PodcastIndex.org directory
    pub item_pi_id: Option<u64>,

    /// PLAYBACK INFO
    ///
    ///  Timestamp of when the payment was sent as an offset from zero (i.e. - playback position).
    pub timestamp: Option<Duration>,
    /// Speed in which the podcast was playing, in decimal notation at the time the payment was sent. So 0.5 is half speed and 2 is double speed.
    pub speed: Option<f64>,

    /// APP INFO
    ///
    /// Name of sending app
    pub app_name: Option<String>,
    /// Version of sending app
    pub app_version: Option<String>,

    /// SENDER INFO
    ///
    /// Name of the sender (free text, not validated in any way)
    pub sender_name: Option<String>,
    /// Static random identifier for users, not displayed by apps to prevent abuse. Apps can set this per-feed or app-wide. A GUID-like random identifier or a hash works well. Max 32 bytes (64 ascii characters). This can be a Nostr hex encoded pubkey (not NIP-19) for purposes of sender attribution.
    pub sender_id: Option<String>,

    /// PAYMENT INFO
    ///
    /// Total number of sats for the payment before any fees are subtracted. This should be the number the listener entered into the app. Preserving this value is important for numerology reasons. Certain numeric values can have significance to the sender and/or receiver, so giving a way to show this is critical.
    pub num_sats: Option<u64>,
    /// Text message to add to the payment. When this field is present, the payment is known as a "boostagram".
    pub message: Option<String>,
    /// App-specific URL containing route to podcast, episode, and/or timestamp at time of the action. The use case for this is sending a link along with the payment that will take the recipient to the exact playback position within the episode where the payment was sent.
    pub boost_link: Option<Url>,
    /// UUID for the boost/stream/auto payment. If there are several recipients, the same identifier should be sent to all of them.
    pub boost_id: Option<String>,

    /// REMOTE INFO
    ///
    /// Sometimes a payment will be sent to a feed's value block because a different feed referenced it in a <podcast:valueTimeSplit> tag. When that happens, this field will contain the guid of the referencing feed.
    pub remote_feed_guid: Option<Uuid>,
    /// Sometimes a payment will be sent to an episode's value block because a different feed referenced it in a <podcast:valueTimeSplit> tag. When that happens, this field will contain the guid of the referencing feed's <item>.
    pub remote_item_guid: Option<String>,

    /// Keysend address of the sender.
    pub reply_address: Option<KeysendAddress>,

    /// Recipients.
    pub recipients: Vec<KeysendPaymentRecipientArgs>,
}

/// Send a keysend payment to multiple Podcasting 2.0 recipients using the Alby API.
pub async fn keysend_payment(
    args: KeysendPaymentArgs<'_>,
) -> Result<MultiKeysendResponse, RequestError> {
    let mut keysends: Vec<MultiKeysendItemArgs> = vec![];

    for recipient in args.recipients.iter() {
        let mut custom_records = HashMap::new();
        if let Some(custom_data) = recipient.address.custom_data.as_ref() {
            custom_records.insert(custom_data.0.clone(), custom_data.1.clone());
        }

        let tlv_record = TlvRecord {
            action: args.action.clone(),
            feed_guid: args.feed_guid,
            feed_name: args.feed_name.clone(),
            feed_pi_id: args.feed_pi_id,
            feed_url: args.feed_url.clone(),
            item_guid: args.item_guid.clone(),
            item_name: args.item_name.clone(),
            item_pi_id: args.item_pi_id,
            timestamp_seconds: args.timestamp.map(|duration| duration.num_seconds() as f64),
            timestamp_hhmmss: None,
            speed: args.speed,
            app_name: args.app_name.clone(),
            app_version: args.app_version.clone(),
            sender_name: args.sender_name.clone(),
            sender_id: args.sender_id.clone(),
            receiver_name: recipient.name.clone(),
            value_msat_total: args.num_sats,
            message: args.message.clone(),
            boost_link: args.boost_link.clone(),
            payment_signature: recipient.payment_signature.clone(),
            payment_id: recipient.payment_id.clone(),
            boost_id: args.boost_id.clone(),
            remote_feed_guid: args.remote_feed_guid,
            remote_item_guid: args.remote_item_guid.clone(),
            reply_address: args
                .reply_address
                .as_ref()
                .map(|address| address.pubkey.clone()),
            reply_custom_key: args.reply_address.as_ref().and_then(|address| {
                address
                    .custom_data
                    .as_ref()
                    .map(|custom_data| custom_data.0.clone())
            }),
            reply_custom_value: args.reply_address.as_ref().and_then(|address| {
                address
                    .custom_data
                    .as_ref()
                    .map(|custom_data| custom_data.1.clone())
            }),
        };

        let tlv_record_string = serde_json::to_string(&tlv_record).map_err(|error| {
            RequestError::Unexpected(format!("Failed to serialize TLV record: {}", error))
        })?;
        custom_records.insert("7629169".to_string(), tlv_record_string);

        keysends.push(MultiKeysendItemArgs {
            num_sats: recipient.num_sats,
            dest_pubkey: recipient.address.pubkey.as_str(),
            custom_records,
        });
    }

    crate::alby::api::payments::multi_keysend(crate::alby::api::payments::MultiKeysendArgs {
        user_agent: args.user_agent,
        token: args.token,
        keysends,
    })
    .await
}
