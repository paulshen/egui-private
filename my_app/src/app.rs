use crate::code::{Code, ColoredText};
use crate::js;
use eframe::{egui, epi};
use egui::*;
use std::collections::HashMap;
use wasm_bindgen::prelude::Closure;

struct FileData(String, ColoredText);

pub struct MyApp {
  filename: String,
  files: HashMap<String, FileData>,
  hover_offset: Option<(usize, f64, bool)>,
  display_quick_info: (
    Option<js::QuickInfoResponse>,
    Option<js::GetDefinitionResponse>,
  ),
  scroll_to_code_offset: Option<usize>,
  navigation_history: Vec<(String, usize, String)>,
}

impl Default for MyApp {
  fn default() -> Self {
    let mut files = HashMap::new();
    files.insert(
      "loading".to_string(),
      FileData(LOADING_CODE.to_string(), syntax_highlighting(LOADING_CODE)),
    );
    MyApp {
      filename: "loading".to_string(),
      files,
      hover_offset: None,
      display_quick_info: (None, None),
      scroll_to_code_offset: None,
      navigation_history: vec![],
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

const SCROLL_PADDING_VERTICAL: f32 = 16.;

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
      .insert(FontFamily::Proportional, vec!["JetBrainsMono".to_owned()]);
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
    let mut scroll_to_code_offset: Option<usize> = self.scroll_to_code_offset;
    self.scroll_to_code_offset = None;
    for event in ctx.input().events.iter() {
      if let Event::JsCall(s) = event {
        let call_response: js::JsCallResponse = serde_json::from_str(s).unwrap();
        if call_response.kind == "fileContents" {
          let file_contents_response: js::FileContentsResponse =
            serde_json::from_str(&call_response.value).unwrap();
          if self.filename != file_contents_response.filename {
            self.filename = file_contents_response.filename;
            let colored_text = syntax_highlighting(&file_contents_response.contents);
            self.files.insert(
              self.filename.clone(),
              FileData(file_contents_response.contents, colored_text),
            );
          }
          scroll_to_code_offset = file_contents_response.offset;
        } else if call_response.kind == "getDefinition" {
          let response: js::GetDefinitionResponse =
            serde_json::from_str(&call_response.value).unwrap();
          if self.filename == response.query_filename {
            if let (Some(ref quick_info), _) = self.display_quick_info {
              if quick_info.text_span.start == response.query_offset {
                self.display_quick_info.1 = Some(response);
              }
            }
          }
        } else if call_response.kind == "quickInfo" {
          let response: js::QuickInfoResponse = serde_json::from_str(&call_response.value).unwrap();
          if let Some((hover_offset, _, _)) = self.hover_offset {
            if response.text_span.includes_offset(hover_offset) {
              let offset = response.text_span.start;
              self.display_quick_info = (Some(response), None);
              let filename = self.filename.clone();
              egui_web::spawn_future(
                async move { js::api::get_definition(filename, offset).await },
              );
            }
          }
        }
      }
    }

    if let Some((hover_offset, hover_time, ref mut did_fetch)) = self.hover_offset {
      if !*did_fetch && ctx.input().time > hover_time + 0.05 {
        *did_fetch = true;
        let filename = self.filename.clone();
        egui_web::spawn_future(
          async move { js::api::get_quick_info(filename, hover_offset).await },
        );
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

        let frame_height = ui.available_size().y;
        ui.horizontal(|ui| {
          ui.allocate_space(vec2(256.0, frame_height));
          let mut sidebar_ui = ui.child_ui(
            Rect::from_min_max(Pos2::zero(), pos2(256.0, frame_height)),
            Layout::top_down(Align::Min),
          );

          ScrollArea::auto_sized().show(ui, |ui| {
            Frame {
              margin: vec2(16., SCROLL_PADDING_VERTICAL),
              ..Default::default()
            }
            .show(ui, |ui| {
              let file_data = self.files.get(&self.filename).unwrap();
              let colored_text = &file_data.1;
              let code = Code::new(&file_data.0, colored_text);
              let mut hover_offset: Option<usize> = None;
              let where_to_put_background = ui.painter().add(Shape::Noop);

              let (code_response, scroll_to_offset, galley) =
                code.ui(ui, &mut hover_offset, scroll_to_code_offset);
              if let Some(scroll_to_offset) = scroll_to_offset {
                ctx.set_scroll_target(scroll_to_offset + code_response.rect.top(), Align::Min);
                ctx.skip_paint();
              }
              if let Some(hover_offset) = hover_offset {
                if match self.hover_offset {
                  Some((old_hover_offset, _, _)) => hover_offset != old_hover_offset,
                  None => true,
                } {
                  let mut does_overlap_with_current = true;
                  if let (Some(ref display_quick_info), _) = self.display_quick_info {
                    if !display_quick_info.text_span.includes_offset(hover_offset) {
                      self.display_quick_info = (None, None);
                    } else {
                      does_overlap_with_current = false;
                    }
                  }
                  self.hover_offset =
                    Some((hover_offset, ui.input().time, !does_overlap_with_current));

                  let repaint_signal = frame.repaint_signal();
                  let closure = Closure::wrap(Box::new(move || {
                    repaint_signal.request_repaint();
                  }) as Box<dyn FnMut()>);
                  let window = web_sys::window().unwrap();
                  use wasm_bindgen::JsCast;
                  window
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                      closure.as_ref().unchecked_ref(),
                      50,
                    )
                    .unwrap();
                  closure.forget();
                }

                if code_response.clicked {
                  if let (_, Some(ref definition)) = self.display_quick_info {
                    let name = definition.name.clone();
                    if self.files.get(&definition.filename).is_some() {
                      self.filename = definition.filename.clone();
                      self.scroll_to_code_offset = Some(definition.offset);
                    } else {
                      let filename = definition.filename.clone();
                      let offset = definition.offset;
                      egui_web::spawn_future(async move {
                        js::api::go_to_location(filename, offset).await;
                      });
                    }
                    self.navigation_history.push((
                      definition.filename.clone(),
                      definition.offset,
                      name,
                    ));
                    self.display_quick_info.1 = None;
                  }
                }
              }

              if let (Some(ref display_quick_info), Some(ref _destination)) =
                self.display_quick_info
              {
                let start_cursor = galley.from_ccursor(paint::text::cursor::CCursor {
                  index: display_quick_info.text_span.start,
                  prefer_next_row: false,
                });
                let end_cursor = galley.from_ccursor(paint::text::cursor::CCursor {
                  index: display_quick_info.text_span.start + display_quick_info.text_span.length,
                  prefer_next_row: false,
                });
                let rects = paint_cursor_selection(
                  code_response.rect.left_top(),
                  &galley,
                  start_cursor,
                  end_cursor,
                );
                let token_color =
                  get_color_at_offset(colored_text, display_quick_info.text_span.start);
                ui.painter().set(
                  where_to_put_background,
                  Shape::Vec(
                    rects
                      .into_iter()
                      .map(|rect| Shape::LineSegment {
                        points: [rect.left_bottom(), rect.right_bottom()],
                        stroke: (1.0, token_color).into(),
                      })
                      .collect(),
                  ),
                );
              }
            })
          });

          // Render sidebar after so we don't update filename on this frame
          sidebar_ui.set_clip_rect(Rect::from_min_max(Pos2::zero(), pos2(256.0, frame_height)));
          for (filename, offset, name) in self.navigation_history.iter().rev() {
            if sidebar_ui.button(name.clone()).clicked {
              self.filename = filename.clone();
              self.scroll_to_code_offset = Some(*offset);
            }
          }
        });

        if let (Some(ref display_quick_info), _) = self.display_quick_info {
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

        // if let (_, Some(ref definition_info)) = self.display_quick_info {
        //   egui_web::console_log(format!("definition {}", definition_info.offset));
        // }
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

fn paint_cursor_selection(
  pos: Pos2,
  galley: &paint::text::Galley,
  min: paint::text::cursor::Cursor,
  max: paint::text::cursor::Cursor,
) -> Vec<Rect> {
  let min = min.rcursor;
  let max = max.rcursor;

  let mut rv = vec![];

  for ri in min.row..=max.row {
    let row = &galley.rows[ri];
    let left = if ri == min.row {
      row.x_offset(min.column)
    } else {
      row.min_x()
    };
    let right = if ri == max.row {
      row.x_offset(max.column)
    } else {
      let newline_size = if row.ends_with_newline {
        row.height() / 2.0 // visualize that we select the newline
      } else {
        0.0
      };
      row.max_x() + newline_size
    };
    let rect = Rect::from_min_max(pos + vec2(left, row.y_min), pos + vec2(right, row.y_max));
    rv.push(rect);
  }

  rv
}

fn get_color_at_offset(text: &ColoredText, offset: usize) -> Color32 {
  let mut iter = 0;
  for row in text.0.iter() {
    for token in row {
      let token_len = token.1.len();
      if token_len == 0 {
        continue;
      }
      iter += token_len;
      if offset < iter {
        let fg = token.0.foreground;
        return Color32::from_rgb(fg.r, fg.g, fg.b);
      }
    }
    iter += 1;
  }
  panic!("panic message");
}
