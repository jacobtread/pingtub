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
}
static DATA: &[u8] = &[255u8; 512 * 512 * 4];

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

        Self { source, texture }
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
        self.texture.draw(0, 0, 512, 512, false);
    }
}

impl VideoTickSource for TuberSource {
    fn video_tick(&mut self, seconds: f32) {}
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
