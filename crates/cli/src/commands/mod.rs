mod alias;
mod bin;
mod clean;
mod completions;
pub mod debug;
mod install;
mod install_all;
mod list;
mod list_remote;
mod migrate;
mod outdated;
mod pin;
pub mod plugin;
mod regen;
mod run;
mod setup;
mod unalias;
mod uninstall;
mod upgrade;

pub use alias::*;
pub use bin::*;
pub use clean::*;
pub use completions::*;
pub use install::*;
pub use install_all::*;
pub use list::*;
pub use list_remote::*;
pub use migrate::*;
pub use outdated::*;
pub use pin::*;
pub use regen::*;
pub use run::*;
pub use setup::*;
pub use unalias::*;
pub use uninstall::*;
pub use upgrade::*;
