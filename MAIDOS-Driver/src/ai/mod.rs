//! AI module — hardware identification and driver recommendation
//!
//! Uses built-in PCI/USB vendor databases and heuristic classification
//! to identify unknown devices and recommend suitable drivers.

pub mod driver_recommender;
pub mod hardware_identifier;
