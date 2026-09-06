//! The `QObject`s exposed to QML.

/// cxx-qt expands into generated glue that does not follow the workspace's lint profile. Applied
/// here rather than repeated at the top of every bridge file.
macro_rules! cxx_qt_modules {
    ($($name:ident),* $(,)?) => {
        $(
            #[allow(
                clippy::used_underscore_binding,
                clippy::unnecessary_box_returns,
                clippy::needless_lifetimes,
                clippy::multiple_unsafe_ops_per_block,
                clippy::undocumented_unsafe_blocks
            )]
            pub mod $name;
        )*
    };
}

cxx_qt_modules!(console, dev_tools, players, server_stats, setup);

/// Sets a `qproperty` only when the value actually differs, so Qt does not emit a change signal
/// (and QML does not re-render) on every poll.
macro_rules! set_if_changed {
    ($self:ident, $getter:ident, $setter:ident, $value:expr) => {{
        let next = $value;
        if *$self.as_ref().$getter() != next {
            $self.as_mut().$setter(next);
        }
    }};
}

pub(crate) use set_if_changed;

/// Builds the `QVariantMap` one table row is handed to QML as.
///
/// The alternative is a `QMap::insert` call per key with its own `QString::from`, which is what
/// the row builders used to be.
pub struct RowBuilder {
    map: cxx_qt_lib::QMap<cxx_qt_lib::QMapPair_QString_QVariant>,
}

impl RowBuilder {
    pub fn new() -> Self {
        Self {
            map: cxx_qt_lib::QMap::default(),
        }
    }

    pub fn set(mut self, key: &str, value: cxx_qt_lib::QVariant) -> Self {
        self.map.insert(cxx_qt_lib::QString::from(key), value);
        self
    }

    /// Convenience for the many string fields, which need a `QString` first.
    pub fn text(self, key: &str, value: &str) -> Self {
        let text = cxx_qt_lib::QString::from(value);
        self.set(key, cxx_qt_lib::QVariant::from(&text))
    }

    pub fn build(self) -> cxx_qt_lib::QVariant {
        cxx_qt_lib::QVariant::from(&self.map)
    }
}
