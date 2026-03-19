use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
};
use windows::core::{Result, HSTRING};
use windows::Data::Xml::Dom::XmlDocument;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

static TOAST_COUNTER: AtomicU32 = AtomicU32::new(0);

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub struct ProgressToast {
    app_id: HSTRING,
    tag: String,
    notifier: windows::UI::Notifications::ToastNotifier,
    toast: Arc<Mutex<Option<ToastNotification>>>,
    title: String,
    body: Arc<Mutex<Option<String>>>,
}

impl ProgressToast {
    pub fn new(app_id: &str, title: &str, initial_status: &str) -> Result<Self> {
        // Initialize COM for this thread
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()? };

        let app_id_hstring = HSTRING::from(app_id);
        let notifier = ToastNotificationManager::CreateToastNotifierWithId(&app_id_hstring)?;

        // Generate a unique tag for this toast
        let tag = format!("progress_{}", TOAST_COUNTER.fetch_add(1, Ordering::SeqCst));

        let toast = Self::create_toast_notification(title, None, 0.0, initial_status, &tag)?;

        notifier.Show(&toast)?;

        Ok(Self {
            app_id: app_id_hstring,
            tag,
            notifier,
            toast: Arc::new(Mutex::new(Some(toast))),
            title: title.to_string(),
            body: Arc::new(Mutex::new(None)),
        })
    }

    pub fn update_progress(&self, value: f64, status: &str) -> Result<()> {
        let body = self.body.lock().unwrap().clone();
        let new_toast = Self::create_toast_notification(
            &self.title,
            body.as_deref(),
            value,
            status,
            &self.tag,
        )?;

        // Show new toast with same tag - system replaces existing one
        self.notifier.Show(&new_toast)?;

        let mut toast_guard = self.toast.lock().unwrap();
        *toast_guard = Some(new_toast);

        Ok(())
    }

    pub fn set_body(&self, body: &str) -> Result<()> {
        *self.body.lock().unwrap() = Some(body.to_string());

        let new_toast =
            Self::create_toast_notification(&self.title, Some(body), 0.0, "", &self.tag)?;

        self.notifier.Show(&new_toast)?;

        let mut toast_guard = self.toast.lock().unwrap();
        *toast_guard = Some(new_toast);

        Ok(())
    }

    pub fn dismiss(&self) -> Result<()> {
        let mut toast_guard = self.toast.lock().unwrap();
        if let Some(toast) = toast_guard.take() {
            // Hide the toast notification
            self.notifier.Hide(&toast)?;
        }
        Ok(())
    }

    fn create_toast_notification(
        title: &str,
        body: Option<&str>,
        progress: f64,
        status: &str,
        tag: &str,
    ) -> Result<ToastNotification> {
        let xml = Self::build_toast_xml(title, body, progress, status);
        let document = XmlDocument::new()?;
        document.LoadXml(&HSTRING::from(xml))?;

        let toast = ToastNotification::CreateToastNotification(&document)?;
        toast.SetTag(&HSTRING::from(tag))?;
        toast.SetGroup(&HSTRING::from("progress_group"))?;

        Ok(toast)
    }

    fn build_toast_xml(title: &str, body: Option<&str>, progress: f64, status: &str) -> String {
        let escaped_title = xml_escape(title);
        let escaped_status = xml_escape(status);
        let progress_value = progress.clamp(0.0, 1.0);

        let body_text = match body {
            Some(b) => format!("<text>{}</text>", xml_escape(b)),
            None => String::new(),
        };

        format!(
            r#"<toast duration="long">
    <visual>
        <binding template="ToastGeneric">
            <text>{}</text>
            {}
            <progress value="{}" status="{}"/>
        </binding>
    </visual>
</toast>"#,
            escaped_title, body_text, progress_value, escaped_status
        )
    }
}

impl Drop for ProgressToast {
    fn drop(&mut self) {
        let _ = self.dismiss();
    }
}
