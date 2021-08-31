pub mod cache;
pub mod model;

// I compared using walkdir to just using fs to visit every dir and it is
// significantly faster on the network drive.

// TODO
// - split this module into load/refresh and build
// - Filter out Music and Audio Music Apps (blocklist of bad directories)
// - We only want mp3 files for now (allowlist of extensions)
// - All files should have a unique ID
// - Serialize a struct with the filename and the ID
// - Define Category (map of albums, map of  artists)
// - Define Artist - map of albums, list of tracks (sort!)
// - Detect discs
// - Define Album - list of tracks (sort!), map of discs
// - build a structure of Map<name, Catgeory>
//   - For each track: category is _category of 'main'
//   - Walk back from file -> disc? -> album -> artist?
//   - disc depends on regex
//   - artist depends on whether there is another part
