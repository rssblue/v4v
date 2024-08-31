use std::collections::HashMap;

use chrono::Duration;
use url::Url;
use uuid::Uuid;

use crate::alby::{
    api::{
        payments::{MultiKeysendItemArgs, MultiKeysendResponse},
        RequestError,
    },
    types::KeysendAddress,
};

use super::tlv::Record;

/// Action for Podcasting 2.0 payment.
#[derive(
    Debug, Default, serde::Deserialize, PartialEq, Clone, serde::Serialize, strum::Display,
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

/// Information describing a payment recipient.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRecipientInfo {
    /// Recipient's keysend address.
    pub address: KeysendAddress,
    /// Number of sats to send.
    pub num_sats: u64,
    /// UUID of a payment sent out to a single recipient.
    #[serde(default)]
    pub payment_id: Option<Uuid>,
    /// Optionally, this field can contain a signature for the payment, to be able to verify that the user who sent it is actually who they claim in the sender_id field. If the sender_id contains a Nostr public key, this field should contain a Nostr sig value as a 64-byte encoded hex string. For the purpose of generating the Nostr signature, the following data should be serialized: [0,sender_id,ts,1,[],message] to conform to the NIP-01 specification. The resulting serialized string should be hashed with sha256 to obtain the value.
    #[serde(default)]
    pub payment_signature: Option<String>,
    /// Recipient's name.
    #[serde(default)]
    pub name: Option<String>,
}

/// Information describing a boost/stream/auto payment.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PaymentInfo {
    /// ACTION
    #[serde(default)]
    pub action: Action,

    /// FEED IDENTIFIER
    ///
    ///  The `<podcast:guid>` tag.
    #[serde(default)]
    pub feed_guid: Option<Uuid>,
    /// Title of the feed.
    #[serde(default)]
    pub feed_name: Option<String>,
    /// ID of podcast in PodcastIndex.org directory.
    #[serde(default)]
    pub feed_pi_id: Option<u64>,
    /// RSS feed URL of podcast.
    #[serde(default)]
    pub feed_url: Option<Url>,

    /// ITEM IDENTIFIER
    ///
    /// The `<item:guid>` tag.
    #[serde(default)]
    pub item_guid: Option<String>,
    /// Title of the item.
    #[serde(default)]
    pub item_name: Option<String>,
    /// ID of the item in PodcastIndex.org directory
    #[serde(default)]
    pub item_pi_id: Option<u64>,

    /// PLAYBACK INFO
    ///
    ///  Timestamp of when the payment was sent as an offset from zero (i.e. - playback position).
    #[serde(
        serialize_with = "crate::pc20::tlv::serialize_duration_to_seconds",
        deserialize_with = "crate::pc20::tlv::deserialize_seconds",
        default
    )]
    pub timestamp: Option<Duration>,
    /// Speed in which the podcast was playing, in decimal notation at the time the payment was sent. So 0.5 is half speed and 2 is double speed.
    #[serde(default)]
    pub speed: Option<f64>,

    /// APP INFO
    ///
    /// Name of sending app
    #[serde(default)]
    pub app_name: Option<String>,
    /// Version of sending app
    #[serde(default)]
    pub app_version: Option<String>,

    /// SENDER INFO
    ///
    /// Name of the sender (free text, not validated in any way)
    #[serde(default)]
    pub sender_name: Option<String>,
    /// Static random identifier for users, not displayed by apps to prevent abuse. Apps can set this per-feed or app-wide. A GUID-like random identifier or a hash works well. Max 32 bytes (64 ascii characters). This can be a Nostr hex encoded pubkey (not NIP-19) for purposes of sender attribution.
    #[serde(default)]
    pub sender_id: Option<String>,

    /// PAYMENT INFO
    ///
    /// Total number of sats for the payment before any fees are subtracted. This should be the number the listener entered into the app. Preserving this value is important for numerology reasons. Certain numeric values can have significance to the sender and/or receiver, so giving a way to show this is critical.
    pub total_num_sats: u64,
    /// Text message to add to the payment. When this field is present, the payment is known as a "boostagram".
    #[serde(default)]
    pub message: Option<String>,
    /// App-specific URL containing route to podcast, episode, and/or timestamp at time of the action. The use case for this is sending a link along with the payment that will take the recipient to the exact playback position within the episode where the payment was sent.
    #[serde(default)]
    pub boost_link: Option<Url>,
    /// UUID for the boost/stream/auto payment. If there are several recipients, the same identifier should be sent to all of them.
    #[serde(default)]
    pub boost_id: Option<Uuid>,

    /// REMOTE INFO
    ///
    /// Sometimes a payment will be sent to a feed's value block because a different feed referenced it in a <podcast:valueTimeSplit> tag. When that happens, this field will contain the guid of the referencing feed.
    #[serde(default)]
    pub remote_feed_guid: Option<Uuid>,
    /// Sometimes a payment will be sent to an episode's value block because a different feed referenced it in a <podcast:valueTimeSplit> tag. When that happens, this field will contain the guid of the referencing feed's `<item>`.
    #[serde(default)]
    pub remote_item_guid: Option<String>,

    /// Keysend address of the sender.
    #[serde(default)]
    pub reply_address: Option<KeysendAddress>,
}

/// Arguments for [make_payment].
#[derive(Debug, Default)]
pub struct MakePaymentArgs<'a> {
    /// User agent.
    pub user_agent: &'a str,
    /// Token.
    pub token: &'a str,
    /// Payment information.
    pub payment_info: PaymentInfo,
    /// Recipients' information.
    pub recipients: Vec<PaymentRecipientInfo>,
}

/// Send a payment to multiple Podcasting 2.0 recipients using the Alby API.
pub async fn make_payment(args: MakePaymentArgs<'_>) -> Result<MultiKeysendResponse, RequestError> {
    let mut keysends: Vec<MultiKeysendItemArgs> = vec![];

    for recipient in args.recipients.iter() {
        let mut custom_records = HashMap::new();
        if let Some(custom_data) = recipient.address.custom_data.as_ref() {
            custom_records.insert(custom_data.0.clone(), custom_data.1.clone());
        }

        let tlv_record = Record {
            action: args.payment_info.action.clone(),
            feed_guid: args.payment_info.feed_guid,
            feed_name: args.payment_info.feed_name.clone(),
            feed_pi_id: args.payment_info.feed_pi_id,
            feed_url: args.payment_info.feed_url.clone(),
            item_guid: args.payment_info.item_guid.clone(),
            item_name: args.payment_info.item_name.clone(),
            item_pi_id: args.payment_info.item_pi_id,
            timestamp_seconds: args.payment_info.timestamp,
            timestamp_hhmmss: None,
            speed: args.payment_info.speed,
            app_name: args.payment_info.app_name.clone(),
            app_version: args.payment_info.app_version.clone(),
            sender_name: args.payment_info.sender_name.clone(),
            sender_id: args.payment_info.sender_id.clone(),
            receiver_name: recipient.name.clone(),
            total_num_millisats: match args.payment_info.total_num_sats {
                0 => Some(args.recipients.iter().map(|r| r.num_sats).sum::<u64>() * 1000),
                _ => Some(args.payment_info.total_num_sats * 1000),
            },
            message: args.payment_info.message.clone(),
            boost_link: args.payment_info.boost_link.clone(),
            payment_signature: recipient.payment_signature.clone(),
            payment_id: recipient.payment_id,
            boost_id: args.payment_info.boost_id,
            remote_feed_guid: args.payment_info.remote_feed_guid,
            remote_item_guid: args.payment_info.remote_item_guid.clone(),
            reply_address: args
                .payment_info
                .reply_address
                .as_ref()
                .map(|address| address.pubkey.clone()),
            reply_custom_key: args
                .payment_info
                .reply_address
                .as_ref()
                .and_then(|address| {
                    address
                        .custom_data
                        .as_ref()
                        .map(|custom_data| custom_data.0.clone())
                }),
            reply_custom_value: args
                .payment_info
                .reply_address
                .as_ref()
                .and_then(|address| {
                    address
                        .custom_data
                        .as_ref()
                        .map(|custom_data| custom_data.1.clone())
                }),
        };

        let tlv_record_string = serde_json::to_string(&tlv_record).map_err(|error| {
            RequestError::Unexpected(format!("Failed to serialize TLV record: {}", error))
        })?;
        // bLIP-10 TLV record:
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
