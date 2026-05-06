// src/components/mod.rs
pub mod ui;
pub mod vinculation;
pub mod home;
pub mod auth_bar;
pub mod create_coop;
pub mod gas_modal;
pub mod cooperatives;

// test of components
#[cfg(all(test, feature = "ssr"))]
mod home_test;

#[cfg(all(test, feature = "ssr"))]
mod auth_bar_test;

#[cfg(test)]
mod vinculation_test;

#[cfg(test)]
mod gas_modal_test;