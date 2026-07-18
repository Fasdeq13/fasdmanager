use gio::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DeviceEntry {
    pub display_name: String,
    pub mount_path: Option<PathBuf>,
    pub is_mounted: bool,
    pub can_eject: bool,
    pub volume: gio::Volume,
}

pub fn list_devices() -> Vec<DeviceEntry> {
    let monitor = gio::VolumeMonitor::get();
    let mut result = Vec::new();

    for volume in monitor.volumes() {
        let display_name = volume.name().to_string();
        let can_eject = volume.can_eject();

        let (mount_path, is_mounted) = match volume.get_mount() {
            Some(mount) => {
                let root = mount.root();
                (root.path().map(PathBuf::from), true)
            }
            None => (None, false),
        };

        result.push(DeviceEntry {
            display_name,
            mount_path,
            is_mounted,
            can_eject,
            volume,
        });
    }

    result
}

pub fn mount_volume(
    volume: &gio::Volume,
    callback: impl FnOnce(Result<(), glib::Error>) + 'static,
) {
    let mount_operation = gtk::MountOperation::new(None::<&gtk::Window>);
    volume.mount(
        gio::MountMountFlags::NONE,
        Some(&mount_operation),
        None::<&gio::Cancellable>,
        move |result| {
            let mapped = result.map(|_| ());
            callback(mapped);
        },
    );
}

pub fn eject_volume(
    volume: &gio::Volume,
    callback: impl FnOnce(Result<(), glib::Error>) + 'static,
) {
    volume.eject_with_operation(
        gio::MountUnmountFlags::NONE,
        None::<&gtk::MountOperation>,
        None::<&gio::Cancellable>,
        move |result| {
            let mapped = result.map(|_| ());
            callback(mapped);
        },
    );
}

pub fn watch_device_changes(on_change: impl Fn() + 'static) -> gio::VolumeMonitor {
    let monitor = gio::VolumeMonitor::get();
    let on_change = std::rc::Rc::new(on_change);

    let cb1 = on_change.clone();
    monitor.connect_volume_added(move |_monitor, _volume| {
        cb1();
    });

    let cb2 = on_change.clone();
    monitor.connect_volume_removed(move |_monitor, _volume| {
        cb2();
    });

    let cb3 = on_change.clone();
    monitor.connect_mount_added(move |_monitor, _mount| {
        cb3();
    });

    let cb4 = on_change;
    monitor.connect_mount_removed(move |_monitor, _mount| {
        cb4();
    });

    monitor
}
