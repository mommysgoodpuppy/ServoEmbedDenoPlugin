#![feature(register_tool)]
#![allow(warnings)]

use deno_core::*;
use deno_core::op2;
use error::AnyError;
use std::any::Any;
use std::borrow::Cow;
use std::ffi::c_void;

use serde::{Serialize, Deserialize};

use servo::base::id::TopLevelBrowsingContextId;
use servo::compositing::windowing::EmbedderCoordinates;
use servo::compositing::windowing::{AnimationState, EmbedderMethods, WindowMethods};
use servo::embedder_traits::EventLoopWaker;
use servo::servo_config::opts;
use servo::servo_url::ServoUrl;
use servo::InitializedServo;
use servo::Servo;


use servo::net::protocols::ProtocolRegistry;
use servo::webrender_api::units::{DeviceIntRect, DevicePixel};
use servo::webrender_traits::RenderingContext;

use euclid::{default::Box2D, Point2D, Rect, Scale, Size2D};

use std::rc::Rc;

struct ServoInstance<Window: WindowMethods + 'static> {
    servo: InitializedServo<Window>,
    browser_id: TopLevelBrowsingContextId,
}

static mut SERVO_INSTANCE: Option<ServoInstance<DummyWindow>> = None;




#[op2(fast)]
fn op_initialize_servo() -> Result<bool, AnyError> {
    // Initialize Servo
    let path = std::env::current_dir().unwrap().join("servo/resources");
    let path = path.to_str().unwrap().to_string();
    opts::set_options(opts::default_opts());

    let embedder = Box::new(DummyEmbedderMethods);
    let window = Rc::new(DummyWindow);
    let user_agent = None; // Use default user agent
    let composite_target = servo::compositing::CompositeTarget::Window;

    let mut servo = Servo::new(embedder, window, user_agent, composite_target);

    let url = ServoUrl::parse("about:blank").unwrap();
    let browser_id = TopLevelBrowsingContextId::new();

    servo.servo.handle_events(vec![servo::compositing::windowing::EmbedderEvent::NewWebView(url, browser_id)]);

    #[derive(Serialize, Deserialize)]
    struct InitializationResult {
        success: bool,
        message: String,
    }

    let success = true;
    let message = "Servo initialized successfully".to_string();

    // Return tuple with primitive types
    Ok(success)
}


struct DummyEventLoopWaker;

impl EventLoopWaker for DummyEventLoopWaker {
    fn wake(&self) {}

    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(DummyEventLoopWaker)
    }
}

struct DummyWindow;

impl WindowMethods for DummyWindow {
    fn get_coordinates(&self) -> EmbedderCoordinates {
        EmbedderCoordinates {
            hidpi_factor: Scale::new(1.0),
            screen: Size2D::new(1920, 1080),
            screen_avail: Size2D::new(1920, 1040),
            framebuffer: Size2D::new(800, 600),
            window: (Size2D::new(800, 600), Point2D::new(0, 0)),
            // Use two Point2D values for DeviceIntRect::new
            viewport: DeviceIntRect::new(Point2D::new(0, 0), Point2D::new(800, 600)),
        }
    }

    fn set_animation_state(&self, _state: AnimationState) {}

    fn rendering_context(&self) -> RenderingContext {
        unimplemented!("Dummy implementation doesn't provide a rendering context")
    }

    // Implement other required methods...
}

struct DummyEmbedderMethods;

impl EmbedderMethods for DummyEmbedderMethods {
    fn create_event_loop_waker(&mut self) -> Box<dyn EventLoopWaker> {
        Box::new(DummyEventLoopWaker)
    }

    fn get_user_agent_string(&self) -> Option<String> {
        Some("Servo/Deno Embedder (like Firefox/Gecko)".to_string())
    }

    fn get_version_string(&self) -> Option<String> {
        Some("0.1.0".to_string())
    }

    fn get_protocol_handlers(&self) -> ProtocolRegistry {
        ProtocolRegistry::default()
    }
}

#[no_mangle]
pub fn deno_plugin_init() -> Extension {
    // Call `op_initialize_servo` and store the resulting `OpDecl`
    const DECL: OpDecl = op_initialize_servo();

    // Build the extension with the op
    Extension {
        name: "servo_deno_plugin",
        ops: std::borrow::Cow::Borrowed(&[DECL]),  // Use the DECL constant
        ..Default::default()
    }
}
