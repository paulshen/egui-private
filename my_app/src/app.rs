use crate::code::{Code, ColoredText};
use crate::js;
use eframe::{egui, epi};
use egui::*;
use wasm_bindgen::prelude::Closure;

pub struct MyApp {
  pub filename: String,
  pub code: String,
  pub colored_text: ColoredText,
  hover_offset: Option<(usize, f64, bool)>,
  display_quick_info: Option<js::QuickInfoResponse>,
}

impl Default for MyApp {
  fn default() -> Self {
    MyApp {
      filename: "loading".to_string(),
      code: LOADING_CODE.to_string(),
      colored_text: syntax_highlighting(LOADING_CODE),
      hover_offset: None,
      display_quick_info: None,
    }
  }
}

fn rgb(rgb: i32) -> Color32 {
  Color32::from_rgb(
    (rgb >> 16 & 0xff) as u8,
    (rgb >> 8 & 0xff) as u8,
    (rgb & 0xff) as u8,
  )
}

impl epi::App for MyApp {
  fn name(&self) -> &str {
    "My App"
  }

  fn setup(&mut self, ctx: &CtxRef) {
    let mut font_definitions = FontDefinitions::default();
    font_definitions.font_data.insert(
      "JetBrainsMono".to_owned(),
      std::borrow::Cow::Borrowed(include_bytes!("JetBrainsMono-Regular.ttf")),
    );
    font_definitions
      .fonts_for_family
      .insert(FontFamily::Monospace, vec!["JetBrainsMono".to_owned()]);
    ctx.set_fonts(font_definitions);
    let mut style = (*ctx.style()).clone();
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1., rgb(0x343d46));
    style.body_text_style = TextStyle::Monospace;
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1., rgb(0xc0c5ce));
    ctx.set_style(style);
  }

  fn update(&mut self, ctx: &CtxRef, frame: &mut epi::Frame<'_>) {
    for event in ctx.input().events.iter() {
      if let Event::JsCall(s) = event {
        let call_response: js::JsCallResponse = serde_json::from_str(s).unwrap();
        if call_response.kind == "fileContents" {
          let file_contents_response: js::FileContentsResponse =
            serde_json::from_str(&call_response.value).unwrap();
          self.filename = file_contents_response.filename;
          self.code = file_contents_response.contents;
          self.colored_text = syntax_highlighting(&self.code);
        } else if call_response.kind == "quickInfo" {
          let file_contents_response: js::QuickInfoResponse =
            serde_json::from_str(&call_response.value).unwrap();
          if let Some((hover_offset, _, _)) = self.hover_offset {
            if file_contents_response
              .text_span
              .includes_offset(hover_offset)
            {
              self.display_quick_info = Some(file_contents_response);
            }
          }
        }
      }
    }

    if let Some((hover_offset, hover_time, ref mut did_fetch)) = self.hover_offset {
      if !*did_fetch && ctx.input().time > hover_time + 0.3 {
        *did_fetch = true;
        let filename = self.filename.clone();
        egui_web::spawn_future(async move {
          js::api::get_quick_info(filename, hover_offset).await;
        });
      }
    }

    CentralPanel::default()
      .frame(Frame {
        margin: Vec2::zero(),
        fill: rgb(0x2b303b),
        ..Default::default()
      })
      .show(ctx, |ui| {
        let style = ui.style_mut();
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1., Color32::BLACK);
        style.visuals.dark_bg_color = rgb(0x2b303b);
        style.visuals.widgets.inactive.bg_fill = rgb(0x4f5b66);
        style.visuals.widgets.hovered.bg_fill = rgb(0x4f5b66);
        style.visuals.widgets.active.bg_fill = rgb(0x4f5b66);
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1., rgb(0x8fa1b3));
        style.visuals.widgets.active.bg_stroke = Stroke::new(1., rgb(0x8fa1b3));

        ScrollArea::auto_sized().show(ui, |ui| {
          Frame {
            margin: vec2(16., 16.),
            ..Default::default()
          }
          .show(ui, |ui| {
            let code = Code::new(&self.code, &self.colored_text);
            let mut hover_offset: Option<usize> = None;
            let code_response = code.ui(ui, &mut hover_offset);
            if let Some(hover_offset) = hover_offset {
              if match self.hover_offset {
                Some((old_hover_offset, _, _)) => hover_offset != old_hover_offset,
                None => true,
              } {
                self.hover_offset = Some((hover_offset, ui.input().time, false));
                if let Some(ref display_quick_info) = self.display_quick_info {
                  if !display_quick_info.text_span.includes_offset(hover_offset) {
                    self.display_quick_info = None;
                  }
                }

                let repaint_signal = frame.repaint_signal();
                let closure = Closure::wrap(Box::new(move || {
                  repaint_signal.request_repaint();
                }) as Box<dyn FnMut()>);
                let window = web_sys::window().unwrap();
                use wasm_bindgen::JsCast;
                window
                  .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    300,
                  )
                  .unwrap();
                closure.forget();
              }

              if code_response.clicked {
                let filename = self.filename.clone();
                egui_web::spawn_future(async move {
                  js::api::go_to_definition(filename, hover_offset).await;
                });
              }
            }
          })
        });

        if let Some(ref display_quick_info) = self.display_quick_info {
          show_tooltip(ui.ctx(), |ui| {
            Frame {
              margin: vec2(2.0, 0.),
              ..Default::default()
            }
            .show(ui, |ui| {
              ui.add(Label::new(
                display_quick_info
                  .display_parts
                  .iter()
                  .map(|display_part| display_part.text.as_str())
                  .collect::<Vec<&str>>()
                  .join(""),
              ));
            })
          });
        }
      });

    frame.set_window_size(ctx.used_size());
  }
}

const LOADING_CODE: &str = "// loading...";

fn syntax_highlighting(text: &str) -> ColoredText {
  ColoredText::text_with_extension(text, "js")
}

impl ColoredText {
  fn text_with_extension(text: &str, extension: &str) -> ColoredText {
    use syntect::easy::HighlightLines;
    use syntect::highlighting::ThemeSet;
    use syntect::parsing::SyntaxSet;
    use syntect::util::LinesWithEndings;

    let ps = SyntaxSet::load_defaults_newlines(); // should be cached and reused
    let ts = ThemeSet::load_defaults(); // should be cached and reused

    let syntax = ps.find_syntax_by_extension(extension).unwrap();

    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

    let lines = LinesWithEndings::from(text)
      .map(|line| {
        h.highlight(line, &ps)
          .into_iter()
          .map(|(style, range)| (style, range.trim_end_matches('\n').to_owned()))
          .collect()
      })
      .collect();

    ColoredText(lines)
  }
}
