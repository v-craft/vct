use core::{
    any::type_name, fmt
};

/// Used to parse short_name from full_names
/// 
/// It's a parsing tool, instead of string container.
/// 
/// # Safety
/// - Must be a valid UTF-8 encoded string.
///   Otherwise, it may panic.
/// 
/// # Validity
/// - The wrong path will not panic, but the returned result is uncertain.
/// 
/// # Example
/// 
/// ```
/// use vct_utils::name::ShortName;
/// use std::string::ToString;
/// #
/// # mod foo {
/// #     pub mod bar {
/// #         pub struct Baz;
/// #     }
/// # }
/// // from Type
/// let short_name = ShortName::of::<foo::bar::Baz>();
/// assert_eq!(short_name.to_string(), "Baz");
/// // from &str
/// let short_name = ShortName::from("my::crate::Tools");
/// assert_eq!(short_name.to_string(), "Tools");
/// // use original to get full name
/// assert_eq!(short_name.original(), "my::crate::Tools");
/// // Can also use `std::fmt::Display` impl
/// ```
#[derive(Clone, Copy)]
pub struct ShortName<'a>(pub &'a str);

impl ShortName<'static> {
    /// Gets a shortened version of the name of the type `T`.
    #[inline]
    pub fn of<T: ?Sized>() -> Self {
        // TODO: Change to const fn if `type_name::<T>()` is stable const fn.
        Self(type_name::<T>())
    }
}

impl<'a> From<&'a str> for ShortName<'a> {
    #[inline]
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}

impl<'a> ShortName<'a> {
    /// Gets the original name before shortening.
    /// 
    /// use ToString::to_string to get short_name
    #[inline]
    pub const fn original(&self) -> &'a str {
        self.0
    }
}

impl <'a> fmt::Debug for ShortName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let full_name = self.0.trim();

        let mut index: usize = 0;
        let end_index = full_name.len();

        while index < end_index {
            // The remaining unresolved content
            let rest_of_str = &full_name[index..end_index];
            // ↑ Safety: index < end_index, valid UTF-8 encoded string
            // let rest_of_str = full_name.get(index..end_index).unwrap_or_default();

            if let Some(special_index) = rest_of_str.find(|c: char| {
                (c == ' ') ||
                (c == '<') ||
                (c == '>') ||
                (c == '(') ||
                (c == ')') ||
                (c == '[') ||
                (c == ']') ||
                (c == ',') ||
                (c == ';')
            }) {
                let segment = &rest_of_str[0..special_index];
                // ↑ Safety: special_index < end_index, valid UTF-8 encoded string
                // let segment = rest_of_str.get(0..special_index).unwrap_or_default();

                f.write_str(parse_type_name(segment))?;

                // Insert the special character
                let special_char = &rest_of_str[special_index..=special_index];
                // ↑ Safety: special_index < end_index, valid UTF-8 encoded string
                f.write_str(special_char)?;

                // When the `begin` of the slice `[begin...]` is at the end, 
                // it is safe and will return an empty slice.
                match special_char {
                    ">" | ")" | "]" if rest_of_str[special_index + 1..].starts_with("::") => {
                        f.write_str("::")?;
                        // Move the index past the "::"
                        index += special_index + 3;
                    }
                    // Move the index just past the special character
                    _ => index += special_index + 1,
                }

                // Skip consecutive spaces
                if special_char == " " {
                    // Safety: index <= end_index
                    // if index == end_index, starts_with(" ") will return false
                    while full_name[index..].starts_with(" ") {
                        index += 1;
                    }
                }

            } else {
                // If there are no special characters left, we're done!
                f.write_str(parse_type_name(rest_of_str))?;
                index = end_index;
            }
        }

        Ok(())
    }
}

impl<'a> fmt::Display for ShortName<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[inline(always)]
fn parse_type_name(string: &str) -> &str {
    let mut segments = string.rsplit("::");

    // Usually, the last string is type name.
    // But for Enums type, need the last two strings.
    let (last, second_last): (&str, Option<&str>) = (segments.next().unwrap(), segments.next());
    // ↑Safety: rsplit return at least one element (may be empty string)
    let Some(second_last) = second_last else {
        return last;
    };

    // for Enums type, enum_name and variant_name should both start with uppercase.
    if second_last.starts_with(char::is_uppercase) {
        let index = string.len() - last.len() - second_last.len() - 2;
        &string[index..]
    } else {
        last
    }
}

#[cfg(test)]
mod tests {
    use super::ShortName;
    use alloc::string::ToString;

    #[test]
    fn single() {
        assert_eq!(ShortName("test_trival").to_string(), "test_trival");
    }

    #[test]
    fn path_separated() {
        assert_eq!(
            ShortName("my_crate::make_fun").to_string(),
            "make_fun"
        );
    }

    #[test]
    fn tuple_type() {
        assert_eq!(
            ShortName("(String, u64)").to_string(),
            "(String, u64)"
        );
    }

    #[test]
    fn array_type() {
        assert_eq!(ShortName("[i32; 3]").to_string(), "[i32; 3]");
    }

    #[test]
    fn trivial_generics() {
        assert_eq!(ShortName("a<B>").to_string(), "a<B>");
    }

    #[test]
    fn multiple_type_parameters() {
        assert_eq!(ShortName("a<B, C>").to_string(), "a<B, C>");
    }

    #[test]
    fn enums() {
        assert_eq!(ShortName("Option::None").to_string(), "Option::None");
        assert_eq!(ShortName("Option::Some(2)").to_string(), "Option::Some(2)");
        assert_eq!(
            ShortName("test_enum::MyEnum::Foo").to_string(),
            "MyEnum::Foo"
        );
    }

    #[test]
    fn generics() {
        assert_eq!(
            ShortName("render::camera::camera::extract_cameras<render::camera::bundle::Camera3d>").to_string(),
            "extract_cameras<Camera3d>"
        );
    }

    #[test]
    fn nested_generics() {
        assert_eq!(
            ShortName("mad_science::do_mad_science<mad_science::Test<mad_science::Tube>, vct::TypeSystemAbuse>").to_string(),
            "do_mad_science<Test<Tube>, TypeSystemAbuse>"
        );
    }

    #[test]
    fn sub_path_after_closing_bracket() {
        assert_eq!(
            ShortName("asset::assets::Assets<scene::dynamic_scene::DynamicScene>::asset_event_system").to_string(),
            "Assets<DynamicScene>::asset_event_system"
        );
        assert_eq!(
            ShortName("(String, String)::default").to_string(),
            "(String, String)::default"
        );
        assert_eq!(
            ShortName("[i32; 16]::default").to_string(),
            "[i32; 16]::default"
        );
    }
}
