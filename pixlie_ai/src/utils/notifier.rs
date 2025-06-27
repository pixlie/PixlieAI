// Copyright 2025 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::error::{PiError, PiResult};
use crate::workspace::{APIProvider, Workspace, WorkspaceCollection};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailNotification {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub url: String,
    pub relevant_items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub sendgrid_api_key: String,
    pub from_email: String,
    pub from_name: String,
    pub to_email: String,
}


impl EmailConfig {
    pub fn from_workspace() -> PiResult<Self> {
        let workspace = WorkspaceCollection::get_default()?;

        let sendgrid_api_key = workspace
            .get_api_key(&APIProvider::SendGrid)
            .cloned()
            .unwrap_or_else(|| "".to_string());

        let from_email = workspace
            .sendgrid_sender_email
            .unwrap_or_else(|| "".to_string());
        let to_email = workspace
            .sendgrid_receiver_email
            .unwrap_or_else(|| "".to_string());
        let from_name = "Pixlie AI Monitor".to_string();

        Ok(Self {
            sendgrid_api_key,
            from_email,
            from_name,
            to_email,
        })
    }
}

pub struct EmailNotifier {
    pub config: EmailConfig,
}

impl EmailNotifier {

    pub fn from_workspace() -> Self {
        let config = EmailConfig::from_workspace().unwrap_or_else(|_| EmailConfig {
            sendgrid_api_key: "".to_string(),
            from_email: "".to_string(),
            from_name: "Pixlie AI Monitor".to_string(),
            to_email: "".to_string(),
        });
        let notifier = Self { config };

        // Log configuration status on creation
        info!(
            "📧 Email configuration loaded from workspace: {}",
            notifier.get_config_status()
        );
        if notifier.is_configured() {
            info!("📧 Email notifications are ENABLED via SendGrid");
        } else {
            info!("📧 Email notifications are DISABLED - will log to console instead");
            info!("📧 To enable email, configure SendGrid in Settings");
        }

        notifier
    }

    pub fn new_with_config(config: EmailConfig) -> Self {
        Self { config }
    }

    /// Send email notification about relevant content changes
    pub fn send_notification(&self, notification: EmailNotification) -> PiResult<()> {
        if self.config.sendgrid_api_key.is_empty() {
            info!("📧 SendGrid API key not configured, logging notification instead:");
            self.log_notification(&notification);
            return Ok(());
        }

        match self.send_email_via_sendgrid(&notification) {
            Ok(_) => {
                info!(
                    "📧 Email notification sent successfully to {}",
                    notification.to
                );
                Ok(())
            }
            Err(e) => {
                error!("📧 Failed to send email notification: {}", e);
                info!("📧 Falling back to logging notification:");
                self.log_notification(&notification);
                Ok(()) // Don't return error, just log the notification
            }
        }
    }

    /// Log notification to console when email is not available
    fn log_notification(&self, notification: &EmailNotification) {
        info!("📧 EMAIL NOTIFICATION");
        info!("   To: {}", notification.to);
        info!("   Subject: {}", notification.subject);
        info!("   URL: {}", notification.url);
        info!("   Relevant Items: {}", notification.relevant_items.len());

        for (i, item) in notification.relevant_items.iter().enumerate() {
            info!("   {}. {}", i + 1, item);
        }

        info!(
            "   Body Preview: {}",
            if notification.body.len() > 200 {
                format!("{}...", &notification.body[..200])
            } else {
                notification.body.clone()
            }
        );
        info!("📧 END EMAIL NOTIFICATION");
    }

    /// Send email via SendGrid API
    fn send_email_via_sendgrid(&self, notification: &EmailNotification) -> PiResult<()> {
        debug!("📧 Attempting to send email via SendGrid API");
        debug!(
            "📧 From: {} <{}>",
            self.config.from_name, self.config.from_email
        );
        debug!("📧 To: {}", notification.to);
        debug!("📧 Subject: {}", notification.subject);

        // Build SendGrid API request
        let payload = serde_json::json!({
            "personalizations": [{
                "to": [{"email": notification.to}]
            }],
            "from": {
                "email": self.config.from_email,
                "name": self.config.from_name
            },
            "subject": notification.subject,
            "content": [{
                "type": "text/plain",
                "value": notification.body
            }]
        });

        debug!("📧 SendGrid payload built, making API request...");

        // Send via reqwest
        let client = reqwest::blocking::Client::new();
        let response = client
            .post("https://api.sendgrid.com/v3/mail/send")
            .header(
                "Authorization",
                format!("Bearer {}", self.config.sendgrid_api_key),
            )
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .map_err(|e| {
                error!("📧 SendGrid API request failed: {}", e);
                PiError::InternalError(format!("SendGrid API request failed: {}", e))
            })?;

        let status = response.status();
        debug!("📧 SendGrid API response status: {}", status);

        if status.is_success() {
            info!("📧 Email sent successfully via SendGrid!");
            Ok(())
        } else {
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("📧 SendGrid API error: {} - {}", status, error_text);
            Err(PiError::InternalError(format!(
                "SendGrid API error: {} - {}",
                status, error_text
            )))
        }
    }

    /// Create a formatted email notification for content changes
    pub fn create_content_change_notification(
        &self,
        to_email: &str,
        url: &str,
        objective: &str,
        relevant_items: Vec<String>,
    ) -> EmailNotification {
        let subject = format!(
            "🔍 New relevant content found: {}",
            if objective.len() > 50 {
                format!("{}...", &objective[..50])
            } else {
                objective.to_string()
            }
        );

        let body = self.format_email_body(url, objective, &relevant_items);

        EmailNotification {
            to: to_email.to_string(),
            subject,
            body,
            url: url.to_string(),
            relevant_items,
        }
    }

    /// Create a formatted email notification for relevant webpage classification
    pub fn create_insight_notification(
        &self,
        to_email: &str,
        url: &str,
        objective: &str,
        insight: &str,
        reason: &str,
    ) -> EmailNotification {
        let subject = format!(
            "🎯 Relevant webpage classified: {}",
            if objective.len() > 50 {
                format!("{}...", &objective[..50])
            } else {
                objective.to_string()
            }
        );

        let body = self.format_insight_email_body(url, objective, insight, reason);

        EmailNotification {
            to: to_email.to_string(),
            subject,
            body,
            url: url.to_string(),
            relevant_items: vec![insight.to_string()],
        }
    }

    /// Format the email body with relevant content
    fn format_email_body(&self, url: &str, objective: &str, relevant_items: &[String]) -> String {
        let mut body = String::new();

        body.push_str(&format!("Hello!\n\n"));
        body.push_str(&format!(
            "Pixlie AI has found {} new relevant item{} for your objective:\n\n",
            relevant_items.len(),
            if relevant_items.len() == 1 { "" } else { "s" }
        ));

        body.push_str(&format!("**Objective:** {}\n", objective));
        body.push_str(&format!("**Source:** {}\n\n", url));

        body.push_str("**New Relevant Items:**\n");
        for (i, item) in relevant_items.iter().enumerate() {
            body.push_str(&format!("{}. {}\n", i + 1, item));
        }

        body.push_str("\n---\n");
        body.push_str("This email was sent by Pixlie AI content monitoring.\n");
        body.push_str(&format!("View source: {}\n", url));

        body
    }

    /// Format the email body for insight notifications
    fn format_insight_email_body(
        &self,
        url: &str,
        objective: &str,
        insight: &str,
        reason: &str,
    ) -> String {
        let mut body = String::new();

        body.push_str("Hello!\n\n");
        body.push_str("Pixlie AI has classified a webpage as relevant to your objective and generated an insight:\n\n");

        body.push_str(&format!("**Objective:** {}\n", objective));
        body.push_str(&format!("**Source:** {}\n\n", url));

        body.push_str("**AI Insight:**\n");
        body.push_str(&format!("{}\n\n", insight));

        body.push_str("**Reason for Classification:**\n");
        body.push_str(&format!("{}\n\n", reason));

        body.push_str("---\n");
        body.push_str("This email was sent by Pixlie AI when relevant content was automatically classified.\n");
        body.push_str(&format!("View source: {}\n", url));

        body
    }

    /// Validate email configuration
    pub fn is_configured(&self) -> bool {
        !self.config.sendgrid_api_key.is_empty()
            && !self.config.from_email.is_empty()
            && !self.config.to_email.is_empty()
    }

    /// Get configuration status for debugging
    pub fn get_config_status(&self) -> String {
        let api_key_status = if self.config.sendgrid_api_key.is_empty() {
            "NOT_SET".to_string()
        } else {
            format!(
                "{}*** ({} chars)",
                &self
                    .config
                    .sendgrid_api_key
                    .chars()
                    .take(3)
                    .collect::<String>(),
                self.config.sendgrid_api_key.len()
            )
        };

        format!(
            "SendGrid API Key: {}, From: {} <{}>",
            api_key_status,
            self.config.from_name,
            if self.config.from_email.is_empty() {
                "NOT_SET"
            } else {
                &self.config.from_email
            }
        )
    }
}
