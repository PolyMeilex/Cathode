use std::marker::PhantomData;

use pulse::{
    context::introspect::{self, CardInfo, ClientInfo},
    volume::{ChannelVolumes, Volume},
};

pub use data::*;

pub struct Introspector<'a> {
    introspector: introspect::Introspector,
    pd: PhantomData<&'a ()>,
}

impl<'a> From<introspect::Introspector> for Introspector<'a> {
    fn from(introspector: introspect::Introspector) -> Self {
        Self {
            introspector,
            pd: PhantomData,
        }
    }
}

macro_rules! list_callback {
    ($tx:ident, $cb:expr) => {{
        let mut list = Some(Vec::new());
        let mut tx = Some($tx);

        move |res| match res {
            pulse::callbacks::ListResult::Item(item) => {
                let item = ($cb)(item);

                if let Some(list) = list.as_mut() {
                    list.push(item);
                }
            }
            pulse::callbacks::ListResult::End => {
                if let (Some(tx), Some(list)) = (tx.take(), list.take()) {
                    tx.send(Ok(list)).unwrap();
                }
            }
            pulse::callbacks::ListResult::Error => {
                if let Some(tx) = tx.take() {
                    tx.send(Err(())).unwrap();
                }
            }
        }
    }};
}

impl<'a> Introspector<'a> {
    /// Gets the card list.
    pub async fn card_list(&self) -> Result<Vec<String>, ()> {
        let (tx, rx) = futures::channel::oneshot::channel();

        self.introspector
            .get_card_info_list(list_callback!(tx, |item: &CardInfo| {
                item.name.as_ref().unwrap().to_string()
            }));

        rx.await.unwrap()
    }

    /// Gets the client list.
    pub async fn client_list(&self) -> Result<Vec<String>, ()> {
        let (tx, rx) = futures::channel::oneshot::channel();

        self.introspector
            .get_client_info_list(list_callback!(tx, |item: &ClientInfo| {
                item.name.as_ref().unwrap().to_string()
            }));

        rx.await.unwrap()
    }

    /// Gets the sink list.
    pub async fn sink_list(&self) -> Result<Vec<data::SinkInfo>, ()> {
        let (tx, rx) = futures::channel::oneshot::channel();

        self.introspector
            .get_sink_info_list(list_callback!(tx, data::SinkInfo::from));

        rx.await.unwrap()
    }

    /// Gets information about a sink by its index.
    pub async fn sink(&self, id: u32) -> Result<data::SinkInfo, ()> {
        let (tx, rx) = futures::channel::oneshot::channel();

        self.introspector
            .get_sink_info_by_index(id, list_callback!(tx, data::SinkInfo::from));

        let res = rx.await.unwrap();

        res.map(|mut list| list.remove(0))
    }

    /// Sets the volume of a sink input stream.
    ///
    /// The callback accepts a `bool`, which indicates success.
    pub async fn set_sink_volume(&mut self, id: u32, v: f64) -> bool {
        let mut vol = ChannelVolumes::default();

        let v = (Volume::NORMAL.0 as f64 * v / 100.0).round() as u32;
        vol.set(2, Volume(v));

        let (tx, rx) = futures::channel::oneshot::channel::<bool>();

        let mut tx = Some(tx);
        self.introspector.set_sink_volume_by_index(
            id,
            &vol,
            Some(Box::new(move |success| {
                if let Some(tx) = tx.take() {
                    tx.send(success).unwrap();
                }
            })),
        );

        rx.await.unwrap()
    }

    /// Gets the sink input list.
    pub async fn sink_input_list(&self) -> Result<Vec<data::SinkInputInfo>, ()> {
        let (tx, rx) = futures::channel::oneshot::channel();

        self.introspector
            .get_sink_input_info_list(list_callback!(tx, data::SinkInputInfo::from));

        rx.await.unwrap()
    }

    /// Gets some information about a sink input by its index.
    pub async fn sink_input(&self, id: u32) -> Result<data::SinkInputInfo, ()> {
        let (tx, rx) = futures::channel::oneshot::channel();

        self.introspector
            .get_sink_input_info(id, list_callback!(tx, data::SinkInputInfo::from));

        let res = rx.await.unwrap();

        res.map(|mut list| list.remove(0))
    }

    /// Sets the volume of a sink input stream.
    ///
    /// The callback accepts a `bool`, which indicates success.
    pub async fn set_sink_input_volume(&mut self, id: u32, v: f64) -> bool {
        let mut vol = ChannelVolumes::default();

        let v = (Volume::NORMAL.0 as f64 * v / 100.0).round() as u32;
        vol.set(2, Volume(v));

        let (tx, rx) = futures::channel::oneshot::channel::<bool>();

        let mut tx = Some(tx);
        self.introspector.set_sink_input_volume(
            id,
            &vol,
            Some(Box::new(move |success| {
                if let Some(tx) = tx.take() {
                    tx.send(success).unwrap();
                }
            })),
        );

        rx.await.unwrap()
    }
}

mod data {
    use pulse::{
        channelmap,
        context::introspect,
        def::{self, PortAvailable},
        format,
        proplist::Proplist,
        sample,
        time::MicroSeconds,
        volume::{ChannelVolumes, Volume},
    };

    /*
     * Sink info
     */

    /// Stores information about a specific port of a sink.
    ///
    /// Please note that this structure can be extended as part of evolutionary API updates at any time
    /// in any new release.
    #[derive(Debug)]
    pub struct SinkPortInfo {
        /// Name of the sink.
        pub name: Option<String>,
        /// Description of this sink.
        pub description: Option<String>,
        /// The higher this value is, the more useful this port is as a default.
        pub priority: u32,
        /// A flag indicating availability status of this port.
        pub available: PortAvailable,
    }

    impl<'a> From<&'a introspect::SinkPortInfo<'a>> for SinkPortInfo {
        fn from(item: &'a introspect::SinkPortInfo<'a>) -> Self {
            SinkPortInfo {
                name: item.name.as_ref().map(|cow| cow.to_string()),
                description: item.description.as_ref().map(|cow| cow.to_string()),
                priority: item.priority,
                available: item.available,
            }
        }
    }

    impl<'a> From<&'a Box<introspect::SinkPortInfo<'a>>> for SinkPortInfo {
        fn from(item: &'a Box<introspect::SinkPortInfo<'a>>) -> Self {
            item.as_ref().into()
        }
    }

    /// Stores information about sinks.
    ///
    /// Please note that this structure can be extended as part of evolutionary API updates at any time
    /// in any new release.
    #[derive(Debug)]
    pub struct SinkInfo {
        /// Name of the sink.
        pub name: Option<String>,
        /// Index of the sink.
        pub index: u32,
        /// Description of this sink.
        pub description: Option<String>,
        /// Sample spec of this sink.
        pub sample_spec: sample::Spec,
        /// Channel map.
        pub channel_map: channelmap::Map,
        /// Index of the owning module of this sink, or `None` if is invalid.
        pub owner_module: Option<u32>,
        /// Volume of the sink.
        pub volume: ChannelVolumes,
        /// Mute switch of the sink.
        pub mute: bool,
        /// Index of the monitor source connected to this sink.
        pub monitor_source: u32,
        /// The name of the monitor source.
        pub monitor_source_name: Option<String>,
        /// Length of queued audio in the output buffer.
        pub latency: MicroSeconds,
        /// Driver name.
        pub driver: Option<String>,
        /// Flags.
        pub flags: def::SinkFlagSet,
        /// Property list.
        pub proplist: Proplist,
        /// The latency this device has been configured to.
        pub configured_latency: MicroSeconds,
        /// Some kind of “base” volume that refers to unamplified/unattenuated volume in the context of
        /// the output device.
        pub base_volume: Volume,
        /// State.
        pub state: def::SinkState,
        /// Number of volume steps for sinks which do not support arbitrary volumes.
        pub n_volume_steps: u32,
        /// Card index, or `None` if invalid.
        pub card: Option<u32>,
        /// Set of available ports.
        pub ports: Vec<SinkPortInfo>,
        // Pointer to active port in the set, or None.
        pub active_port: Option<SinkPortInfo>,
        /// Set of formats supported by the sink.
        pub formats: Vec<format::Info>,
    }

    impl<'a> From<&'a introspect::SinkInfo<'a>> for SinkInfo {
        fn from(item: &'a introspect::SinkInfo<'a>) -> Self {
            SinkInfo {
                name: item.name.as_ref().map(|cow| cow.to_string()),
                index: item.index,
                description: item.description.as_ref().map(|cow| cow.to_string()),
                sample_spec: item.sample_spec,
                channel_map: item.channel_map,
                owner_module: item.owner_module,
                volume: item.volume,
                mute: item.mute,
                monitor_source: item.monitor_source,
                monitor_source_name: item.monitor_source_name.as_ref().map(|cow| cow.to_string()),
                latency: item.latency,
                driver: item.driver.as_ref().map(|cow| cow.to_string()),
                flags: item.flags,
                proplist: item.proplist.clone(),
                configured_latency: item.configured_latency,
                base_volume: item.base_volume,
                state: item.state,
                n_volume_steps: item.n_volume_steps,
                card: item.card,
                ports: item.ports.iter().map(From::from).collect(),
                active_port: item.active_port.as_ref().map(From::from),
                formats: item.formats.clone(),
            }
        }
    }

    /*
     * Sink input info
     */

    /// Stores information about sink inputs.
    ///
    /// Please note that this structure can be extended as part of evolutionary API updates at any time
    /// in any new release.
    #[derive(Debug)]
    pub struct SinkInputInfo {
        /// Index of the sink input.
        pub index: u32,
        /// Name of the sink input.
        pub name: Option<String>,
        /// Index of the module this sink input belongs to, or `None` when it does not belong to any
        /// module.
        pub owner_module: Option<u32>,
        /// Index of the client this sink input belongs to, or invalid when it does not belong to any
        /// client.
        pub client: Option<u32>,
        /// Index of the connected sink.
        pub sink: u32,
        /// The sample specification of the sink input.
        pub sample_spec: sample::Spec,
        /// Channel map.
        pub channel_map: channelmap::Map,
        /// The volume of this sink input.
        pub volume: ChannelVolumes,
        /// Latency due to buffering in sink input, see [`TimingInfo`](crate::def::TimingInfo) for
        /// details.
        pub buffer_usec: MicroSeconds,
        /// Latency of the sink device, see [`TimingInfo`](crate::def::TimingInfo) for details.
        pub sink_usec: MicroSeconds,
        /// The resampling method used by this sink input.
        pub resample_method: Option<String>,
        /// Driver name.
        pub driver: Option<String>,
        /// Stream muted.
        pub mute: bool,
        /// Property list.
        pub proplist: Proplist,
        /// Stream corked.
        pub corked: bool,
        /// Stream has volume. If not set, then the meaning of this struct’s volume member is
        /// unspecified.
        pub has_volume: bool,
        /// The volume can be set. If not set, the volume can still change even though clients can’t
        /// control the volume.
        pub volume_writable: bool,
        /// Stream format information.
        pub format: format::Info,
    }

    impl<'a> From<&'a introspect::SinkInputInfo<'a>> for SinkInputInfo {
        fn from(item: &'a introspect::SinkInputInfo<'a>) -> Self {
            // description: item.description.as_ref().map(|cow| cow.to_string()),
            Self {
                index: item.index,
                name: item.name.as_ref().map(|cow| cow.to_string()),
                owner_module: item.owner_module,
                client: item.client,
                sink: item.sink,
                sample_spec: item.sample_spec,
                channel_map: item.channel_map,
                volume: item.volume,
                buffer_usec: item.buffer_usec,
                sink_usec: item.sink_usec,
                resample_method: item.resample_method.as_ref().map(|cow| cow.to_string()),
                driver: item.driver.as_ref().map(|cow| cow.to_string()),
                mute: item.mute,
                proplist: item.proplist.clone(),
                corked: item.corked,
                has_volume: item.has_volume,
                volume_writable: item.volume_writable,
                format: item.format.clone(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::Context;
    use pulse::{context::FlagSet, proplist::Proplist};

    #[test]
    fn lists() {
        let props = Proplist::new().unwrap();
        let mut context = Context::new_with_proplist("Test", &props);

        glib::MainContext::default().block_on(async move {
            context.connect(None, FlagSet::NOFLAGS).await.unwrap();

            let introspect = context.introspect();
            let res = introspect.card_list().await.unwrap();
            dbg!(res.len());
            let res = introspect.client_list().await.unwrap();
            dbg!(res.len());
            let res = introspect.sink_list().await.unwrap();
            dbg!(res.len());
            let res = introspect.sink_input_list().await.unwrap();
            dbg!(res.len());
        });
    }

    #[test]
    fn item() {
        let props = Proplist::new().unwrap();
        let mut context = Context::new_with_proplist("Test", &props);

        glib::MainContext::default().block_on(async move {
            context.connect(None, FlagSet::NOFLAGS).await.unwrap();

            let introspect = context.introspect();
            // let res = introspect.card_list().await.unwrap();
            // dbg!(res);
            // let res = introspect.client_list().await.unwrap();
            // dbg!(res);
            // let res = introspect.sink_list().await.unwrap();
            // dbg!(res);
            let res = introspect.sink_input(199).await.unwrap();
            dbg!(res);
        });
    }
}
