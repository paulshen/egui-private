use crate::code::{Code, ColoredText};
use eframe::{egui, epi};
use egui::{vec2, Color32, Stroke, Vec2, Widget};

pub struct MyApp {
  colored_text: ColoredText,
}

impl Default for MyApp {
  fn default() -> Self {
    MyApp {
      colored_text: syntax_highlighting(SAMPLE_CODE),
    }
  }
}

impl epi::App for MyApp {
  fn name(&self) -> &str {
    "My App"
  }

  fn setup(&mut self, ctx: &egui::CtxRef) {
    let mut font_definitions = egui::FontDefinitions::default();
    font_definitions.font_data.insert(
      "JetBrainsMono".to_owned(),
      std::borrow::Cow::Borrowed(include_bytes!("JetBrainsMono-Regular.ttf")),
    );
    font_definitions.fonts_for_family.insert(
      egui::FontFamily::Monospace,
      vec!["JetBrainsMono".to_owned()],
    );
    ctx.set_fonts(font_definitions);
  }

  fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
    egui::CentralPanel::default()
      .frame(egui::Frame {
        margin: Vec2::zero(),
        fill: Color32::from_rgb(0x2b, 0x30, 0x3b),
        ..Default::default()
      })
      .show(ctx, |ui| {
        let style = ui.style_mut();
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1., Color32::BLACK);
        style.visuals.dark_bg_color = Color32::from_rgb(0x2b, 0x30, 0x3b);
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(0x4f, 0x5b, 0x66);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(0x4f, 0x5b, 0x66);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(0x4f, 0x5b, 0x66);
        style.visuals.widgets.hovered.bg_stroke =
          Stroke::new(1., Color32::from_rgb(0x8f, 0xa1, 0xb3));
        style.visuals.widgets.active.bg_stroke =
          Stroke::new(1., Color32::from_rgb(0x8f, 0xa1, 0xb3));

        egui::ScrollArea::auto_sized().show(ui, |ui| {
          egui::Frame {
            margin: vec2(16., 16.),
            ..Default::default()
          }
          .show(ui, |ui| {
            let code = Code::new(SAMPLE_CODE, &self.colored_text);
            code.ui(ui);
          })
        });
      });

    frame.set_window_size(ctx.used_size());
  }
}

const SAMPLE_CODE: &str =
  "export function createWorkInProgress(current: Fiber, pendingProps: any): Fiber {
  let workInProgress = current.alternate;
  if (workInProgress === null) {
    // We use a double buffering pooling technique because we know that we'll
    // only ever need at most two versions of a tree. We pool the other unused
    // node that we're free to reuse. This is lazily created to avoid allocating
    // extra objects for things that are never updated. It also allow us to
    // reclaim the extra memory if needed.
    workInProgress = createFiber(
      current.tag,
      pendingProps,
      current.key,
      current.mode,
    );
    workInProgress.elementType = current.elementType;
    workInProgress.type = current.type;
    workInProgress.stateNode = current.stateNode;

    if (__DEV__) {
      // DEV-only fields
      workInProgress._debugID = current._debugID;
      workInProgress._debugSource = current._debugSource;
      workInProgress._debugOwner = current._debugOwner;
      workInProgress._debugHookTypes = current._debugHookTypes;
    }

    workInProgress.alternate = current;
    current.alternate = workInProgress;
  } else {
    workInProgress.pendingProps = pendingProps;
    // Needed because Blocks store data on type.
    workInProgress.type = current.type;

    // We already have an alternate.
    // Reset the effect tag.
    workInProgress.flags = NoFlags;

    // The effect list is no longer valid.
    workInProgress.nextEffect = null;
    workInProgress.firstEffect = null;
    workInProgress.lastEffect = null;
    workInProgress.subtreeFlags = NoFlags;
    workInProgress.deletions = null;

    if (enableProfilerTimer) {
      // We intentionally reset, rather than copy, actualDuration & actualStartTime.
      // This prevents time from endlessly accumulating in new commits.
      // This has the downside of resetting values for different priority renders,
      // But works for yielding (the common case) and should support resuming.
      workInProgress.actualDuration = 0;
      workInProgress.actualStartTime = -1;
    }
  }

  // Reset all effects except static ones.
  // Static effects are not specific to a render.
  workInProgress.flags = current.flags & StaticMask;
  workInProgress.childLanes = current.childLanes;
  workInProgress.lanes = current.lanes;

  workInProgress.child = current.child;
  workInProgress.memoizedProps = current.memoizedProps;
  workInProgress.memoizedState = current.memoizedState;
  workInProgress.updateQueue = current.updateQueue;

  // Clone the dependencies object. This is mutated during the render phase, so
  // it cannot be shared with the current fiber.
  const currentDependencies = current.dependencies;
  workInProgress.dependencies =
    currentDependencies === null
      ? null
      : {
          lanes: currentDependencies.lanes,
          firstContext: currentDependencies.firstContext,
        };

  // These will be overridden during the parent's reconciliation
  workInProgress.sibling = current.sibling;
  workInProgress.index = current.index;
  workInProgress.ref = current.ref;

  if (enableProfilerTimer) {
    workInProgress.selfBaseDuration = current.selfBaseDuration;
    workInProgress.treeBaseDuration = current.treeBaseDuration;
  }

  if (__DEV__) {
    workInProgress._debugNeedsRemount = current._debugNeedsRemount;
    switch (workInProgress.tag) {
      case IndeterminateComponent:
      case FunctionComponent:
      case SimpleMemoComponent:
        workInProgress.type = resolveFunctionForHotReloading(current.type);
        break;
      case ClassComponent:
        workInProgress.type = resolveClassForHotReloading(current.type);
        break;
      case ForwardRef:
        workInProgress.type = resolveForwardRefForHotReloading(current.type);
        break;
      default:
        break;
    }
  }

  return workInProgress;
}";

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
