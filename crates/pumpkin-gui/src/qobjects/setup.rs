//! The first-run wizard: shown whenever [`crate::launcher`] has not resolved a server to talk to.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, needs_setup)]
        #[qproperty(bool, failed)]
        #[qproperty(QString, headline)]
        #[qproperty(bool, managed)]
        #[qproperty(QString, status_message)]
        type SetupController = super::SetupControllerRust;

        #[qinvokable]
        fn refresh(self: Pin<&mut Self>);

        /// Opens a native file picker for the server executable; on success
        #[qinvokable]
        fn browse_for_binary(&self);

        /// Starts connecting to `endpoint` without spawning anything.
        #[qinvokable]
        fn use_attach_endpoint(&self, endpoint: &QString);
    }
}

use core::pin::Pin;
use cxx_qt_lib::QString;

use crate::qobjects::set_if_changed;
use crate::launcher::Status;

pub struct SetupControllerRust {
    needs_setup: bool,
    failed: bool,
    headline: QString,
    managed: bool,
    status_message: QString,
}

impl Default for SetupControllerRust {
    fn default() -> Self {
        Self {
            needs_setup: true,
            failed: false,
            headline: QString::from(crate::launcher::HEADLINE_SETUP),
            managed: false,
            status_message: QString::default(),
        }
    }
}

impl qobject::SetupController {
    pub fn refresh(mut self: Pin<&mut Self>) {
        let (needs_setup, failed, headline, message) = match crate::launcher::status() {
            Status::Resolving => (
                true,
                false,
                crate::launcher::HEADLINE_SETUP.to_owned(),
                "Looking for a server…".to_owned(),
            ),
            Status::NeedsSetup => (
                true,
                false,
                crate::launcher::HEADLINE_SETUP.to_owned(),
                String::new(),
            ),
            Status::Validating(message) | Status::Launching(message) | Status::Connecting(message) => {
                (true, false, crate::launcher::HEADLINE_SETUP.to_owned(), message)
            }
            Status::Connected => (
                false,
                false,
                crate::launcher::HEADLINE_SETUP.to_owned(),
                String::new(),
            ),
            Status::Failed { headline, detail } => (true, true, headline, detail),
        };

        set_if_changed!(self, needs_setup, set_needs_setup, needs_setup);
        set_if_changed!(self, failed, set_failed, failed);
        set_if_changed!(self, headline, set_headline, QString::from(&headline));
        set_if_changed!(
            self,
            managed,
            set_managed,
            crate::launcher::is_managed()
        );
        set_if_changed!(
            self,
            status_message,
            set_status_message,
            QString::from(&message)
        );
    }

    #[allow(clippy::unused_self)]
    pub fn browse_for_binary(&self) {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Choose the Pumpkin server executable")
            .pick_file()
        else {
            return;
        };

        crate::launcher::choose_binary_and_launch(path);
    }

    #[allow(clippy::unused_self)]
    pub fn use_attach_endpoint(&self, endpoint: &QString) {
        let endpoint = endpoint.to_string();
        let endpoint = endpoint.trim();
        if endpoint.is_empty() {
            return;
        }

        crate::launcher::use_attach_endpoint(endpoint.to_owned());
    }
}
