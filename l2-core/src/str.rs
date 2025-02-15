pub trait StringTrim {
    fn trim_all(self) -> String;
}

pub trait Trim {
    fn trim_all(&self) -> &str;
}

impl StringTrim for String {
    ///this function is needed to trim also null bytes from a String
    /// # Example
    ///
    ///```
    /// use l2_core::str::StringTrim;
    /// let mut s = " ggg \0\0\0".to_string();
    /// s = s.trim_all();
    ///
    /// assert_eq!(s, "ggg");
    /// ```
    fn trim_all(self) -> String {
        String::from(self.trim_matches(|c: char| c.is_ascii_whitespace() || c == '\0'))
    }
}

impl Trim for str {
    ///this function is needed to trim also null bytes from a str slice
    /// # Example
    ///
    ///```
    /// use l2_core::str::Trim;
    /// let mut s:&'static str = " ggg \0\0\0";
    /// s = s.trim_all();
    ///
    /// assert_eq!(s, "ggg");
    /// ```
    fn trim_all(&self) -> &str {
        self.trim_matches(|c: char| c.is_ascii_whitespace() || c == '\0')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_trim() {
        let mut s = " ggg \0\0\0".to_string();
        s = s.trim_all();

        assert_eq!(s, "ggg");
    }

    #[test]
    fn test_str_trim() {
        let mut s: &'static str = " ggg \0\0\0";
        s = s.trim_all();

        assert_eq!(s, "ggg");
    }
}