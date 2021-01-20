use eframe::{egui, epi};
use egui::{vec2, Color32, Stroke};

struct State {}

pub struct MyApp {
  #[allow(unused)]
  state: State,
}

impl Default for MyApp {
  fn default() -> Self {
    Self { state: State {} }
  }
}

impl epi::App for MyApp {
  fn name(&self) -> &str {
    "My App"
  }

  fn setup(&mut self, ctx: &egui::CtxRef) {
    let mut font_definitions = egui::FontDefinitions::default();
    font_definitions.font_data.insert(
      "Inter".to_owned(),
      std::borrow::Cow::Borrowed(include_bytes!("Inter-Regular.ttf")),
    );
    font_definitions.font_data.insert(
      "JetBrainsMono".to_owned(),
      std::borrow::Cow::Borrowed(include_bytes!("JetBrainsMono-Regular.ttf")),
    );
    font_definitions
      .fonts_for_family
      .insert(egui::FontFamily::Proportional, vec!["Inter".to_owned()]);
    font_definitions.fonts_for_family.insert(
      egui::FontFamily::Monospace,
      vec!["JetBrainsMono".to_owned()],
    );
    ctx.set_fonts(font_definitions);
  }

  fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
    egui::CentralPanel::default()
      .frame(egui::Frame {
        margin: vec2(16., 16.),
        fill: Color32::WHITE,
        ..Default::default()
      })
      .show(ctx, |ui| {
        let style = ui.style_mut();
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1., Color32::BLACK);

        ui.vertical_centered(|ui| {
          ui.heading("My App");
          ui.advance_cursor(16.);

          let (id, rect) = ui.allocate_space(vec2(256., 256.));
          let board_interact = ui.interact(rect, id, egui::Sense::click());
          let painter = ui.painter();
          painter.rect_stroke(
            rect,
            4.,
            Stroke::new(
              4.,
              if board_interact.hovered {
                Color32::GRAY
              } else {
                Color32::BLACK
              },
            ),
          );
        });
      });

    frame.set_window_size(ctx.used_size());
  }
}
