//! The server-stats `QObject`: pulls the newest [`crate::Snapshot`] into Qt properties.
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

        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;

        include!("cxx-qt-lib/qlist.h");
        type QList_f64 = cxx_qt_lib::QList<f64>;
        type QList_QVariant = cxx_qt_lib::QList<QVariant>;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, server_ready)]
        #[qproperty(f32, cpu_total)]
        /// Degrees Celsius, or -1 when the machine exposes no CPU sensor.
        #[qproperty(f64, cpu_temp_c)]
        /// Kept separate from `cpu_per_core` so the core grid can use a model that does not
        /// change when the readings do; recreating the delegates restarts their animations.
        #[qproperty(i32, cpu_core_count)]
        #[qproperty(f64, tps)]
        #[qproperty(f64, mspt)]
        #[qproperty(i32, player_count)]
        #[qproperty(f64, mem_process_rss)]
        #[qproperty(f64, mem_system_used)]
        #[qproperty(f64, mem_system_total)]
        #[qproperty(f64, uptime_secs)]
        #[qproperty(QString, pumpkin_version)]
        #[qproperty(QList_f64, cpu_per_core)]
        #[qproperty(QList_f64, tick_times_ms)]
        #[qproperty(f64, worlds_size_bytes)]
        #[qproperty(f64, disk_free)]
        #[qproperty(f64, disk_total)]
        #[qproperty(f64, net_in_bps)]
        #[qproperty(f64, net_out_bps)]
        #[qproperty(QString, java_address)]
        #[qproperty(QString, bedrock_address)]
        #[qproperty(f64, tick_budget_ms)]
        #[qproperty(i32, theme_preference)]
        /// One entry per loaded world, each a map QML reads by key.
        #[qproperty(QList_QVariant, worlds)]
        type ServerStats = super::ServerStatsRust;

        /// Pulls the newest snapshot. Driven by a QML `Timer`.
        #[qinvokable]
        fn refresh(self: Pin<&mut Self>);
    }
}

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QList, QMap, QMapPair_QString_QVariant, QString, QVariant};

/// Assigns a Qt property only when the value actually moved.
///
/// Every setter emits a change signal, so unconditional writes at 2 Hz would repaint the whole
/// window even while nothing changed.
macro_rules! set_if_changed {
    ($self:ident, $getter:ident, $setter:ident, $value:expr) => {{
        let next = $value;
        if *$self.as_ref().$getter() != next {
            $self.as_mut().$setter(next);
        }
    }};
}

pub struct ServerStatsRust {
    server_ready: bool,
    cpu_total: f32,
    cpu_temp_c: f64,
    cpu_core_count: i32,
    tps: f64,
    mspt: f64,
    player_count: i32,
    mem_process_rss: f64,
    mem_system_used: f64,
    mem_system_total: f64,
    uptime_secs: f64,
    pumpkin_version: QString,
    cpu_per_core: QList<f64>,
    tick_times_ms: QList<f64>,
    worlds_size_bytes: f64,
    disk_free: f64,
    disk_total: f64,
    net_in_bps: f64,
    net_out_bps: f64,
    java_address: QString,
    bedrock_address: QString,
    tick_budget_ms: f64,
    theme_preference: i32,
    worlds: QList<QVariant>,
    last_worlds: Vec<crate::WorldRow>,
}

impl Default for ServerStatsRust {
    fn default() -> Self {
        let theme_preference = match crate::gui_side().map(|side| side.theme) {
            Some(crate::ThemePreference::Light) => 0,
            Some(crate::ThemePreference::Dark) => 1,
            _ => -1,
        };

        Self {
            server_ready: false,
            cpu_total: 0.0,
            cpu_temp_c: -1.0,
            cpu_core_count: 0,
            tps: 0.0,
            mspt: 0.0,
            player_count: 0,
            mem_process_rss: 0.0,
            mem_system_used: 0.0,
            mem_system_total: 0.0,
            uptime_secs: 0.0,
            pumpkin_version: QString::default(),
            cpu_per_core: QList::<f64>::default(),
            tick_times_ms: QList::<f64>::default(),
            worlds_size_bytes: -1.0,
            disk_free: 0.0,
            disk_total: 0.0,
            net_in_bps: 0.0,
            net_out_bps: 0.0,
            java_address: QString::default(),
            bedrock_address: QString::default(),
            tick_budget_ms: 50.0,
            theme_preference,
            worlds: QList::<QVariant>::default(),
            last_worlds: Vec::new(),
        }
    }
}

/// Rebuilds a `QList<f64>` from a slice, but only when the contents actually changed.
fn list_changed(current: &QList<f64>, next: &[f64]) -> bool {
    // QList::len is isize; compare on the Rust side rather than casting the slice length.
    usize::try_from(current.len()).is_ok_and(|len| len != next.len())
        || current.iter().zip(next).any(|(old, new)| old != new)
}

fn to_qlist(values: impl IntoIterator<Item = f64>) -> QList<f64> {
    let mut list = QList::<f64>::default();
    for value in values {
        list.append(value);
    }
    list
}

impl qobject::ServerStats {
    // One straight-line assignment per property; splitting it up would only scatter the mapping.
    #[allow(clippy::too_many_lines)]
    pub fn refresh(mut self: Pin<&mut Self>) {
        let Some(side) = crate::gui_side() else {
            return;
        };
        let snapshot = side.snapshot.load();

        set_if_changed!(self, server_ready, set_server_ready, snapshot.server_ready);
        set_if_changed!(self, cpu_total, set_cpu_total, snapshot.cpu_total);
        set_if_changed!(
            self,
            cpu_temp_c,
            set_cpu_temp_c,
            snapshot.cpu_temp_c.map_or(-1.0, f64::from)
        );
        set_if_changed!(
            self,
            cpu_core_count,
            set_cpu_core_count,
            i32::try_from(snapshot.cpu_per_core.len()).unwrap_or(0)
        );
        set_if_changed!(self, tps, set_tps, snapshot.tps);
        set_if_changed!(self, mspt, set_mspt, snapshot.mspt);
        set_if_changed!(
            self,
            player_count,
            set_player_count,
            i32::try_from(
                snapshot
                    .players
                    .iter()
                    .filter(|player| player.online)
                    .count()
            )
            .unwrap_or(i32::MAX)
        );

        // Qt has no 64-bit unsigned property type; bytes go over as f64, which stays exact well
        // past any plausible memory or disk size.
        set_if_changed!(
            self,
            mem_process_rss,
            set_mem_process_rss,
            snapshot.mem_process_rss as f64
        );
        set_if_changed!(
            self,
            mem_system_used,
            set_mem_system_used,
            snapshot.mem_system_used as f64
        );
        set_if_changed!(
            self,
            mem_system_total,
            set_mem_system_total,
            snapshot.mem_system_total as f64
        );
        set_if_changed!(
            self,
            uptime_secs,
            set_uptime_secs,
            snapshot.uptime_secs as f64
        );
        set_if_changed!(
            self,
            worlds_size_bytes,
            set_worlds_size_bytes,
            // -1 marks "not scanned yet" so QML can show a placeholder instead of a bogus 0 B.
            snapshot.worlds_size_bytes.map_or(-1.0, |size| size as f64)
        );
        set_if_changed!(self, disk_free, set_disk_free, snapshot.disk_free as f64);
        set_if_changed!(self, disk_total, set_disk_total, snapshot.disk_total as f64);
        set_if_changed!(self, net_in_bps, set_net_in_bps, snapshot.net_in_bps as f64);
        set_if_changed!(
            self,
            net_out_bps,
            set_net_out_bps,
            snapshot.net_out_bps as f64
        );

        set_if_changed!(
            self,
            pumpkin_version,
            set_pumpkin_version,
            QString::from(&snapshot.meta.pumpkin_version)
        );
        set_if_changed!(
            self,
            java_address,
            set_java_address,
            QString::from(&snapshot.meta.java_address)
        );
        set_if_changed!(
            self,
            bedrock_address,
            set_bedrock_address,
            QString::from(&snapshot.meta.bedrock_address)
        );
        set_if_changed!(
            self,
            tick_budget_ms,
            set_tick_budget_ms,
            if snapshot.meta.tick_budget_ms > 0.0 {
                snapshot.meta.tick_budget_ms
            } else {
                50.0
            }
        );

        let cores: Vec<f64> = snapshot
            .cpu_per_core
            .iter()
            .map(|usage| f64::from(*usage))
            .collect();
        if list_changed(self.as_ref().cpu_per_core(), &cores) {
            self.as_mut().set_cpu_per_core(to_qlist(cores));
        }

        let ticks: Vec<f64> = snapshot
            .tick_times_nanos
            .iter()
            .map(|nanos| *nanos as f64 / 1_000_000.0)
            .collect();
        if list_changed(self.as_ref().tick_times_ms(), &ticks) {
            self.as_mut().set_tick_times_ms(to_qlist(ticks));
        }

        if self.as_ref().rust().last_worlds != snapshot.worlds {
            let rows = snapshot.worlds.iter().map(world_to_variant).fold(
                QList::<QVariant>::default(),
                |mut list, row| {
                    list.append(row);
                    list
                },
            );
            self.as_mut()
                .rust_mut()
                .last_worlds
                .clone_from(&snapshot.worlds);
            self.as_mut().set_worlds(rows);
        }
    }
}

fn world_to_variant(world: &crate::WorldRow) -> QVariant {
    let mut map = QMap::<QMapPair_QString_QVariant>::default();

    map.insert(
        QString::from("name"),
        QVariant::from(&QString::from(&world.name)),
    );
    map.insert(
        QString::from("dimension"),
        QVariant::from(&QString::from(&world.dimension)),
    );
    map.insert(
        QString::from("chunks"),
        QVariant::from(&i32::try_from(world.loaded_chunks).unwrap_or(i32::MAX)),
    );
    map.insert(
        QString::from("entities"),
        QVariant::from(&i32::try_from(world.entities).unwrap_or(i32::MAX)),
    );
    map.insert(
        QString::from("players"),
        QVariant::from(&i32::try_from(world.players).unwrap_or(i32::MAX)),
    );
    map.insert(
        QString::from("timeOfDay"),
        QVariant::from(&(world.time_of_day as f64)),
    );
    map.insert(
        QString::from("weather"),
        QVariant::from(&QString::from(&world.weather)),
    );
    map.insert(
        QString::from("size"),
        QVariant::from(&world.size_bytes.map_or(-1.0, |size| size as f64)),
    );

    QVariant::from(&map)
}
