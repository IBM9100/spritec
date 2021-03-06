use neon::prelude::*;
use spritec::math::{Rgba, Rgb, Vec3, Mat4, Radians};
use spritec::query3d::{File, GeometryFilter, GeometryQuery};
use spritec::renderer::{
    FileQuery,
    Light,
    Camera,
    Outline,
    RenderCamera,
    RenderJob,
    RenderLights,
    RenderNode,
    RenderedImage,
    Size,
    ThreadRenderContext,
};
use spritec::scene::{LightType, CameraType};
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Returns the rendered sprite given parameters from JavaScript
fn render_sprite(mut cx: FunctionContext) -> JsResult<JsArrayBuffer> {
    // Arguments from JavaScript
    let path = cx.argument::<JsString>(0)?.value();
    let width = cx.argument::<JsNumber>(1)?.value() as u32;
    let height = cx.argument::<JsNumber>(2)?.value() as u32;

    // TODO: Change to return a class so we can reuse resources
    let mut ctx = ThreadRenderContext::new().expect("Unable to create ThreadRenderContext");
    let file = Arc::new(Mutex::new(File::open(Path::new(&path)).expect("Unable to open file")));
    let camera = RenderCamera::Camera(Arc::new(default_camera()));

    let job = RenderJob {
        scale: unsafe { NonZeroU32::new_unchecked(1) },
        root: RenderNode::RenderedImage(RenderedImage {
            size: Size {
                width: NonZeroU32::new(width).expect("Width is not a u32"),
                height: NonZeroU32::new(height).expect("Height is not a u32"),
            },
            background: Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            },
            camera,
            lights: RenderLights::Lights(Arc::new(vec![Arc::new(Light {
                data: Arc::new(LightType::Directional {
                    color: Rgb::white(),
                    intensity: 1.0,
                }),
                world_transform: Mat4::rotation_x((-60.0f32).to_radians()),
            })])),
            ambient_light: Rgb::white() * 0.5,
            geometry: FileQuery {
                query: GeometryQuery {
                    models: GeometryFilter::all_in_default_scene(),
                    animation: None,
                },
                file,
            },
            outline: Outline {
                thickness: 0.0,
                color: Rgba::black(),
            },
        }),
    };
    let image = job.execute(&mut ctx).expect("Sprite creation failed");

    let mut array_buffer = cx.array_buffer(image.width() * image.height() * 4)?;
    cx.borrow_mut(&mut array_buffer, |data| {
        let slice = data.as_mut_slice::<u8>();
        for (i, pixel) in image.pixels().enumerate() {
            slice[i * 4 + 0] = pixel[0];
            slice[i * 4 + 1] = pixel[1];
            slice[i * 4 + 2] = pixel[2];
            slice[i * 4 + 3] = pixel[3];
        }
    });
    Ok(array_buffer)
}

fn default_camera() -> Camera {
    let eye = Vec3 {x: 8.0, y: 8.0, z: 8.0};
    let target = Vec3::zero();

    let cam_type = CameraType::Perspective {
        name: None,
        aspect_ratio: 1.0,
        field_of_view_y: Radians::from_degrees(40.0),
        near_z: 0.1,
        far_z: Some(1000.0),
    };

    Camera {
        view: Mat4::look_at_rh(eye, target, Vec3::up()),
        projection: cam_type.to_projection(),
    }
}


register_module!(mut cx, {
    cx.export_function("render_sprite", render_sprite)?;
    Ok(())
});
