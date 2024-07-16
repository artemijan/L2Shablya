pub trait StringTrim {
    fn trim_all(self) -> String;
}

pub trait StrTrim {
    fn trim_all(&self) -> &str;
}

impl StringTrim for String {
    ///this function is needed to trim also null bytes from a String
    /// # Example
    /// ```
    /// let s = " ggg \0\0\0\".to_string();
    /// s.trim_all()
    /// _____________
    /// $ ggg
    /// ```
    fn trim_all(self) -> String {
        String::from(self.trim_matches(|c: char| c.is_ascii_whitespace() || c == '\0'))
    }
}

impl StrTrim for str {
    ///this function is needed to trim also null bytes from a str slice
    /// # Example
    /// ```
    /// let s:&str = " ggg \0\0\0\";
    /// s.trim_all()
    /// _____________
    /// $ ggg
    /// ```
    fn trim_all(&self) -> &str {
        self.trim_matches(|c: char| c.is_ascii_whitespace() || c == '\0')
    }
}
