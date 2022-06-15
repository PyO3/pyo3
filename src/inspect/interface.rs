//! Generates a Python interface file (.pyi) using the inspected elements.

use std::fmt::{Display, Formatter};
use libc::write;
use crate::inspect::classes::ClassInfo;
use crate::inspect::fields::{ArgumentInfo, ArgumentKind, FieldInfo, FieldKind};

/// Interface generator for a Python class.
///
/// Instances are created with [`InterfaceGenerator::new`].
/// The documentation is generated via the [`Display`] implementation.
pub struct InterfaceGenerator<'a> {
    info: ClassInfo<'a>
}

impl<'a> InterfaceGenerator<'a> {
    pub fn new(info: ClassInfo<'a>) -> Self {
        Self {
            info
        }
    }

    fn class_header(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // class ClassName(BaseClassName):

        write!(f, "class {}", self.info.name())?;
        if let Some(base) = self.info.base() {
            write!(f, "({})", base)?;
        }
        write!(f, ":")
    }

    fn field(field: &FieldInfo, f: &mut Formatter<'_>) -> std::fmt::Result {
        match field.kind {
            FieldKind::New => {
                write!(f, "    def __new__(cls")?;
                Self::arguments(field.arguments, true, f)?;
                write!(f, ") -> None")?;
            },
            FieldKind::Getter => {
                writeln!(f, "    @property")?;
                write!(f, "    def {}(self", field.name)?;
                Self::signature_end(field, false, f)?;
            },
            FieldKind::Setter => {
                writeln!(f, "    @{}.setter", field.name)?;
                write!(f, "    def {}(self", field.name)?;
                Self::signature_end(field, true, f)?;
            },
            FieldKind::Function => {
                write!(f, "    def {}(self", field.name)?;
                Self::signature_end(field, true, f)?;
            },
            FieldKind::ClassMethod => {
                writeln!(f, "    @classmethod")?;
                write!(f, "    def {}(cls", field.name)?;
                Self::signature_end(field, true, f)?;
            },
            FieldKind::ClassAttribute => {
                write!(f, "    {}", field.name)?;
                if let Some(output_type) = field.py_type {
                    write!(f, ": {}", (output_type)())?;
                }
                return writeln!(f, " = ...");
            },
            FieldKind::StaticMethod => {
                writeln!(f, "    @staticmethod")?;
                write!(f, "    def {}(", field.name)?;
                Self::signature_end(field, false, f)?;
            },
        };

        writeln!(f, ": ...")
    }

    fn signature_end(field: &FieldInfo, start_with_comma: bool, f: &mut Formatter<'_>) -> std::fmt::Result {
        // def whatever(self [THIS FUNCTION]

        Self::arguments(field.arguments, start_with_comma, f)?;
        if let Some(output_type) = field.py_type {
            write!(f, ") -> {}", (output_type)())
        } else {
            write!(f, ")")
        }
    }

    fn arguments(arguments: &[ArgumentInfo], start_with_comma: bool, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut add_comma = start_with_comma;
        let mut positional_only = true;
        let mut keyword_only = false;

        for argument in arguments {
            if add_comma {
                write!(f, ", ")?;
            }

            if positional_only && !matches!(argument.kind, ArgumentKind::Position) {
                // PEP570
                if add_comma {
                    write!(f, "/, ")?;
                }
                positional_only = false
            }

            if !keyword_only && matches!(argument.kind, ArgumentKind::Keyword) {
                // PEP3102
                write!(f, "*, ")?;
                keyword_only = true
            }

            match argument.kind {
                ArgumentKind::VarArg => write!(f, "*")?,
                ArgumentKind::KeywordArg => write!(f, "**")?,
                _ => {},
            };

            write!(f, "{}", argument.name)?;

            if let Some(py_type) = argument.py_type {
                write!(f, ": {}", (py_type)())?;
            }

            if argument.default_value {
                write!(f, " = ...")?;
            }

            add_comma = true;
        }

        Ok(())
    }
}

impl<'a> Display for InterfaceGenerator<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.class_header(f)?;

        if self.info.fields.is_empty() && self.info.class.fields.is_empty() {
            writeln!(f, " ...")?;
        } else {
            writeln!(f)?;
        }

        for field in self.info.fields() {
            Self::field(*field, f)?;
        }

        Ok(())
    }
}
