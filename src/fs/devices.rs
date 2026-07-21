use gio::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DeviceEntry {
    pub display_name: String,
    pub mount_path: Option<PathBuf>,
    pub is_mounted: bool,
    pub can_eject: bool,
    pub can_unmount: bool,
    pub volume: gio::Volume,
}

pub fn list_devices() -> Vec<DeviceEntry> {
    let monitor = gio::VolumeMonitor::get();
    let mut result = Vec::new();

    for volume in monitor.volumes() {
        let display_name = volume.name().to_string();
        let can_eject = volume.can_eject();

        let (mount_path, is_mounted, can_unmount) = match volume.get_mount() {
            Some(mount) => {
                let root = mount.root();
                (root.path().map(PathBuf::from), true, mount.can_unmount())
            }
            None => (None, false, false),
        };

        result.push(DeviceEntry {
            display_name,
            mount_path,
            is_mounted,
            can_eject,
            can_unmount,
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

#[derive(Debug, Clone)]
pub enum UnmountProgressEvent {
    Started,
    Flushing { bytes_left: i64 },
    Finished(Result<(), String>),
}

pub fn unmount_volume_with_progress(
    volume: &gio::Volume,
    on_progress: impl Fn(UnmountProgressEvent) + 'static,
) {
    let Some(mount) = volume.get_mount() else {
        on_progress(UnmountProgressEvent::Finished(Err(
            "volume is not mounted".to_string(),
        )));
        return;
    };

    let mount_operation = gtk::MountOperation::new(None::<&gtk::Window>);

    let on_progress_for_signal = std::rc::Rc::new(on_progress);
    let on_progress_for_signal_clone = on_progress_for_signal.clone();

    mount_operation.connect_show_unmount_progress(move |_op, _message, _time_left, bytes_left| {
        on_progress_for_signal_clone(UnmountProgressEvent::Flushing { bytes_left });
    });

    on_progress_for_signal(UnmountProgressEvent::Started);

    let on_progress_for_finish = on_progress_for_signal.clone();
    mount.unmount_with_operation(
        gio::MountUnmountFlags::NONE,
        Some(&mount_operation),
        None::<&gio::Cancellable>,
        move |result| {
            let mapped = result.map_err(|e| e.to_string());
            on_progress_for_finish(UnmountProgressEvent::Finished(mapped));
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
