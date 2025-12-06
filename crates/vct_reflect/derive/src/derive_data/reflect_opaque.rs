use crate::TypeAttributes;
use syn::{
    Attribute, Generics, PathSegment, Ident, Path, 
    Token, parenthesized, parse::ParseStream, token::Paren
};

/// A struct used to define a simple reflection-opaque types (including primitives).
pub(crate) struct ReflectOpaqueParser {
    pub attrs: TypeAttributes,
    pub custom_path: Option<Path>,
    pub type_ident: Ident,
    pub type_path: Path,
    pub generics: Generics,
}

impl ReflectOpaqueParser {
    pub fn parse(input: ParseStream) -> syn::Result<Self> {
        let origin_span = input.span();
        // For outer document comments.
        let origin_attrs = input.call(Attribute::parse_outer)?;

        let (custom_path, custom_name) = Self::parse_custom_path(input)?;

        let type_path = Path::parse_mod_style(input)?;

        let type_ident = type_path.segments.last().unwrap().ident.clone();

        let mut generics = input.parse::<Generics>()?;
        generics.where_clause = input.parse()?;

        let custom_path = if let Some(mut path) = custom_path {
            let name = PathSegment::from(custom_name.unwrap_or_else(|| type_ident.clone()));
            path.segments.push(name);
            Some(path)
        } else {
            None
        };

        // Parse outer document comments.
        let mut attrs = TypeAttributes::parse_attrs(origin_attrs.as_slice())?;
        // Parse inner attributes.
        if input.peek(Paren) {
            let content;
            parenthesized!(content in input);
            attrs.parse_stream(&content)?;
        }
        attrs.is_opaque = Some(origin_span);
        attrs.validity()?;

        Ok(Self {
            attrs,
            custom_path,
            type_ident,
            type_path,
            generics,
        })
    }

    fn parse_custom_path(input: ParseStream) -> syn::Result<(Option<Path>, Option<Ident>)> {
        if input.peek(Paren) {
            let path;
            parenthesized!(path in input);
            input.parse::<Token![in]>()?;
            if input.peek(Token![::]) {
                return Err(input.error("did not expect a leading double colon (`::`)"));
            }
            let path = Path::parse_mod_style(input)?;
            if path.segments.is_empty() {
                return Err(input.error("expected a path"))
            }

            if !input.peek(Token![as]) {
                return Ok((Some(path), None));
            }

            input.parse::<Token![as]>()?;
            let name: Ident = input.parse()?;
            Ok((Some(path), Some(name)))
            
        } else {
            Ok((None, None))
        }
    }

}

