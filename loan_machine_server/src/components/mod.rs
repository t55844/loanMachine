// src/components/mod.rs
pub mod ui;
pub mod vinculation;
pub mod home;
pub mod auth_bar;

// test of components
#[cfg(all(test, feature = "ssr"))]
mod home_test;