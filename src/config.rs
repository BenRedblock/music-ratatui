pub struct Config {
    pub ytdl_libs: String,
    pub ytdl_output: String,
}

impl Config {
    pub fn new() -> Self {
        Config {
            ytdl_libs: String::from("libs"),
            ytdl_output: String::from("output"),
        }
    }
}
