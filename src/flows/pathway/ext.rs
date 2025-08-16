use bevy::{
    prelude::*,
    tasks::block_on,
};

use crate::{
    backend::AppBackend,
    prelude::*,
};

/// [`App`] extension trait for [`Pathway`]-related methods
pub trait AppPathwayExt {
    /// Initializes the [`Pathway`] and its associated [`Backend`]
    fn insert_pathway<P>(&mut self, backend: P::Backend) -> &mut Self
    where
        P: Pathway<Backend: for<'a> Backend<P::Key<'a>> + Send + Sync + 'static>;

    /// Initializes the [`Pathway`] and its associated [`Backend`] using default values
    fn init_pathway<P>(&mut self) -> &mut Self
    where
        P: Pathway<Backend: FromWorld + for<'a> Backend<P::Key<'a>> + Send + Sync + 'static>;
}

impl AppPathwayExt for App {
    fn insert_pathway<P>(&mut self, backend: P::Backend) -> &mut Self
    where
        P: Pathway<Backend: for<'a> Backend<P::Key<'a>> + Send + Sync + 'static>,
    {
        self.insert_backend(backend)
    }

    fn init_pathway<P>(&mut self) -> &mut Self
    where
        P: Pathway<Backend: FromWorld + for<'a> Backend<P::Key<'a>> + Send + Sync + 'static>,
    {
        self.init_backend::<P::Backend, _>()
    }
}

/// [`World`] extension trait for [`Pathway`]-related methods
pub trait WorldPathwayExt {
    /// Capture the [`World`] state with the given [`Pathway`]
    fn capture<P>(&mut self, pathway: &P) -> P::Capture
    where
        P: Pathway<Capture: CaptureInput<P>>;

    /// Capture the [`World`] state with the given [`Pathway`] and builder
    fn capture_with<P>(
        &mut self,
        pathway: &P,
        builder: <P::Capture as CaptureInput<P>>::Builder,
    ) -> P::Capture
    where
        P: Pathway<Capture: CaptureInput<P>>;

    /// Capture the [`World`] state with the given [`Pathway`] and save it to persistent storage
    ///
    /// # Errors
    /// - If the [`Format`] fails to serialize the capture
    /// - If the [`Backend`] fails to save the capture
    fn save<P>(&mut self, pathway: &P) -> Result<(), Error>
    where
        P: Pathway<Capture: CaptureInput<P> + CaptureSerialize>;

    /// Applies the given capture to the [`World`] state
    ///
    /// # Errors
    /// - If the capture fails to apply
    fn apply<P>(&mut self, pathway: &P, capture: P::Capture) -> Result<P::Capture, Error>
    where
        P: Pathway<Capture: CaptureOutput<P>>;

    /// Loads a capture from persistent storage and applies it to [`World`] state
    ///
    /// # Errors
    /// - If the [`Backend`] fails tot load the capture
    /// - If the [`Format`] fails to deserialize the capture
    /// - If the capture fails to apply
    fn load<P>(&mut self, pathway: &P) -> Result<P::Capture, Error>
    where
        P: Pathway<Capture: CaptureOutput<P> + CaptureDeserialize>;
}

impl WorldPathwayExt for World {
    fn capture<P>(&mut self, pathway: &P) -> P::Capture
    where
        P: Pathway<Capture: CaptureInput<P>>,
    {
        self.capture_with(pathway, P::Capture::builder(pathway, self))
    }

    fn capture_with<P>(
        &mut self,
        pathway: &P,
        builder: <P::Capture as CaptureInput<P>>::Builder,
    ) -> P::Capture
    where
        P: Pathway<Capture: CaptureInput<P>>,
    {
        type Builder<P> = <<P as Pathway>::Capture as CaptureInput<P>>::Builder;
        let label = pathway.capture(self).intern();
        let out;

        if let Some(mut flow) = self
            .get_resource_mut::<Flows<Builder<P>>>()
            .and_then(|mut r| r.take_flow(label))
        {
            flow.initialize(self);
            out = flow.run(builder, self);

            self.resource_mut::<Flows<Builder<P>>>()
                .insert_flow(label, flow);
        } else {
            // No flows registered, return the input as-is
            out = builder;
        }

        P::Capture::build(out, pathway, self)
    }

    fn save<P>(&mut self, pathway: &P) -> Result<(), Error>
    where
        P: Pathway<Capture: CaptureInput<P> + CaptureSerialize>,
    {
        let cap = self.capture(pathway);
        let backend = &self.resource::<AppBackend<P::Backend>>().0;

        let seed = cap.value(self);

        block_on(backend.save::<P::Format, _>(pathway.key(), &seed))
    }

    fn apply<P>(&mut self, pathway: &P, capture: P::Capture) -> Result<P::Capture, Error>
    where
        P: Pathway<Capture: CaptureOutput<P>>,
    {
        type Builder<P> = <<P as Pathway>::Capture as CaptureOutput<P>>::Builder;

        let seed = P::Capture::builder(capture, pathway, self);
        let label = pathway.apply(self).intern();

        let out;

        if let Some(mut flow) = self
            .get_resource_mut::<Flows<Builder<P>>>()
            .and_then(|mut r| r.take_flow(label))
        {
            flow.initialize(self);
            out = flow.run(seed, self);

            self.resource_mut::<Flows<Builder<P>>>()
                .insert_flow(label, flow);
        } else {
            // No flows registered, return the input as-is
            out = seed;
        }

        P::Capture::build(out, pathway, self)
            .map_err(|_| Error::custom("Failed to build capture from applier"))
    }

    fn load<P>(&mut self, pathway: &P) -> Result<P::Capture, Error>
    where
        P: Pathway<Capture: CaptureOutput<P> + CaptureDeserialize>,
    {
        let backend = &self.resource::<AppBackend<P::Backend>>().0;
        let seed = <P::Capture as CaptureDeserialize>::seed(self);
        let capture = block_on(backend.load::<P::Format, _, _>(pathway.key(), seed))?;

        self.apply(pathway, capture)
    }
}
