use syn::{ExprParen, TypePath, spanned::Spanned};

#[derive(Copy, Clone)]
pub(crate) struct AliasType<'a> {
    path: &'a TypePath,
}


impl<'a> AliasType<'a> {
    pub fn new(path: &'a TypePath) -> Self {
        Self  { path }
    }

    pub fn type_path(&self) -> &'a TypePath {
        self.path
    }

    // pub fn as_expr_path(&self) -> Result<ExprPath, syn::Error> {
    //     // let mut expr_path = self.path.clone();
    //     // Ok(if let Some(segment) = expr_path.path.segments.last_mut() {
    //     //     match &mut segment.arguments {
    //     //         syn::PathArguments::None => {},
    //     //         syn::PathArguments::AngleBracketed(arg) => {
    //     //             arg.colon2_token = Some(PathSep::default())
    //     //         },
    //     //         syn::PathArguments::Parenthesized(arg) => {
    //     //             return Err(syn::Error::new(
    //     //                 arg.span(),
    //     //                 "cannot use parenthesized type as alias type",
    //     //             ))
    //     //         },
    //     //     } 
    //     // })
    //     todo!()
    // }

}







