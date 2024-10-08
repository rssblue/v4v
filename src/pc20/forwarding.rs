pub use super::payments::MakePaymentArgs as ForwardPaymentArgs;
use super::payments::{make_payment, PaymentInfo, PaymentRecipientInfo};
use crate::alby::{
    api::{
        invoices::{CreateInvoiceArgs as AlbyCreateInvoiceArgs, CreateInvoiceResponse},
        RequestError,
    },
    webhooks::AlbyInvoice,
};

/// Arguments for creating an invoice for forwarding payments to multiple Podcasting 2.0
/// recipients.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateInvoiceArgs<'a> {
    /// User agent.
    pub user_agent: &'a str,
    /// Alby token.
    pub token: &'a str,
    /// Payment information.
    pub payment_info: PaymentInfo,
    /// Recipients' information.
    pub recipients: Vec<PaymentRecipientInfo>,
}

/// Metadata fields for sat forwarding service using Alby API.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateInvoiceMetadataForwardingStruct {
    /// Payment information.
    pub payment_info: PaymentInfo,
    /// Recipients' information.
    pub recipients: Vec<PaymentRecipientInfo>,
}

/// Metadata for sat forwarding service using Alby API.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateInvoiceMetadata {
    /// Forwarding data.
    pub forwarding_data: CreateInvoiceMetadataForwardingStruct,
}

impl TryFrom<AlbyInvoice> for CreateInvoiceMetadata {
    type Error = serde_json::Error;

    fn try_from(invoice: AlbyInvoice) -> Result<Self, Self::Error> {
        let forwarding_data = serde_json::from_value(invoice.metadata)?;

        Ok(Self { forwarding_data })
    }
}

/// Creates an invoice for forwarding payments to multiple Podcasting 2.0 recipients.
pub async fn create_invoice(
    args: CreateInvoiceArgs<'_>,
) -> Result<CreateInvoiceResponse, RequestError> {
    // `total_num_millisats` might be different from the sum of `num_sats` in `recipients` because
    // of how it may be calculated on the front end.
    let total_sats = args
        .recipients
        .iter()
        .fold(0, |acc, recipient| acc + recipient.num_sats);

    let metadata_value = serde_json::json!(CreateInvoiceMetadata {
        forwarding_data: CreateInvoiceMetadataForwardingStruct {
            payment_info: args.payment_info.clone(),
            recipients: args.recipients,
        },
    });

    let invoice_args = AlbyCreateInvoiceArgs {
        user_agent: args.user_agent,
        token: args.token,
        num_sats: total_sats,
        metadata: metadata_value,
        description: args.payment_info.message.clone(),
        payer_name: args.payment_info.sender_name.clone(),
    };

    crate::alby::api::invoices::create_invoice(invoice_args).await
}

/// Forwards payments to multiple Podcasting 2.0 recipients.
pub async fn forward_payments(args: ForwardPaymentArgs<'_>) -> Result<(), RequestError> {
    make_payment(ForwardPaymentArgs {
        user_agent: args.user_agent,
        token: args.token,
        payment_info: args.payment_info.clone(),
        recipients: args.recipients.clone(),
    })
    .await
    .map(|_| ())
}

/// Ensure forwarded amounts do not exceed a specified aomunt.
///
/// Useful for double-checking that the sats forwarded do not exceed the sats received.
pub fn clip_recipients_at_amount(
    total_sats: u64,
    recipients: &[PaymentRecipientInfo],
) -> Vec<PaymentRecipientInfo> {
    let mut totals_sats_sent = 0;
    let mut clipped_recipients = vec![];

    for recipient in recipients {
        totals_sats_sent += recipient.num_sats;
        if totals_sats_sent > total_sats {
            break;
        }

        clipped_recipients.push(recipient.clone());
    }

    clipped_recipients
}
