mod backend;
mod chrome;
mod root;
mod screens;
mod shared;
mod shells;

pub(crate) use screens::mediatek::MediatekView;
pub(crate) use screens::settings::SettingsView;
pub use root::Root;