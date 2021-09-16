use syn::Error as SynError;

/// Possible errors while deriving.
#[derive(Debug)]
pub(crate) enum DeriveInputParserError {
    /// Derive attribute must be placed on a structure or enum.
    UnsupportedData,
    /// Data fields must be named.
    UnnamedDataFields,
    FieldError {
        field_error: FieldError,
    },
}

/// Possible errors while parsing field.
#[derive(Debug)]
pub(crate) enum FieldError {
    /// Failed to parse field meta due incorrect syntax.
    BadAttributeSyntax { syn_error: SynError },
    /// Unsupported field attribute type.
    UnsupportedAttribute,
    /// Field meta has wrong value type.
    /// For example an int was expected, but a string was supplied.
    AttributeWrongValueType,
}

impl From<FieldError> for DeriveInputParserError {
    fn from(field_error: FieldError) -> Self {
        DeriveInputParserError::FieldError { field_error }
    }
}

impl From<SynError> for DeriveInputParserError {
    fn from(syn_error: SynError) -> Self {
        DeriveInputParserError::FieldError {
            field_error: FieldError::BadAttributeSyntax { syn_error },
        }
    }
}

impl From<SynError> for FieldError {
    fn from(syn_error: SynError) -> Self {
        FieldError::BadAttributeSyntax { syn_error }
    }
}
