use raylib::{ffi::CSSPalette, prelude::*};
use serde::{Deserialize, Serialize};
use std::env::current_dir;
use std::{env::set_current_dir, process::Command};

//================================================================

const SCREEN_SIZE: (f32, f32) = (1024.0, 800.0);
const ICON_EXIT: &[u8] = include_bytes!("../asset/exit.png");
const ICON_LOGO: &[u8] = include_bytes!("../asset/logo.png");
const FONT_DATA: &[u8] = include_bytes!("../asset/font.ttf");
const MAIN_COLOR: Color = Color::new(33, 150, 243, 255);

#[derive(Serialize, Deserialize)]
struct Meta {
    list: Vec<String>,
    address: String,
}

impl Meta {
    const FILE_PATH: &str = "launcher.json";

    fn new() -> anyhow::Result<Self> {
        Ok(serde_json::from_str(&std::fs::read_to_string(
            Self::FILE_PATH,
        )?)?)
    }
}

fn main() {
    if let Err(error) = run() {
        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("Error")
            .set_description(format!("{error}"))
            .show();
    }
}

fn run() -> anyhow::Result<()> {
    let (mut handle, thread) = raylib::init()
        .size(SCREEN_SIZE.0 as i32, SCREEN_SIZE.1 as i32)
        .title("Launcher")
        .log_level(TraceLogLevel::LOG_NONE)
        .undecorated()
        .transparent()
        .build();

    let meta = Meta::new()?;
    let exit = load_texture_raw(&mut handle, &thread, ICON_EXIT.to_vec())?;
    let logo = load_texture_raw(&mut handle, &thread, ICON_LOGO.to_vec())?;
    let font = get_font(&mut handle, &thread)?;
    let mut game_list = get_game_list(&mut handle, &thread, &meta)?;
    let mut game_name = String::new();
    let mut game_name_fail = 0.0;

    handle.set_target_fps(60);

    while !handle.window_should_close() {
        let mut draw = handle.begin_drawing(&thread);
        let mouse = draw.get_mouse_position();
        let click = draw.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);

        draw.clear_background(Color::TRANSPARENT);

        //================
        // header
        //================

        draw.draw_rectangle_rec(
            Rectangle::new(0.0, 0.0, SCREEN_SIZE.0 - 64.0, SCREEN_SIZE.1),
            MAIN_COLOR,
        );
        draw.draw_rectangle_rec(
            Rectangle::new(SCREEN_SIZE.0 - 64.0, 64.0, 64.0, SCREEN_SIZE.1 - 64.0),
            MAIN_COLOR,
        );
        draw.draw_triangle(
            Vector2::new(SCREEN_SIZE.0 - 64.0, 0.0),
            Vector2::new(SCREEN_SIZE.0 - 64.0, 64.0),
            Vector2::new(SCREEN_SIZE.0, 64.0),
            MAIN_COLOR,
        );

        let hover = point_in_triangle(
            mouse,
            Vector2::new(SCREEN_SIZE.0 - 56.0, 0.0),
            Vector2::new(SCREEN_SIZE.0, 56.0),
            Vector2::new(SCREEN_SIZE.0, 0.0),
        );
        let color_a = if hover { Color::BLACK } else { Color::WHITE };
        let color_b = if hover { Color::WHITE } else { Color::BLACK };

        draw.draw_triangle(
            Vector2::new(SCREEN_SIZE.0 - 56.0, 0.0),
            Vector2::new(SCREEN_SIZE.0, 56.0),
            Vector2::new(SCREEN_SIZE.0, 0.0),
            color_a,
        );

        draw.draw_texture(&exit, (SCREEN_SIZE.0 - 32.0) as i32, 0, color_b);
        draw.draw_texture(&logo, 16, 8, Color::WHITE);

        //================
        // footer
        //================

        draw_text_edit(
            &mut draw,
            &font,
            &mut game_name,
            Color::WHITE.lerp(Color::RED, game_name_fail),
        );

        game_name_fail = (game_name_fail - draw.get_frame_time()).max(0.0);

        //================
        // game
        //================

        for (i, game) in game_list.iter_mut().enumerate() {
            if game.draw(&mut draw, &meta, i as f32) {
                if game_name.is_empty() {
                    game_name_fail = 1.0;
                } else {
                    let path = current_dir()?;
                    set_current_dir(&game.path)?;
                    Command::new("bash")
                        .env("CNDJ_NAME", &game_name)
                        .env("CNDJ_ADDRESS", &meta.address)
                        .args(["launcher_play.sh"])
                        .spawn()?;
                    set_current_dir(path)?;
                }
            }
        }

        if hover && click {
            break;
        }
    }

    Ok(())
}

fn draw_text_edit(draw: &mut RaylibDrawHandle, font: &Font, name: &mut String, color: Color) {
    if let Some(character) = draw.get_char_pressed() {
        name.push(character);
    } else if draw.is_key_pressed(KeyboardKey::KEY_BACKSPACE)
        || draw.is_key_pressed_repeat(KeyboardKey::KEY_BACKSPACE)
    {
        name.pop();
    }

    let alpha = ((draw.get_time() as f32 * 4.0).sin() + 1.0) / 2.0;
    let shift = font.measure_text(name, 6.0 * 9.0, 0.0);

    draw.draw_text_ex(
        font,
        "Nombre",
        Vector2::new(16.0, SCREEN_SIZE.1 - 64.0),
        6.0 * 9.0,
        0.0,
        color,
    );

    draw.draw_rectangle(
        168,
        SCREEN_SIZE.1 as i32 - 52,
        SCREEN_SIZE.0 as i32 - 168 - 16,
        40,
        color,
    );
    draw.draw_rectangle(
        168 + 6 + shift.x as i32,
        SCREEN_SIZE.1 as i32 - 48,
        4,
        32,
        Color::BLACK.alpha(alpha),
    );
    draw.draw_text_ex(
        font,
        name,
        Vector2::new(168.0 + 4.0, SCREEN_SIZE.1 - 64.0),
        6.0 * 9.0,
        0.0,
        Color::BLACK,
    );
}

fn ease_out(x: f32) -> f32 {
    if x < 0.5 {
        2.0 * x * x
    } else {
        1.0 - (-2.0 * x + 2.0).powf(2.0) / 2.0
    }
}

fn point_in_triangle(p: Vector2, a: Vector2, b: Vector2, c: Vector2) -> bool {
    let ab = Vector2::new(b.x - a.x, b.y - a.y).cross(Vector2::new(p.x - a.x, p.y - a.y));
    let bc = Vector2::new(c.x - b.x, c.y - b.y).cross(Vector2::new(p.x - b.x, p.y - b.y));
    let ca = Vector2::new(a.x - c.x, a.y - c.y).cross(Vector2::new(p.x - c.x, p.y - c.y));

    (ab >= 0.0 && bc >= 0.0 && ca >= 0.0) || (ab <= 0.0 && bc <= 0.0 && ca <= 0.0)
}

fn load_texture_raw(
    handle: &mut RaylibHandle,
    thread: &RaylibThread,
    data: Vec<u8>,
) -> anyhow::Result<Texture2D> {
    let image = Image::load_image_from_mem(".png", &data)?;
    let mut image = handle.load_texture_from_image(thread, &image)?;
    image.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_TRILINEAR);
    image.gen_texture_mipmaps();
    Ok(image)
}

fn load_texture(
    handle: &mut RaylibHandle,
    thread: &RaylibThread,
    path: &str,
) -> anyhow::Result<Texture2D> {
    let mut image = handle.load_texture(&thread, path)?;
    image.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_TRILINEAR);
    image.gen_texture_mipmaps();
    Ok(image)
}

fn get_font(handle: &mut RaylibHandle, thread: &RaylibThread) -> anyhow::Result<Font> {
    Ok(handle.load_font_from_memory(&thread, ".ttf", FONT_DATA, 6 * 9, None)?)
}

fn get_game_list(
    handle: &mut RaylibHandle,
    thread: &RaylibThread,
    meta: &Meta,
) -> anyhow::Result<Vec<Game>> {
    let mut list = Vec::new();

    for game in &meta.list {
        list.push(Game::new(
            game.to_string(),
            load_texture(handle, thread, &format!("{}/launcher_hero.png", game))?,
            load_texture(handle, thread, &format!("{}/launcher_logo.png", game))?,
        ));
    }

    Ok(list)
}

struct Game {
    path: String,
    hero: Texture2D,
    logo: Texture2D,
    hover: f32,
}

impl Game {
    fn new(path: String, hero: Texture2D, logo: Texture2D) -> Self {
        Self {
            path,
            hero,
            logo,
            hover: 0.0,
        }
    }

    fn draw(&mut self, draw: &mut RaylibDrawHandle, meta: &Meta, i: f32) -> bool {
        let mouse = draw.get_mouse_position();
        let click = draw.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        let frame = draw.get_frame_time();
        let screen_size = SCREEN_SIZE.1 - 64.0 * 2.0;

        let scale_hero = Vector2::new(self.hero.width as f32, screen_size / meta.list.len() as f32);
        let scale_logo = Vector2::new(self.logo.width as f32, self.logo.height as f32);
        let hover = ease_out(self.hover);
        let point = Vector2::new(0.0, 64.0 + scale_hero.y * i);
        let shape = Rectangle::new(point.x, point.y, scale_hero.x, scale_hero.y);
        let color = Color::WHITE.lerp(Color::BLACK, 0.5 - (hover * 0.5));
        let zoom_h = 1.0 - hover * 0.1;
        let zoom_l = 1.0 + hover * 0.1;

        if shape.check_collision_point_rec(mouse) {
            self.hover += frame * 4.0;
        } else {
            self.hover -= frame * 2.0;
        }

        self.hover = self.hover.clamp(0.0, 1.0);

        draw.draw_texture_pro(
            &self.hero,
            Rectangle::new(
                (scale_hero.x - scale_hero.x * zoom_h) * 0.5,
                (scale_hero.y - scale_hero.y * zoom_h) * 0.5
                    + (self.hero.height as f32 - scale_hero.y).max(0.0) * 0.5,
                scale_hero.x * zoom_h,
                scale_hero.y * zoom_h,
            ),
            shape,
            Vector2::ZERO,
            0.0,
            color,
        );
        draw.draw_texture_pro(
            &self.logo,
            Rectangle::new(0.0, 0.0, scale_logo.x, scale_logo.y),
            Rectangle::new(
                point.x + (scale_hero.x - scale_logo.x * zoom_l) * 0.5,
                point.y + (scale_hero.y - scale_logo.y * zoom_l) * 0.5,
                scale_logo.x * zoom_l,
                scale_logo.y * zoom_l,
            ),
            Vector2::ZERO,
            0.0,
            color,
        );

        shape.check_collision_point_rec(mouse) && click
    }
}
