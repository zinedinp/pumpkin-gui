//! The first-run wizard: shown whenever [`crate::launcher`] has not resolved a server to talk to.

// cxx-qt expands into generated glue that does not follow the workspace's lint profile.
#![allow(
    clippy::used_underscore_binding,
    clippy::unnecessary_box_returns,
    clippy::needless_lifetimes,
    clippy::multiple_unsafe_ops_per_block,
    clippy::undocumented_unsafe_blocks
)]

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

        if *self.as_ref().needs_setup() != needs_setup {
            self.as_mut().set_needs_setup(needs_setup);
        }

        if *self.as_ref().failed() != failed {
            self.as_mut().set_failed(failed);
        }

        let headline = QString::from(&headline);
        if *self.as_ref().headline() != headline {
            self.as_mut().set_headline(headline);
        }

        let managed = crate::launcher::is_managed();
        if *self.as_ref().managed() != managed {
            self.as_mut().set_managed(managed);
        }

        let message = QString::from(&message);
        if *self.as_ref().status_message() != message {
            self.as_mut().set_status_message(message);
        }
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
