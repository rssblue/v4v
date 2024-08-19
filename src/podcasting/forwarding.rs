use crate::alby::api::{
    invoices::{CreateInvoiceArgs as AlbyCreateInvoiceArgs, CreateInvoiceResponse},
    RequestError,
};

use super::payments::{PaymentInfo, PaymentRecipientInfo};

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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CreateInvoiceMetadataForwardingStruct {
    /// Payment information.
    pub payment_info: PaymentInfo,
    /// Recipients' information.
    pub recipients: Vec<PaymentRecipientInfo>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CreateInvoiceMetadata {
    pub forwarding_data: CreateInvoiceMetadataForwardingStruct,
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
            payment_info: args.payment_info,
            recipients: args.recipients,
        },
    });

    let invoice_args = AlbyCreateInvoiceArgs {
        user_agent: args.user_agent,
        token: args.token,
        num_sats: total_sats,
        metadata: metadata_value,
    };

    crate::alby::api::invoices::create_invoice(invoice_args).await
}
