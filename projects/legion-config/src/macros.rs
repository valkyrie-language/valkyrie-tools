#[macro_export]
macro_rules! bind_writer {
    ($writer:ident, $class:ident) => {
        struct $writer<'i> {
            ptr: &'i mut $class,
        }
        impl<'de> serde::Deserialize<'de> for $class {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let mut out = $class::default();
                let writer = $writer { ptr: &mut out };
                deserializer.deserialize_any(writer)?;
                Ok(out)
            }
            fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let writer = $writer { ptr: place };
                deserializer.deserialize_any(writer)?;
                Ok(())
            }
        }
    };
}
