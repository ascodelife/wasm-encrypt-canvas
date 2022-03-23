use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::HtmlElement;

struct CanvasAttributes<'a> {
    id: &'a str,
    width: u32,
    height: u32,
}
static CANVAS_ATTRIBUTES: CanvasAttributes = CanvasAttributes {
    id: "encrypt-canvas",
    width: 100,
    height: 100,
};

#[derive(Serialize, Deserialize, Debug)]
struct Position {
    x: f64,
    y: f64,
}
#[derive(Serialize, Deserialize, Debug)]
struct FontStyle {
    size: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct RenderString {
    cipher: String,
    position: Position,
    font_style: FontStyle,
}

#[derive(Serialize, Deserialize)]
pub struct Params {
    render_info: Vec<RenderString>,
    user_token: String,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
#[allow(unused_macros)]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub fn encrypt_canvas(params: &JsValue) -> Result<(), JsValue> {
    let Params {
        mut render_info,
        user_token,
    } = params.into_serde().unwrap();
    // 模拟解密过程
    decrypt_info(&mut render_info);
    // console_log!("render_info is {:?}", render_info);
    // 展示非付费用户能力
    let mut user = User::<FreePlan>::new(user_token, 0, render_info);
    user.render_as_canvas()?;
    user.render_as_img()?;
    // 展示付费用户能力
    user.fetch_vip_level();
    if user.get_vip_level() > 0 {
        let vip_user: User<VipPlan> = user.into();
        vip_user.render_as_div()?;
    }
    Ok(())
}

struct User<T> {
    user_token: String,
    vip_level: usize,
    info: Vec<RenderString>,
    _type: PhantomData<T>,
}

/// 免费用户只能将信息渲染为 canvas 或 img 这类不可复制和编辑的视图
trait Free {
    fn fetch_vip_level(&mut self);
    fn render_as_canvas(&self) -> Result<(), JsValue>;
    fn render_as_img(&self) -> Result<(), JsValue>;
}

/// 给付费用户额外提供将信息渲染为 dom 的能力
trait Vip: Free {
    fn render_as_div(&self) -> Result<(), JsValue>;
}

struct FreePlan;
struct VipPlan;

impl<T> User<T> {
    fn new(user_token: String, vip_level: usize, info: Vec<RenderString>) -> Self {
        Self {
            user_token,
            vip_level,
            info,
            _type: PhantomData::default(),
        }
    }
    fn get_vip_level(&self) -> usize {
        self.vip_level
    }
    fn set_vip_level(&mut self, vip_level: usize) {
        self.vip_level = vip_level
    }
}

impl<T> Free for User<T> {
    fn fetch_vip_level(&mut self) {
        // 省略若干通过 user_token 获取用户信息的异步请求
        let vip_level = 3;
        self.set_vip_level(vip_level);
    }

    fn render_as_canvas(&self) -> Result<(), JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();
        let canvas = document
            .create_element("canvas")?
            .dyn_into::<web_sys::HtmlCanvasElement>()?;

        canvas.set_id(CANVAS_ATTRIBUTES.id);
        canvas.set_width(CANVAS_ATTRIBUTES.width);
        canvas.set_height(CANVAS_ATTRIBUTES.height);

        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
        for item in &self.info {
            context.set_font(&format!("{}px serif", item.font_style.size).to_string());
            context.fill_text(&item.cipher, item.position.x, item.position.y)?;
        }

        let p = get_paragraph("不可复制的画布")?;
        body.append_child(&p)?;
        body.append_child(&canvas)?;

        Ok(())
    }

    fn render_as_img(&self) -> Result<(), JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        let canvas = match document.get_element_by_id(CANVAS_ATTRIBUTES.id) {
            Some(canvas) => canvas,
            None => return Err(JsValue::from_str("cannot find canvas")),
        }
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
        let data_url = canvas.to_data_url()?;

        let img = document
            .create_element("img")?
            .dyn_into::<web_sys::HtmlImageElement>()?;
        img.set_src(&data_url);
        img.set_width(CANVAS_ATTRIBUTES.width);
        img.set_height(CANVAS_ATTRIBUTES.height);

        let p = get_paragraph("不可复制的图像")?;
        body.append_child(&p)?;
        body.append_child(&img)?;

        Ok(())
    }
}

impl Vip for User<VipPlan> {
    fn render_as_div(&self) -> Result<(), JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        let div_warp = document
            .create_element("div")?
            .dyn_into::<web_sys::HtmlElement>()?;
        div_warp.style().set_property("position", "relative")?;
        div_warp
            .style()
            .set_property("width", &format!("{}px", CANVAS_ATTRIBUTES.width))?;
        div_warp
            .style()
            .set_property("height", &format!("{}px", CANVAS_ATTRIBUTES.height))?;

        for item in &self.info {
            let div = document
                .create_element("div")?
                .dyn_into::<web_sys::HtmlElement>()?;
            div.set_inner_text(&item.cipher);
            div.style().set_property("position", "absolute")?;
            div.style()
                .set_property("left", &format!("{}px", item.position.x))?;
            div.style()
                .set_property("top", &format!("{}px", item.position.y))?;
            div.style()
                .set_property("font-size", &format!("{}px", item.font_style.size))?;
            div_warp.append_child(&div)?;
        }

        let p = get_paragraph("VIP 专用：可复制的普通元素")?;
        body.append_child(&p)?;
        body.append_child(&div_warp)?;

        Ok(())
    }
}

impl From<User<FreePlan>> for User<VipPlan> {
    fn from(user: User<FreePlan>) -> Self {
        Self::new(user.user_token, user.vip_level, user.info)
    }
}

fn get_paragraph(text: &str) -> Result<HtmlElement, JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let p = document
        .create_element("p")?
        .dyn_into::<web_sys::HtmlElement>()?;
    p.set_inner_text(text);
    Ok(p)
}

/// 解密信息
fn decrypt_info(info: &mut Vec<RenderString>) {
    for item in info {
        item.cipher = decrypt(&item.cipher);
    }
}

/// 加密密钥
static KEY: u8 = 123;

#[allow(dead_code)]
/// 简单模拟一个字符串加密
fn encrypt(str: &String) -> String {
    let u: Vec<u8> = (str
        .to_owned()
        .into_bytes()
        .iter()
        .map(|char| char ^ KEY)
        .collect::<Vec<u8>>())
    .to_vec();
    String::from_utf8_lossy(&u).to_string()
}

/// 简单模拟一个字符串解密
fn decrypt(str: &String) -> String {
    let u: Vec<u8> = (str
        .to_owned()
        .into_bytes()
        .iter()
        .map(|char| char ^ KEY)
        .collect::<Vec<u8>>())
    .to_vec();
    String::from_utf8_lossy(&u).to_string()
}

#[test]
fn test() {
    let str = String::from("bar");
    let cipher = encrypt(&str);
    let plaintext = decrypt(&cipher);
    println!("{:?}", cipher);
    assert_eq!(str, plaintext);
}
