use clack_host::prelude::*;

mod diva_stub {
    use clack_plugin::plugin::descriptor::{PluginDescriptor, StaticPluginDescriptor};
    use clack_plugin::prelude::*;
    use std::ffi::CStr;

    pub struct DivaPluginStub;

    pub struct DivaPluginStubAudioProcessor<'a> {
        shared: &'a DivaPluginStubShared<'a>,
    }
    pub struct DivaPluginStubShared<'a> {
        host: HostHandle<'a>,
    }

    impl<'a> PluginShared<'a> for DivaPluginStubShared<'a> {
        fn new(host: HostHandle<'a>) -> Result<Self, PluginError> {
            Ok(Self { host })
        }
    }

    impl Plugin for DivaPluginStub {
        type AudioProcessor<'a> = DivaPluginStubAudioProcessor<'a>;
        type Shared<'a> = DivaPluginStubShared<'a>;
        type MainThread<'a> = ();

        fn get_descriptor() -> Box<dyn PluginDescriptor> {
            use clack_plugin::plugin::descriptor::features::*;

            Box::new(StaticPluginDescriptor {
                id: CStr::from_bytes_with_nul(b"com.u-he.diva\0").unwrap(),
                name: CStr::from_bytes_with_nul(b"Diva\0").unwrap(),
                features: Some(&[SYNTHESIZER, STEREO]),
                ..Default::default()
            })
        }
    }

    impl<'a> PluginAudioProcessor<'a, DivaPluginStubShared<'a>, ()>
        for DivaPluginStubAudioProcessor<'a>
    {
        fn activate(
            _host: HostAudioThreadHandle<'a>,
            _main_thread: &mut (),
            shared: &'a DivaPluginStubShared<'a>,
            _audio_config: AudioConfiguration,
        ) -> Result<Self, PluginError> {
            Ok(Self { shared })
        }

        fn process(
            &mut self,
            _process: Process,
            mut audio: Audio,
            _events: Events,
        ) -> Result<ProcessStatus, PluginError> {
            self.shared.host.request_callback();

            for event in _events.input {
                _events.output.try_push(event).unwrap();
            }

            let mut output_channels = audio.output_port(0).unwrap().channels().unwrap();
            let output_buf = output_channels.as_f32_mut().unwrap().iter_mut();

            for channel in output_buf {
                for (input, output) in [42.0f32, 69.0, 21.0, 34.5].iter().zip(channel.iter_mut()) {
                    *output = *input;
                }
            }
            Ok(ProcessStatus::Sleep)
        }
    }

    clack_export_entry!(SinglePluginEntry<DivaPluginStub>);
    #[allow(unused)] // This is only used in doctests
    pub static DIVA_STUB_ENTRY: EntryDescriptor = clap_entry;
}

pub fn get_working_instance<H: Host, FS, FH>(
    shared: FS,
    main_thread: FH,
) -> Result<PluginInstance<H>, Box<dyn std::error::Error>>
where
    FS: for<'b> FnOnce(&'b ()) -> <H as Host>::Shared<'b>,
    FH: for<'b> FnOnce(&'b <H as Host>::Shared<'b>) -> <H as Host>::MainThread<'b>,
{
    let host_info = HostInfo::new("Legit Studio", "Legit Ltd.", "https://example.com", "4.3.2")?;

    let bundle = unsafe {
        PluginBundle::load_from_raw(
            &diva_stub::DIVA_STUB_ENTRY,
            "/home/user/.clap/u-he/libdiva.so",
        )?
    };

    let plugin_descriptor = bundle
        .get_plugin_factory()
        .unwrap()
        .plugin_descriptors()
        // We're assuming this specific plugin is in this bundle for this example.
        // A real host would store all descriptors in a list and show them to the user.
        .find(|d| d.id().unwrap().to_bytes() == b"com.u-he.diva")
        .unwrap();

    let plugin_instance = PluginInstance::<H>::new(
        shared,
        main_thread,
        &bundle,
        plugin_descriptor.id().unwrap(),
        &host_info,
    )?;

    Ok(plugin_instance)
}
