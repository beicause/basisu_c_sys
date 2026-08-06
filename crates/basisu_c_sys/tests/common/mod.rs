// Use real path for snapshots, as symlink makes `insta` panic on windows.
pub const SNAPSHOT_PATH: &str = "../../../basisu_c_sys_asset_files/tests/snapshots/";
