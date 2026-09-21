use super::*;

/// Input handler used by the terminal canvas.
///
/// The default [`ElementInputHandler`] keeps macOS's "press and hold" accent
/// menu enabled, which replaces key auto-repeat with an accent-character popup.
/// A terminal should behave like Terminal.app and repeat the held key instead,
/// so [`InputHandler::apple_press_and_hold_enabled`] reflects the user's
/// `key_repeat_enabled` setting (returning `true` would re-enable the accent
/// menu and swallow repeats).
///
/// When that method reports `false`, the macOS backend delivers repeated
/// printable keys straight to [`InputHandler::replace_text_in_range`] as raw
/// input, which the terminal forwards to the PTY — making `x` in vim repeat,
/// and held letters/digits auto-repeat while typing.
pub(super) struct TerminalInputHandler {
    inner: ElementInputHandler<TerminalView>,
    key_repeat_enabled: bool,
}

impl TerminalInputHandler {
    pub(super) fn new(
        bounds: Bounds<Pixels>,
        view: Entity<TerminalView>,
        key_repeat_enabled: bool,
    ) -> Self {
        Self {
            inner: ElementInputHandler::new(bounds, view),
            key_repeat_enabled,
        }
    }
}

impl InputHandler for TerminalInputHandler {
    fn selected_text_range(
        &mut self,
        ignore_disabled_input: bool,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<UTF16Selection> {
        self.inner
            .selected_text_range(ignore_disabled_input, window, cx)
    }

    fn marked_text_range(
        &mut self,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<std::ops::Range<usize>> {
        self.inner.marked_text_range(window, cx)
    }

    fn text_for_range(
        &mut self,
        range_utf16: std::ops::Range<usize>,
        adjusted_range: &mut Option<std::ops::Range<usize>>,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<String> {
        self.inner
            .text_for_range(range_utf16, adjusted_range, window, cx)
    }

    fn replace_text_in_range(
        &mut self,
        replacement_range: Option<std::ops::Range<usize>>,
        text: &str,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.inner
            .replace_text_in_range(replacement_range, text, window, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<std::ops::Range<usize>>,
        new_text: &str,
        new_selected_range: Option<std::ops::Range<usize>>,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.inner.replace_and_mark_text_in_range(
            range_utf16,
            new_text,
            new_selected_range,
            window,
            cx,
        );
    }

    fn unmark_text(&mut self, window: &mut Window, cx: &mut App) {
        self.inner.unmark_text(window, cx);
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: std::ops::Range<usize>,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<Bounds<Pixels>> {
        self.inner.bounds_for_range(range_utf16, window, cx)
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<usize> {
        self.inner.character_index_for_point(point, window, cx)
    }

    fn set_selected_text_range(
        &mut self,
        range_utf16: std::ops::Range<usize>,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.inner.set_selected_text_range(range_utf16, window, cx);
    }

    fn element_bounds(&mut self, window: &mut Window, cx: &mut App) -> Option<Bounds<Pixels>> {
        self.inner.element_bounds(window, cx)
    }

    fn text_length_utf16(&mut self, window: &mut Window, cx: &mut App) -> Option<usize> {
        self.inner.text_length_utf16(window, cx)
    }

    fn apple_press_and_hold_enabled(&mut self) -> bool {
        // When key repeat is enabled, disable the macOS accent-menu that replaces
        // key auto-repeat so held keys repeat (vim `x`, typing, deleting,
        // navigating). When disabled, the accent menu stays available.
        !self.key_repeat_enabled
    }

    fn accepts_text_input(&mut self, window: &mut Window, cx: &mut App) -> bool {
        self.inner.accepts_text_input(window, cx)
    }

    fn prefers_ime_for_printable_keys(&mut self, _window: &mut Window, _cx: &mut App) -> bool {
        false
    }
}

impl EntityInputHandler for TerminalView {
    fn text_for_range(
        &mut self,
        _range: std::ops::Range<usize>,
        _actual_range: &mut Option<std::ops::Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        None
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: 0..0,
            reversed: false,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<std::ops::Range<usize>> {
        self.marked_text_range()
    }

    fn unmark_text(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.clear_marked_text(cx);
    }

    fn replace_text_in_range(
        &mut self,
        _replacement_range: Option<std::ops::Range<usize>>,
        text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.clear_marked_text(cx);
        self.commit_text(text, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        _range: Option<std::ops::Range<usize>>,
        new_text: &str,
        new_marked_range: Option<std::ops::Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set_marked_text(new_text.to_string(), new_marked_range, cx);
    }

    fn bounds_for_range(
        &mut self,
        _range: std::ops::Range<usize>,
        _bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        // 获取光标位置用于 IME 定位
        let screen_line = self.terminal_frame_snapshot.cursor_screen_line;
        let col = self.terminal_frame_snapshot.cursor_column;

        // 计算像素位置
        let origin = Point::new(
            self.terminal_bounds.origin.x + self.line_margin_width() + self.cell_width * col as f32,
            self.terminal_bounds.origin.y + self.line_height * screen_line as f32,
        );

        Some(Bounds::new(origin, size(self.cell_width, self.line_height)))
    }

    fn character_index_for_point(
        &mut self,
        _point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        None
    }
}