pub enum ContentType {
    File,
    Directory,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Directory => "directory",
        }
    }
}
