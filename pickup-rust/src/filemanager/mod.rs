pub mod cache;
pub mod collection;
pub mod list;
pub mod model;
pub mod options;
pub mod utils;

// I compared using walkdir to just using fs to visit every dir and it is
// significantly faster on the network drive.
