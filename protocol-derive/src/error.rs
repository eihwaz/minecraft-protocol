use proc_macro2::Ident;
use syn::Error as SynError;

/// Possible errors while parsing AST.
#[derive(Debug)]
pub(crate) enum ParseError<'a> {
    /// Parser expects a struct.
    NotStruct {
        name: &'a Ident,
    },
    /// Fields must be named.
    UnnamedFields,
    FieldError {
        field_error: FieldError,
    },
}

/// Possible errors while parsing field.
#[derive(Debug)]
pub(crate) enum FieldError {
    /// Failed to parse field attribute.
    BadAttributes { syn_error: SynError },
    /// Unsupported field attributes.
    NonListAttributes,
    /// Attribute value must be string.
    AttributeValueNotString,
    /// Attribute value must be integer.
    AttributeValueNotInteger,
}

impl From<FieldError> for ParseError<'_> {
    fn from(field_error: FieldError) -> Self {
        ParseError::FieldError { field_error }
    }
}

impl From<SynError> for ParseError<'_> {
    fn from(syn_error: SynError) -> Self {
        ParseError::FieldError {
            field_error: FieldError::BadAttributes { syn_error },
        }
    }
}

impl From<SynError> for FieldError {
    fn from(syn_error: SynError) -> Self {
        FieldError::BadAttributes { syn_error }
    }
}
