//! Module list for every page and endpoint. Each `pub mod name;` line tells Rust to
//! include the file `name.rs` from this folder. A file that is not listed here is
//! not compiled at all. `main.rs` then wires each page's handler function to a URL.
//! Files in this list: app_api (the Android app's JSON), blog, catalog, comments
//! (guestbook), data (garden charts), gallery, garden_camera (camera tools),
//! index (home page), login, plant_actions (add/edit/delete plants),
//! plant_listing (Quick Search) and widgets (small JSON helpers for the pages).
pub mod app_api;
pub mod blog;
pub mod catalog;
pub mod comments;
pub mod data;
pub mod gallery;
pub mod garden_camera;
pub mod index;
pub mod login;
pub mod plant_actions;
pub mod plant_listing;
pub mod widgets;
