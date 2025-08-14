//! Middleware for [`Format`](crate::prelude::Format), allowing you to easily
//! add features like compression or encryption.

#[cfg(feature = "brotli")]
mod brotli {
    use std::marker::PhantomData;

    use brotli::enc::BrotliEncoderParams;

    use crate::prelude::*;

    /// Brotli middleware for compressing your data after serializing
    ///
    /// # Example
    /// ```rust
    /// # use bevy_save::prelude::*;
    /// struct MyPipeline;
    ///
    /// impl Pipeline for MyPipeline {
    ///     type Backend = DefaultBackend;
    ///     /// This will emit brotli-compressed MessagePack
    ///     type Format = Brotli<DefaultFormat>;
    ///     type Key<'a> = &'a str;
    ///
    ///     fn key(&self) -> Self::Key<'_> {
    ///         "my_pipeline"
    ///     }
    /// }
    pub struct Brotli<F>(PhantomData<F>);

    impl<F> Default for Brotli<F> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }

    impl<F: Format> Format for Brotli<F> {
        fn extension() -> &'static str {
            // TODO: Should be `format!("{}.br", F::extension())`
            ".br"
        }

        fn serialize<W: std::io::prelude::Write, T: serde::Serialize>(
            writer: W,
            value: &T,
        ) -> Result<(), crate::Error> {
            let params = BrotliEncoderParams::default();
            let writer = brotli::CompressorWriter::with_params(writer, 4096, &params);
            F::serialize(writer, value)
        }

        fn deserialize<
            R: std::io::prelude::Read,
            S: for<'de> serde::de::DeserializeSeed<'de, Value = T>,
            T,
        >(
            reader: R,
            seed: S,
        ) -> Result<T, crate::Error> {
            let reader = brotli::Decompressor::new(reader, 4096);
            F::deserialize(reader, seed)
        }
    }
}

#[cfg(feature = "brotli")]
pub use brotli::Brotli;
