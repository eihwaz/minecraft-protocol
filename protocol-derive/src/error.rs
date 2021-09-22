use syn::Error as SynError;

/// Possible errors while deriving.
#[derive(Debug)]
pub(crate) enum DeriveInputParserError {
    /// Derive attribute must be placed on a structure or enum.
    UnsupportedData,
    /// Data fields must be named.
    UnnamedDataFields,
    /// Possible errors while parsing attributes.
    AttributeError { attribute_error: AttributeError },
}

/// Possible errors while parsing attributes.
#[derive(Debug)]
pub(crate) enum AttributeError {
    /// Failed to parse field meta due incorrect syntax.
    BadAttributeSyntax { syn_error: SynError },
    /// Unsupported field attribute type.
    UnsupportedAttribute,
    /// Field meta has wrong value type.
    /// For example an int was expected, but a string was supplied.
    AttributeWrongValueType,
}

impl From<AttributeError> for DeriveInputParserError {
    fn from(attribute_error: AttributeError) -> Self {
        DeriveInputParserError::AttributeError { attribute_error }
    }
}

impl From<SynError> for DeriveInputParserError {
    fn from(syn_error: SynError) -> Self {
        DeriveInputParserError::AttributeError {
            attribute_error: AttributeError::BadAttributeSyntax { syn_error },
        }
    }
}

impl From<SynError> for AttributeError {
    fn from(syn_error: SynError) -> Self {
        AttributeError::BadAttributeSyntax { syn_error }
    }
}
