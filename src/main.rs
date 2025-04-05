use std::{cell::RefCell, rc::Rc};

use app::AppState;
use wasm_bindgen::prelude::*;
use web_sys::{Event, HtmlCanvasElement, WebGl2RenderingContext, WebGlProgram, WebGlShader};

mod app;
mod polyline;
mod scribe;

fn main() -> Result<(), JsValue> {
    wasm_log::init(wasm_log::Config::default());

    log::info!("Hello, world!");

    let canvas = document().get_element_by_id("canvas").unwrap();
    let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>()?;

    let context = canvas
        .get_context("webgl2")?
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()?;

    let app_state = Rc::new(RefCell::new(AppState::new(&context)?));

    let onresize = {
        let canvas = canvas.clone();
        let context = context.clone();
        let app_state = app_state.clone();
        Closure::<dyn FnMut(_)>::new(move |_event: Event| {
            app_state.borrow_mut().on_resize(&canvas, &context);
        })
    };
    window().add_event_listener_with_callback("resize", onresize.as_ref().unchecked_ref())?;
    onresize.forget();

    app_state.borrow_mut().on_resize(&canvas, &context);

    let f = Rc::new(RefCell::<Option<Closure<dyn FnMut()>>>::new(None));
    let g = f.clone();

    {
        let context = context.clone();
        *g.borrow_mut() = Some(Closure::<dyn FnMut()>::new(move || {
            app_state.borrow_mut().draw(&context);

            request_animation_frame(f.borrow().as_ref().unwrap());
        }));
    }

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

pub fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

pub fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

pub fn body() -> web_sys::HtmlElement {
    document().body().expect("document should have a body")
}

pub fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

pub fn reinterpret_cast_slice<S, T>(input: &[S]) -> &[T] {
    let length_in_bytes = input.len() * std::mem::size_of::<S>();
    let desired_length = length_in_bytes / std::mem::size_of::<T>();
    unsafe { std::slice::from_raw_parts(input.as_ptr() as *const T, desired_length) }
}
