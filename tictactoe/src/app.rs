use super::hyperlink::Hyperlink;
use eframe::{egui, epi};
use egui::{pos2, vec2, Color32, Pos2, Rect, Stroke, Widget};
use rand::seq::SliceRandom;

#[derive(Copy, Clone, Debug, PartialEq)]
enum Player {
  User,
  Computer,
}

#[derive(Copy, Clone, Debug, Default)]
struct State {
  board: [[Option<Player>; 3]; 3],
  last_player_move_time: Option<f64>,
}

impl State {
  fn cells(&mut self) -> impl Iterator<Item = (usize, usize, &mut Option<Player>)> {
    self
      .board
      .iter_mut()
      .enumerate()
      .flat_map(|(x, v)| v.iter_mut().enumerate().map(move |(y, v)| (x, y, v)))
  }
}

pub struct TicTacToeApp {
  state: State,
}

impl Default for TicTacToeApp {
  fn default() -> Self {
    Self {
      state: Default::default(),
    }
  }
}

impl epi::App for TicTacToeApp {
  fn name(&self) -> &str {
    "Egui template"
  }

  fn setup(&mut self, ctx: &egui::CtxRef) {
    let mut font_definitions = egui::FontDefinitions::default();
    font_definitions.font_data.insert(
      "Inter".to_owned(),
      std::borrow::Cow::Borrowed(include_bytes!("Inter-Regular.ttf")),
    );
    font_definitions
      .fonts_for_family
      .insert(egui::FontFamily::Proportional, vec!["Inter".to_owned()]);
    font_definitions.family_and_size.insert(
      egui::TextStyle::Heading,
      (egui::FontFamily::Proportional, 48.0),
    );
    font_definitions.family_and_size.insert(
      egui::TextStyle::Button,
      (egui::FontFamily::Proportional, 32.0),
    );
    font_definitions.family_and_size.insert(
      egui::TextStyle::Body,
      (egui::FontFamily::Proportional, 16.0),
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
        style.visuals.widgets.active.fg_stroke = Stroke::new(1., Color32::BLACK);
        style.visuals.widgets.hovered.fg_stroke =
          Stroke::new(1., Color32::from_rgb(0xFF, 0x66, 0x76));
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1., Color32::BLACK);
        style.visuals.hyperlink_color = Color32::BLACK;

        ui.vertical_centered(|ui| {
          ui.heading("tictactoe");
          ui.advance_cursor(24.);
          let _board_response = self.show_board(ui, frame);
          ui.advance_cursor(16.);

          let button = egui::Button::new("reset").frame(false);
          if button.ui(ui).clicked {
            self.state.last_player_move_time = None;
            self.state.board = Default::default();
            ui.ctx().request_repaint();
          }

          ui.advance_cursor(16.);
          ui.allocate_ui(vec2(_board_response.rect.width(), 0.), |ui| {
            ui.horizontal_wrapped_for_text(egui::TextStyle::Body, |ui| {
              ui.label("Built with Rust and");
              ui.add(Hyperlink::new("https://github.com/emilk/egui").text("egui"));
              ui.label("\nDon't try too hard. The \"AI\" makes random moves.");
            });
          })
        });
      });

    frame.set_window_size(ctx.used_size());
  }
}

impl TicTacToeApp {
  const CELL_SIZE: f32 = 108.;
  const GRID_LINE_WIDTH: f32 = 4.;

  fn show_board(&mut self, ui: &mut egui::Ui, frame: &mut epi::Frame<'_>) -> egui::Response {
    let cell_size = Self::CELL_SIZE;
    let grid_size = cell_size * 3. + Self::GRID_LINE_WIDTH * 2.;

    let (id, rect) = ui.allocate_space(vec2(grid_size, grid_size));
    draw_grid(ui, rect);

    let board_interact = ui.interact(rect, id, egui::Sense::click());

    let num_player = self
      .state
      .cells()
      .filter(|(_, _, p)| **p == Some(Player::User))
      .count();
    let num_computer = self
      .state
      .cells()
      .filter(|(_, _, p)| **p == Some(Player::Computer))
      .count();

    let winning_cells = get_winner(self.state.board).map(|(_, cells)| cells);

    let hc = if winning_cells.is_none() && board_interact.hovered {
      ui.input()
        .mouse
        .pos
        .and_then(|mouse_pos| hovered_coord(&rect, mouse_pos))
    } else {
      None
    };

    if winning_cells.is_none() && num_player > num_computer && num_player + num_computer < 9 {
      if let Some(last_player_move_time) = self.state.last_player_move_time {
        if ui.input().time - last_player_move_time > 0.5 {
          let empty_cells: Vec<(usize, usize)> = self
            .state
            .cells()
            .filter(|(_, _, p)| p.is_none())
            .map(|(r, c, _)| (r, c))
            .collect();
          if let Some((r, c)) = empty_cells.choose(&mut rand::thread_rng()) {
            self.state.board[*r][*c] = Some(Player::Computer);
            ui.ctx().request_repaint();
          }
        }
      }
    }

    let painter = ui.painter();

    if let Some(hovered_coord) = hc {
      painter.rect_filled(
        Rect::from_center_size(
          Self::center_for_coord(&rect, hovered_coord),
          vec2(cell_size, cell_size),
        ),
        0.,
        Color32::BLACK,
      );

      if board_interact.clicked && num_player == num_computer {
        let (r, c) = hovered_coord;
        if self.state.board[r][c].is_none() {
          self.state.board[r][c] = Some(Player::User);
          self.state.last_player_move_time = Some(ui.input().time);

          let repaint_signal = frame.repaint_signal();
          request_repaint(repaint_signal);
        }
      }
    }

    for (r, c, p) in self.state.cells() {
      if let Some(p) = *p {
        let center = Self::center_for_coord(&rect, (r, c));
        let color =
          if winning_cells.map(|winning_cells| winning_cells.contains(&(r, c))) == Some(true) {
            Color32::from_rgb(0xFF, 0xCB, 0x66)
          } else if hc == Some((r, c)) {
            Color32::WHITE
          } else {
            Color32::BLACK
          };
        match p {
          Player::User => {
            let bounds =
              Rect::from_center_size(center, vec2(Self::CELL_SIZE - 16., Self::CELL_SIZE - 16.));
            painter.line_segment([bounds.left_top(), bounds.right_bottom()], (4., color));
            painter.line_segment([bounds.left_bottom(), bounds.right_top()], (4., color));
          }
          Player::Computer => {
            painter.circle_stroke(center, (Self::CELL_SIZE - 16.) / 2., (4., color));
          }
        }
      }
    }

    board_interact
  }

  fn center_for_coord(rect: &Rect, coord: (usize, usize)) -> Pos2 {
    pos2(
      rect.left()
        + coord.1 as f32 * (Self::CELL_SIZE + Self::GRID_LINE_WIDTH)
        + (Self::CELL_SIZE / 2.),
      rect.top()
        + coord.0 as f32 * (Self::CELL_SIZE + Self::GRID_LINE_WIDTH)
        + (Self::CELL_SIZE / 2.),
    )
  }
}

fn coords() -> impl Iterator<Item = (usize, usize)> {
  (0..3).flat_map(move |r| (0..3).map(move |c| (r, c)))
}

fn draw_grid(ui: &mut egui::Ui, rect: Rect) {
  let painter = ui.painter();
  for i in 1..=2 {
    let offset =
      TicTacToeApp::CELL_SIZE * i as f32 + (TicTacToeApp::GRID_LINE_WIDTH * (i - 1) as f32);
    painter.line_segment(
      [
        pos2(
          rect.left() + offset + TicTacToeApp::GRID_LINE_WIDTH / 2.,
          rect.top(),
        ),
        pos2(
          rect.left() + offset + TicTacToeApp::GRID_LINE_WIDTH / 2.,
          rect.bottom(),
        ),
      ],
      (TicTacToeApp::GRID_LINE_WIDTH, Color32::BLACK),
    );
    painter.line_segment(
      [
        pos2(
          rect.left(),
          rect.top() + offset + TicTacToeApp::GRID_LINE_WIDTH / 2.,
        ),
        pos2(
          rect.right(),
          rect.top() + offset + TicTacToeApp::GRID_LINE_WIDTH / 2.,
        ),
      ],
      (TicTacToeApp::GRID_LINE_WIDTH, Color32::BLACK),
    );
  }
}

fn hovered_coord(rect: &Rect, mouse_pos: Pos2) -> Option<(usize, usize)> {
  for c in coords() {
    let y = c.0 as f32 * (TicTacToeApp::CELL_SIZE + TicTacToeApp::GRID_LINE_WIDTH) + rect.top();
    let x = c.1 as f32 * (TicTacToeApp::CELL_SIZE + TicTacToeApp::GRID_LINE_WIDTH) + rect.left();
    let left = x;
    let top = y;
    let right = x + TicTacToeApp::CELL_SIZE;
    let bottom = y + TicTacToeApp::CELL_SIZE;
    let is_hovering =
      left <= mouse_pos.x && mouse_pos.x <= right && top <= mouse_pos.y && mouse_pos.y <= bottom;
    if is_hovering {
      return Some(c);
    }
  }
  None
}

fn get_winner(board: [[Option<Player>; 3]; 3]) -> Option<(Player, [(usize, usize); 3])> {
  for (r, row) in board.iter().enumerate() {
    if let Some(p) = row[0] {
      if row[1] == Some(p) && row[2] == Some(p) {
        return Some((p, [(r, 0), (r, 1), (r, 2)]));
      }
    }
  }
  for c in 0..3 {
    if let Some(p) = board[0][c] {
      if board[1][c] == Some(p) && board[2][c] == Some(p) {
        return Some((p, [(0, c), (1, c), (2, c)]));
      }
    }
  }
  if let Some(p) = board[0][0] {
    if board[1][1] == Some(p) && board[2][2] == Some(p) {
      return Some((p, [(0, 0), (1, 1), (2, 2)]));
    }
  }
  if let Some(p) = board[0][2] {
    if board[1][1] == Some(p) && board[2][0] == Some(p) {
      return Some((p, [(0, 2), (1, 1), (2, 0)]));
    }
  }
  None
}

#[cfg(target_arch = "wasm32")]
pub fn request_repaint(repaint_signal: std::sync::Arc<dyn epi::RepaintSignal>) {
  use wasm_bindgen::JsCast;
  let window = web_sys::window().unwrap();
  let closure = wasm_bindgen::prelude::Closure::wrap(Box::new(move || {
    repaint_signal.request_repaint();
  }) as Box<dyn FnMut()>);
  window
    .set_timeout_with_callback_and_timeout_and_arguments_0(closure.as_ref().unchecked_ref(), 500)
    .unwrap();
  closure.forget();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn request_repaint(repaint_signal: std::sync::Arc<dyn epi::RepaintSignal>) {
  std::thread::spawn(move || {
    std::thread::sleep(std::time::Duration::from_millis(500));
    repaint_signal.request_repaint();
  });
}
