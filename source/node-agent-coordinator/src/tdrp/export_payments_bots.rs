use crate::protocol::Failure;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Html,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum ExportStatus {
    Queued,
    Processing {
        percentage: u32,
    },
    Completed {
        download_url: String,
        total_bytes: u64,
    },
    Failed {
        reason: String,
    },
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportJob {
    pub job_id: String,
    pub user_id: String,
    pub chat_id: String,
    pub format: ExportFormat,
    pub status: ExportStatus,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaymentInvoice {
    pub invoice_id: String,
    pub title: String,
    pub description: String,
    pub currency: String,
    pub total_amount_cents: u64,
    pub is_paid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_payment_charge_id: Option<String>,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InlineQueryResult {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub result_type: String,
    pub send_message_text: String,
}

#[derive(Debug, Clone, Default)]
pub struct ExportPaymentsBotsState {
    pub export_jobs: BTreeMap<String, ExportJob>,
    pub invoices: BTreeMap<String, PaymentInvoice>,
    pub registered_inline_results: BTreeMap<String, Vec<InlineQueryResult>>,
    next_export_id: u32,
    next_invoice_id: u32,
}

impl ExportPaymentsBotsState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_chat_export(
        &mut self,
        user_id: String,
        chat_id: String,
        format: ExportFormat,
        now_ms: i64,
    ) -> Result<ExportJob, Failure> {
        self.next_export_id += 1;
        let job_id = format!("export_{}", self.next_export_id);
        let job = ExportJob {
            job_id: job_id.clone(),
            user_id,
            chat_id,
            format,
            status: ExportStatus::Queued,
            created_at_ms: now_ms,
        };
        self.export_jobs.insert(job_id, job.clone());
        Ok(job)
    }

    pub fn get_export_status(&self, job_id: &str) -> Result<ExportJob, Failure> {
        self.export_jobs
            .get(job_id)
            .cloned()
            .ok_or_else(|| Failure::new("EXPORT_NOT_FOUND", format!("job {job_id} not found")))
    }

    pub fn update_export_progress(
        &mut self,
        job_id: &str,
        status: ExportStatus,
    ) -> Result<ExportJob, Failure> {
        let job = self
            .export_jobs
            .get_mut(job_id)
            .ok_or_else(|| Failure::new("EXPORT_NOT_FOUND", format!("job {job_id} not found")))?;

        if matches!(job.status, ExportStatus::Cancelled) {
            return Err(Failure::new(
                "EXPORT_CANCELLED",
                "cannot update a cancelled export job",
            ));
        }

        job.status = status;
        Ok(job.clone())
    }

    pub fn cancel_export(&mut self, job_id: &str) -> Result<bool, Failure> {
        let job = self
            .export_jobs
            .get_mut(job_id)
            .ok_or_else(|| Failure::new("EXPORT_NOT_FOUND", format!("job {job_id} not found")))?;

        if matches!(job.status, ExportStatus::Completed { .. }) {
            return Err(Failure::new(
                "CANNOT_CANCEL_COMPLETED",
                "cannot cancel already completed job",
            ));
        }

        job.status = ExportStatus::Cancelled;
        Ok(true)
    }

    pub fn create_invoice(
        &mut self,
        title: String,
        description: String,
        currency: String,
        total_amount_cents: u64,
        now_ms: i64,
    ) -> Result<PaymentInvoice, Failure> {
        if title.trim().is_empty() {
            return Err(Failure::new(
                "INVALID_INVOICE_TITLE",
                "invoice title cannot be empty",
            ));
        }
        if total_amount_cents == 0 {
            return Err(Failure::new(
                "INVALID_AMOUNT",
                "invoice amount must be greater than zero",
            ));
        }

        self.next_invoice_id += 1;
        let invoice_id = format!("inv_{}", self.next_invoice_id);
        let invoice = PaymentInvoice {
            invoice_id: invoice_id.clone(),
            title,
            description,
            currency: currency.to_uppercase(),
            total_amount_cents,
            is_paid: false,
            provider_payment_charge_id: None,
            created_at_ms: now_ms,
        };
        self.invoices.insert(invoice_id, invoice.clone());
        Ok(invoice)
    }

    pub fn settle_payment(
        &mut self,
        invoice_id: &str,
        charge_id: String,
    ) -> Result<PaymentInvoice, Failure> {
        let invoice = self.invoices.get_mut(invoice_id).ok_or_else(|| {
            Failure::new(
                "INVOICE_NOT_FOUND",
                format!("invoice {invoice_id} not found"),
            )
        })?;

        if invoice.is_paid {
            return Err(Failure::new(
                "INVOICE_ALREADY_PAID",
                "invoice has already been settled",
            ));
        }

        invoice.is_paid = true;
        invoice.provider_payment_charge_id = Some(charge_id);
        Ok(invoice.clone())
    }

    pub fn register_inline_results(
        &mut self,
        bot_id: &str,
        results: Vec<InlineQueryResult>,
    ) {
        self.registered_inline_results
            .insert(bot_id.to_string(), results);
    }

    pub fn query_inline_bot(&self, bot_id: &str, query: &str) -> Vec<InlineQueryResult> {
        let query_lower = query.to_lowercase();
        self.registered_inline_results
            .get(bot_id)
            .map(|list| {
                list.iter()
                    .filter(|r| {
                        if query_lower.is_empty() {
                            true
                        } else {
                            r.title.to_lowercase().contains(&query_lower)
                                || r.description
                                    .as_ref()
                                    .is_some_and(|d| d.to_lowercase().contains(&query_lower))
                        }
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn handle_route(&mut self, method: &str, args: &Value) -> Result<Value, Failure> {
        if let Some(op) = method.strip_prefix("export.") {
            match op {
                "startChatExport" => {
                    let user_id = args
                        .get("userId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "userId required"))?
                        .to_string();
                    let chat_id = args
                        .get("chatId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "chatId required"))?
                        .to_string();
                    let format: ExportFormat = args
                        .get("format")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or(ExportFormat::Html);
                    let now_ms = args
                        .get("nowMs")
                        .and_then(Value::as_i64)
                        .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

                    let job = self.start_chat_export(user_id, chat_id, format, now_ms)?;
                    Ok(json!({ "exportJob": job }))
                }
                "getExportStatus" => {
                    let job_id = args
                        .get("jobId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "jobId required"))?;
                    let job = self.get_export_status(job_id)?;
                    Ok(json!({ "exportJob": job }))
                }
                "updateProgress" => {
                    let job_id = args
                        .get("jobId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "jobId required"))?;
                    let status: ExportStatus = args
                        .get("status")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "status required"))?;

                    let job = self.update_export_progress(job_id, status)?;
                    Ok(json!({ "exportJob": job }))
                }
                "cancelExport" => {
                    let job_id = args
                        .get("jobId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "jobId required"))?;
                    let cancelled = self.cancel_export(job_id)?;
                    Ok(json!({ "cancelled": cancelled }))
                }
                other => Err(Failure::new(
                    "UNKNOWN_EXPORT_METHOD",
                    format!("unrecognized export method {other}"),
                )),
            }
        } else if let Some(op) = method.strip_prefix("payments.") {
            match op {
                "createInvoice" => {
                    let title = args
                        .get("title")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    let description = args
                        .get("description")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    let currency = args
                        .get("currency")
                        .and_then(Value::as_str)
                        .unwrap_or("USD")
                        .to_string();
                    let total_amount_cents = args
                        .get("totalAmountCents")
                        .and_then(Value::as_u64)
                        .unwrap_or(0);
                    let now_ms = args
                        .get("nowMs")
                        .and_then(Value::as_i64)
                        .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

                    let inv = self.create_invoice(
                        title,
                        description,
                        currency,
                        total_amount_cents,
                        now_ms,
                    )?;
                    Ok(json!({ "invoice": inv }))
                }
                "settlePayment" => {
                    let invoice_id = args
                        .get("invoiceId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "invoiceId required"))?;
                    let charge_id = args
                        .get("chargeId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "chargeId required"))?
                        .to_string();

                    let inv = self.settle_payment(invoice_id, charge_id)?;
                    Ok(json!({ "invoice": inv }))
                }
                other => Err(Failure::new(
                    "UNKNOWN_PAYMENTS_METHOD",
                    format!("unrecognized payments method {other}"),
                )),
            }
        } else if let Some(op) = method.strip_prefix("bots.") {
            match op {
                "registerInline" => {
                    let bot_id = args
                        .get("botId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "botId required"))?;
                    let results: Vec<InlineQueryResult> = args
                        .get("results")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default();
                    self.register_inline_results(bot_id, results);
                    Ok(json!({ "success": true }))
                }
                "queryInline" => {
                    let bot_id = args
                        .get("botId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| Failure::new("MISSING_ARGUMENT", "botId required"))?;
                    let query = args.get("query").and_then(Value::as_str).unwrap_or("");
                    let results = self.query_inline_bot(bot_id, query);
                    Ok(json!({ "results": results }))
                }
                other => Err(Failure::new(
                    "UNKNOWN_BOTS_METHOD",
                    format!("unrecognized bots method {other}"),
                )),
            }
        } else {
            Err(Failure::new(
                "UNKNOWN_METHOD",
                format!("unrecognized method {method}"),
            ))
        }
    }
}
