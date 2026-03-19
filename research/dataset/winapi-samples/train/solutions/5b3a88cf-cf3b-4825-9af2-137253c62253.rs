use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result};

// Helper to convert OsStr to null-terminated UTF-16
fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

// XML escape helper
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

struct Toast {
    app_id: String,
    title: Option<String>,
    text1: Option<String>,
    buttons: Vec<(String, String)>,
}

impl Toast {
    pub fn new(app_id: &str) -> Self {
        Self {
            app_id: app_id.to_string(),
            title: None,
            text1: None,
            buttons: Vec::new(),
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn text1(mut self, text: &str) -> Self {
        self.text1 = Some(text.to_string());
        self
    }

    pub fn button(mut self, label: &str, activation_args: &str) -> Self {
        if self.buttons.len() < 5 {
            let escaped_label = xml_escape(label);
            let escaped_args = xml_escape(activation_args);
            self.buttons.push((escaped_label, escaped_args));
        }
        self
    }

    pub fn show(&self) -> Result<()> {
        // Implementation would use Win32 Toast APIs
        // This is a placeholder that demonstrates the structure
        Ok(())
    }
}
