//! A Painter example.

use crochet::{AppHolder, Column, Cx, DruidAppData, Padding, Painter, Row, SizedBox, TextBox};
use druid::{AppLauncher, Env, PaintCtx, PlatformError, Widget, WindowDesc};

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder).window_size((450.0, 450.0));
    let data = Default::default();
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
}

fn ui_builder() -> impl Widget<DruidAppData> {
    let mut app = App::new();
    AppHolder::new(move |cx| app.run(cx))
}

struct App {
    first_string: String,
    second_string: String,
}

impl App {
    pub fn new() -> Self {
        App {
            first_string: "Hello".to_string(),
            second_string: "World".to_string(),
        }
    }

    pub fn run(&mut self, cx: &mut Cx) {
        Column::new().build(cx, |cx| {
            Padding::new().uniform(5.0).build(cx, |cx| {
                Row::new().build(cx, |cx| {
                    if let Some(new_first_string) = TextBox::new(&self.first_string).build(cx) {
                        self.first_string = new_first_string;
                    }
                    if let Some(new_second_string) = TextBox::new(&self.second_string).build(cx) {
                        self.second_string = new_second_string;
                    }
                });
            });
            SizedBox::new().uniform(400.0).build(cx, |cx| {
                let data = (self.first_string.clone(), self.second_string.clone());
                Painter::new(data.clone()).build(cx, |ctx, env, (first, second)| {
                    paint(ctx, env, first, second);
                });
            });
        });
    }
}

fn paint(ctx: &mut PaintCtx, env: &Env, first: &str, second: &str) {
    use druid::{
        kurbo::{BezPath, Point},
        piet::{ImageFormat, InterpolationMode, Text, TextLayoutBuilder},
        Affine, Color, FontDescriptor, FontFamily, Rect, RenderContext, TextLayout,
    };

    // Clear the whole widget with the color of your choice
    // (ctx.size() returns the size of the layout rect we're painting in)
    // Note: ctx also has a `clear` method, but that clears the whole context,
    // and we only want to clear this widget's area.
    let size = ctx.size();
    let rect = size.to_rect();
    ctx.fill(rect, &Color::WHITE);

    // We can paint with a Z index, this indicates that this code will be run
    // after the rest of the painting. Painting with z-index is done in order,
    // so first everything with z-index 1 is painted and then with z-index 2 etc.
    // As you can see this(red) curve is drawn on top of the green curve
    ctx.paint_with_z_index(1, move |ctx| {
        let mut path = BezPath::new();
        path.move_to((0.0, size.height));
        path.quad_to((40.0, 50.0), (size.width, 0.0));
        // Create a color
        let stroke_color = Color::rgb8(128, 0, 0);
        // Stroke the path with thickness 1.0
        ctx.stroke(path, &stroke_color, 5.0);
    });

    // Create an arbitrary bezier path
    let mut path = BezPath::new();
    path.move_to(Point::ORIGIN);
    path.quad_to((40.0, 50.0), (size.width, size.height));
    // Create a color
    let stroke_color = Color::rgb8(0, 128, 0);
    // Stroke the path with thickness 5.0
    ctx.stroke(path, &stroke_color, 5.0);

    // Rectangles: the path for practical people
    let rect = Rect::from_origin_size((10.0, 10.0), (100.0, 100.0));
    // Note the Color:rgba8 which includes an alpha channel (7F in this case)
    let fill_color = Color::rgba8(0x00, 0x00, 0x00, 0x7F);
    ctx.fill(rect, &fill_color);

    // Text is easy; in real use TextLayout should either be stored in the
    // widget and reused, or a label child widget to manage it all.
    // This is one way of doing it, you can also use a builder-style way.
    let mut layout = TextLayout::<String>::from_text(first);
    layout.set_font(FontDescriptor::new(FontFamily::SERIF).with_size(24.0));
    layout.set_text_color(fill_color);
    layout.rebuild_if_needed(ctx.text(), env);

    // Let's rotate our text slightly. First we save our current (default) context:
    ctx.with_save(|ctx| {
        // Now we can rotate the context (or set a clip path, for instance):
        // This makes it so that anything drawn after this (in the closure) is
        // transformed.
        // The transformation is in radians, but be aware it transforms the canvas,
        // not just the part you are drawing. So we draw at (80.0, 40.0) on the rotated
        // canvas, this is NOT the same position as (80.0, 40.0) on the original canvas.
        ctx.transform(Affine::rotate(std::f64::consts::FRAC_PI_4));
        layout.draw(ctx, (80.0, 40.0));
    });
    // When we exit with_save, the original context's rotation is restored

    // This is the builder-style way of drawing text.
    let text = ctx.text();
    let layout = text
        .new_text_layout(second.to_string())
        .font(FontFamily::SERIF, 24.0)
        .text_color(Color::rgb8(128, 0, 0))
        .build()
        .unwrap();
    ctx.draw_text(&layout, (100.0, 25.0));

    // Let's burn some CPU to make a (partially transparent) image buffer
    let image_data = make_image_data(256, 256);
    let image = ctx
        .make_image(256, 256, &image_data, ImageFormat::RgbaSeparate)
        .unwrap();
    // The image is automatically scaled to fit the rect you pass to draw_image
    ctx.draw_image(&image, size.to_rect(), InterpolationMode::Bilinear);
}

fn make_image_data(width: usize, height: usize) -> Vec<u8> {
    let mut result = vec![0; width * height * 4];
    for y in 0..height {
        for x in 0..width {
            let ix = (y * width + x) * 4;
            result[ix] = x as u8;
            result[ix + 1] = y as u8;
            result[ix + 2] = !(x as u8);
            result[ix + 3] = 127;
        }
    }
    result
}
