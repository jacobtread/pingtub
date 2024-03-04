use std::{
    io::{BufReader, Cursor},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Sample, Stream,
};
use obs_wrapper::{
    // Macro for registering modules
    graphics::{GraphicsColorFormat, GraphicsTexture},
    obs_register_module,
    obs_string,
    obs_sys,
    prelude::*,
    source::*,
};

/// Module that provides the [TuberSource]
struct TuberModule {
    /// Underlying module context
    ctx: ModuleContext,
}

impl Module for TuberModule {
    fn new(ctx: ModuleContext) -> Self {
        Self { ctx }
    }

    fn get_ctx(&self) -> &ModuleContext {
        &self.ctx
    }

    fn load(&mut self, load_context: &mut LoadContext) -> bool {
        // Create the source
        let source = load_context
            .create_source_builder::<TuberSource>()
            // Since GetNameSource is implemented, this method needs to be called to
            // enable it.
            .enable_get_name()
            .enable_update()
            .enable_video_render()
            .enable_video_tick()
            .enable_get_width()
            .enable_get_height()
            .build();

        // Tell OBS about the source so that it will show it.
        load_context.register_source(source);

        // Nothing could have gone wrong, so return true.
        true
    }

    fn description() -> ObsString {
        obs_string!("Png-tuber OBS integration")
    }

    fn name() -> ObsString {
        obs_string!("Ping-tub")
    }

    fn author() -> ObsString {
        obs_string!("Jacobtread")
    }
}

/// Input source that render the png-tuber model
struct TuberSource {
    source: SourceContext,
    texture: GraphicsTexture,

    buffer: RgbaImage,
    buffer_speaking: RgbaImage,

    buffer_index: i8,
    ticks: usize,

    speaking: Arc<AtomicBool>,

    stream: Stream,
}

static DATA: &[u8] = &[255u8; 512 * 512 * 4];
static TEST_IMAGE_DATA: &[u8] = include_bytes!("../test/jacobtread.png");
use image::{io::Reader as ImageReader, EncodableLayout, ImageBuffer, Rgba, RgbaImage};
const THRESHOLD: f32 = 0.1; // Adjust the threshold as needed

impl Sourceable for TuberSource {
    // Unique source ID
    fn get_id() -> ObsString {
        obs_string!("tuber_source")
    }

    // Source is a video input source
    fn get_type() -> SourceType {
        SourceType::INPUT
    }

    fn create(create: &mut CreatableSourceContext<Self>, source: SourceContext) -> Self {
        let mut texture = GraphicsTexture::new(512, 512, GraphicsColorFormat::RGBA);

        texture.set_image(DATA, 512 * 4, false);

        let img = ImageReader::new(Cursor::new(TEST_IMAGE_DATA))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();
        let img = img.resize(512, 512, image::imageops::FilterType::Nearest);

        let buffer_speaking = img.to_rgba8();
        let img = img.clone().brighten(-50);
        let buffer = img.to_rgba8();

        let mut texture =
            GraphicsTexture::new(img.width(), img.height(), GraphicsColorFormat::RGBA);

        let speaking = Arc::new(AtomicBool::new(false));

        let host = cpal::default_host();
        let input_device = host.default_input_device().unwrap();
        let input_config = input_device.default_input_config().unwrap().into();

        let speaking_used = speaking.clone();

        let stream = input_device
            .build_input_stream(
                &input_config,
                move |data: &[f32], _| {
                    // Calculate amplitude (root mean square)
                    let amplitude = data.iter().fold(0.0, |sum, &sample| sum + sample * sample);
                    let amplitude = (amplitude / data.len() as f32).sqrt();

                    // Check if amplitude exceeds threshold
                    let is_speaking = amplitude > THRESHOLD;

                    // Update speaking status
                    speaking_used.store(is_speaking, Ordering::SeqCst);
                },
                |err| {},
                None,
            )
            .unwrap();

        stream.play().unwrap();

        Self {
            source,
            texture,
            speaking,
            buffer,
            buffer_index: -1,
            ticks: 0,
            buffer_speaking,
            stream,
        }
    }
}

impl GetWidthSource for TuberSource {
    fn get_width(&mut self) -> u32 {
        512
    }
}

impl GetHeightSource for TuberSource {
    fn get_height(&mut self) -> u32 {
        512
    }
}

impl VideoRenderSource for TuberSource {
    fn video_render(&mut self, context: &mut GlobalContext, render: &mut VideoRenderContext) {
        self.texture.draw(0, 0, 0, 0, false);
    }
}

impl VideoTickSource for TuberSource {
    fn video_tick(&mut self, seconds: f32) {
        let speaking = self.speaking.load(std::sync::atomic::Ordering::SeqCst);

        if self.buffer_index != 1 && speaking {
            self.texture.set_image(
                self.buffer_speaking.as_bytes(),
                self.buffer_speaking.width() * 4,
                false,
            );
            self.buffer_index = 1;
        } else if self.buffer_index != 0 && !speaking {
            self.texture
                .set_image(self.buffer.as_bytes(), self.buffer.width() * 4, false);
            self.buffer_index = 0;
        }
    }
}

impl UpdateSource for TuberSource {
    fn update(&mut self, settings: &mut DataObj, context: &mut GlobalContext) {}
}

impl GetNameSource for TuberSource {
    fn get_name() -> ObsString {
        obs_string!("Ping-Tuber Source")
    }
}

obs_register_module!(TuberModule);
