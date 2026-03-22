"""Modern 2026 theme for OBD Diagnostic Pro."""

import customtkinter as ctk
import math

# ── Color Palette ──────────────────────────────────────────────
COLORS = {
    # Backgrounds
    "bg_primary": "#0A0E1A",       # Deep navy (main background)
    "bg_secondary": "#111827",     # Sidebar / status bar
    "bg_card": "#1E2537",          # Card surfaces
    "bg_card_hover": "#253047",    # Slightly lighter card hover
    "bg_input": "#151B2B",         # Input fields

    # Accents
    "accent": "#06B6D4",           # Cyan OBD (primary brand color)
    "accent_hover": "#0891B2",     # Darker cyan hover
    "accent_light": "#22D3EE",     # Lighter cyan
    "cyan": "#06B6D4",             # Cyan highlight

    # Status
    "success": "#10B981",          # Green
    "warning": "#F59E0B",          # Amber
    "danger": "#EF4444",           # Red
    "danger_hover": "#DC2626",     # Red hover
    "danger_dark": "#B91C1C",      # Dark red
    "danger_light": "#FCA5A5",     # Light red text
    "highlight": "#06B6D4",        # Cyan highlight

    # Text
    "text_primary": "#F1F5F9",     # Near-white
    "text_secondary": "#94A3B8",   # Slate gray
    "text_muted": "#64748B",       # Muted slate

    # Borders & misc
    "border": "#1E293B",           # Subtle border
    "border_light": "#334155",     # Lighter border
    "divider": "#1E293B",          # Divider line

    # Cards with visible borders
    "card_border": "#2A3447",        # Visible card border
    "card_border_accent": "#3B82F6", # Active card border
    "input_border": "#334155",       # Input field border
    "sidebar_border": "#1E293B",     # Sidebar right border

    # Gauge specific
    "gauge_track": "#1E293B",      # Gauge background arc
    "gauge_fill": "#10B981",       # Default gauge fill (green)
}

# ── Font Stack ─────────────────────────────────────────────────
# Cross-platform font fallback
import platform

def _safe_font(*families_and_params):
    """Return a font tuple with cross-platform fallback.
    On macOS: SF Pro Display/Text, Menlo
    On Windows: Segoe UI, Consolas
    On Linux: Ubuntu, DejaVu Sans Mono
    """
    return families_and_params  # CTk handles fallback internally via tkinter

FONTS = {
    "h1": ("Helvetica", 26, "bold"),
    "h2": ("Helvetica", 20, "bold"),
    "h3": ("Helvetica", 16, "bold"),
    "body": ("Helvetica", 13),
    "body_bold": ("Helvetica", 13, "bold"),
    "small": ("Helvetica", 11),
    "small_bold": ("Helvetica", 11, "bold"),
    "mono": ("Courier", 13),
    "mono_large": ("Courier", 22, "bold"),
    "mono_small": ("Courier", 11),
    "nav": ("Helvetica", 12),
    "nav_active": ("Helvetica", 12, "bold"),
    "value_large": ("Courier", 28, "bold"),
    # Backward compat
    "title": ("Helvetica", 26, "bold"),
    "subtitle": ("Helvetica", 20, "bold"),
    "heading": ("Helvetica", 16, "bold"),
}

# Try to use better fonts on macOS
if platform.system() == "Darwin":
    FONTS.update({
        "h1": ("SF Pro Display", 26, "bold"),
        "h2": ("SF Pro Display", 20, "bold"),
        "h3": ("SF Pro Display", 16, "bold"),
        "body": ("SF Pro Text", 13),
        "body_bold": ("SF Pro Text", 13, "bold"),
        "small": ("SF Pro Text", 11),
        "small_bold": ("SF Pro Text", 11, "bold"),
        "mono": ("Menlo", 13),
        "mono_large": ("Menlo", 22, "bold"),
        "mono_small": ("Menlo", 11),
        "nav": ("SF Pro Text", 12),
        "nav_active": ("SF Pro Text", 12, "bold"),
        "value_large": ("Menlo", 28, "bold"),
        "title": ("SF Pro Display", 26, "bold"),
        "subtitle": ("SF Pro Display", 20, "bold"),
        "heading": ("SF Pro Display", 16, "bold"),
    })
elif platform.system() == "Windows":
    FONTS.update({
        "h1": ("Segoe UI", 26, "bold"),
        "h2": ("Segoe UI", 20, "bold"),
        "h3": ("Segoe UI", 16, "bold"),
        "body": ("Segoe UI", 13),
        "body_bold": ("Segoe UI", 13, "bold"),
        "small": ("Segoe UI", 11),
        "small_bold": ("Segoe UI", 11, "bold"),
        "mono": ("Consolas", 13),
        "mono_large": ("Consolas", 22, "bold"),
        "mono_small": ("Consolas", 11),
        "nav": ("Segoe UI", 12),
        "nav_active": ("Segoe UI", 12, "bold"),
        "value_large": ("Consolas", 28, "bold"),
        "title": ("Segoe UI", 26, "bold"),
        "subtitle": ("Segoe UI", 20, "bold"),
        "heading": ("Segoe UI", 16, "bold"),
    })


def apply_theme(root):
    """Apply modern dark theme."""
    ctk.set_appearance_mode("dark")
    ctk.set_default_color_theme("blue")


# ── Gauge Widget ───────────────────────────────────────────────
class GaugeWidget(ctk.CTkCanvas):
    """Modern circular gauge with thick arc and center value."""

    def __init__(self, parent, label="", unit="", min_val=0, max_val=100,
                 warning_threshold=None, danger_threshold=None, size=160):
        super().__init__(parent, width=size, height=size,
                         bg=COLORS["bg_primary"], highlightthickness=0)
        self.label = label
        self.unit = unit
        self.min_val = min_val
        self.max_val = max_val
        self.warning_threshold = warning_threshold
        self.danger_threshold = danger_threshold
        self.size = size
        self.current_value = None
        self._draw_gauge()

    def set_value(self, value):
        """Update gauge value."""
        self.current_value = max(self.min_val, min(self.max_val, value))
        self.delete("all")
        self._draw_gauge()

    def reset(self):
        """Reset gauge to idle state (no fill, '--' display)."""
        self.current_value = None
        self.delete("all")
        self._draw_gauge()

    def _get_color(self):
        """Get arc color based on thresholds."""
        if self.danger_threshold is not None and self.current_value >= self.danger_threshold:
            return COLORS["danger"]
        if self.warning_threshold is not None and self.current_value >= self.warning_threshold:
            return COLORS["warning"]
        return COLORS["success"]

    def _draw_gauge(self):
        cx = self.size / 2
        r = self.size * 0.38
        arc_w = 12
        start = 225
        sweep = 270

        # Track arc
        self.create_arc(cx - r, cx - r, cx + r, cx + r,
                        start=start, extent=-sweep,
                        outline=COLORS["gauge_track"], width=arc_w, style="arc")

        # Fill arc (only when value is set)
        if self.current_value is not None:
            pct = (self.current_value - self.min_val) / max(self.max_val - self.min_val, 1)
            fill_sweep = sweep * pct
            if fill_sweep > 0:
                self.create_arc(cx - r, cx - r, cx + r, cx + r,
                                start=start, extent=-fill_sweep,
                                outline=self._get_color(), width=arc_w, style="arc")

        # Value text
        val_text = f"{int(self.current_value)}" if self.current_value is not None else "--"
        self.create_text(cx, cx - 4, text=val_text,
                         font=FONTS["mono_large"], fill=COLORS["text_primary"])

        # Unit below value
        self.create_text(cx, cx + 20, text=self.unit,
                         font=FONTS["small"], fill=COLORS["text_muted"])

        # Label at top
        self.create_text(cx, 14, text=self.label,
                         font=FONTS["small"], fill=COLORS["text_secondary"])


# ── Line Graph Widget ─────────────────────────────────────────
class GraphWidget(ctk.CTkCanvas):
    """Polished line graph with gradient fill, threshold colors, and current value badge."""

    def __init__(self, parent, label="", unit="", min_val=0, max_val=100,
                 color=None, width=280, height=90, max_samples=60,
                 warning_threshold=None, danger_threshold=None):
        super().__init__(parent, width=width, height=height,
                         bg=COLORS["bg_card"], highlightthickness=0)
        self.label = label
        self.unit = unit
        self.min_val = min_val
        self.max_val = max_val
        self.base_color = color or COLORS["success"]
        self.warning_threshold = warning_threshold
        self.danger_threshold = danger_threshold
        self.w = width
        self.h = height
        self.max_samples = max_samples
        from collections import deque
        self.data = deque(maxlen=max_samples)
        self._draw()

    def add_value(self, value):
        """Append a value and redraw."""
        self.data.append(value)
        self.delete("all")
        self._draw()

    def reset(self):
        """Clear all data."""
        self.data.clear()
        self.delete("all")
        self._draw()

    def _get_color(self, value=None):
        """Get color based on current value and thresholds."""
        if value is None:
            return self.base_color
        if self.danger_threshold is not None and value >= self.danger_threshold:
            return COLORS["danger"]
        if self.warning_threshold is not None and value >= self.warning_threshold:
            return COLORS["warning"]
        return self.base_color

    def _hex_to_rgb(self, hex_color):
        h = hex_color.lstrip('#')
        return tuple(int(h[i:i+2], 16) for i in (0, 2, 4))

    def _rgb_to_hex(self, r, g, b):
        return f"#{int(r):02x}{int(g):02x}{int(b):02x}"

    def _draw(self):
        w, h = self.w, self.h
        pad_l, pad_r, pad_t, pad_b = 36, 8, 20, 16
        gw = w - pad_l - pad_r
        gh = h - pad_t - pad_b
        bottom_y = pad_t + gh

        # Background rounded rect
        self.create_rectangle(0, 0, w, h, fill=COLORS["bg_card"], outline="")

        # Current value for color
        cur_val = self.data[-1] if self.data else None
        line_color = self._get_color(cur_val)

        # Label (left) + current value badge (right)
        self.create_text(pad_l, 10, text=self.label, anchor="w",
                         font=FONTS["small_bold"], fill=COLORS["text_secondary"])
        if cur_val is not None:
            val_text = f"{cur_val:.0f} {self.unit}"
            self.create_text(w - pad_r, 10, text=val_text, anchor="e",
                             font=FONTS["small_bold"], fill=line_color)
        else:
            self.create_text(w - pad_r, 10, text=f"-- {self.unit}", anchor="e",
                             font=FONTS["small"], fill=COLORS["text_muted"])

        # Grid lines (4 horizontal)
        val_range = max(self.max_val - self.min_val, 1)
        for i in range(4):
            y = pad_t + int(gh * i / 3)
            self.create_line(pad_l, y, pad_l + gw, y,
                             fill=COLORS["border"], dash=(1, 3))
            val = self.max_val - (self.max_val - self.min_val) * i / 3
            self.create_text(pad_l - 4, y, text=f"{int(val)}", anchor="e",
                             font=("Helvetica", 7), fill=COLORS["text_muted"])

        # Bottom axis
        self.create_line(pad_l, bottom_y, pad_l + gw, bottom_y,
                         fill=COLORS["border"])
        # Left axis
        self.create_line(pad_l, pad_t, pad_l, bottom_y,
                         fill=COLORS["border"])

        if len(self.data) < 2:
            # Empty state
            self.create_text(pad_l + gw // 2, pad_t + gh // 2, text="--",
                             font=FONTS["mono"], fill=COLORS["text_muted"])
            return

        # Compute points
        points = []
        for i, val in enumerate(self.data):
            x = pad_l + int(gw * i / (self.max_samples - 1))
            clamped = max(self.min_val, min(self.max_val, val))
            y = pad_t + gh - int(gh * (clamped - self.min_val) / val_range)
            points.append((x, y))

        # Gradient fill under curve (multiple horizontal strips)
        line_rgb = self._hex_to_rgb(line_color)
        bg_rgb = self._hex_to_rgb(COLORS["bg_card"])
        strips = 8
        for s in range(strips):
            alpha = 0.25 * (1 - s / strips)  # Fade from 25% opacity to 0%
            strip_color = self._rgb_to_hex(
                bg_rgb[0] + (line_rgb[0] - bg_rgb[0]) * alpha,
                bg_rgb[1] + (line_rgb[1] - bg_rgb[1]) * alpha,
                bg_rgb[2] + (line_rgb[2] - bg_rgb[2]) * alpha,
            )
            strip_top = pad_t + int(gh * s / strips)
            strip_bot = pad_t + int(gh * (s + 1) / strips)

            # Build polygon for this strip (clipped to curve)
            poly = []
            for x, y in points:
                cy = max(strip_top, min(strip_bot, y))
                poly.append(x)
                poly.append(cy)
            # Close polygon at bottom of strip
            poly.append(points[-1][0])
            poly.append(strip_bot)
            poly.append(points[0][0])
            poly.append(strip_bot)

            if len(poly) >= 6:
                self.create_polygon(*poly, fill=strip_color, outline="")

        # Draw main line
        flat_pts = []
        for x, y in points:
            flat_pts.extend([x, y])
        self.create_line(*flat_pts, fill=line_color, width=2, smooth=True)

        # Current value dot
        last_x, last_y = points[-1]
        dot_r = 4
        self.create_oval(last_x - dot_r, last_y - dot_r,
                         last_x + dot_r, last_y + dot_r,
                         fill=line_color, outline=COLORS["bg_card"], width=2)

        # Min/Max labels at bottom
        if len(self.data) > 5:
            min_v = min(self.data)
            max_v = max(self.data)
            self.create_text(pad_l, h - 3, text=f"min:{min_v:.0f}", anchor="w",
                             font=("Helvetica", 7), fill=COLORS["text_muted"])
            self.create_text(pad_l + gw, h - 3, text=f"max:{max_v:.0f}", anchor="e",
                             font=("Helvetica", 7), fill=COLORS["text_muted"])


# ── Data Card ──────────────────────────────────────────────────
class DataCard(ctk.CTkFrame):
    """Minimal data card with colored left accent bar."""

    def __init__(self, parent, label="", unit="", accent_color=None, **kwargs):
        super().__init__(parent, fg_color=COLORS["bg_card"],
                         corner_radius=8, height=56, **kwargs)
        self.pack_propagate(False)
        self.accent_color = accent_color or COLORS["accent"]

        # Left accent bar
        accent_bar = ctk.CTkFrame(self, fg_color=self.accent_color,
                                   width=3, corner_radius=2)
        accent_bar.pack(side="left", fill="y", padx=(0, 0), pady=6)

        # Content
        content = ctk.CTkFrame(self, fg_color="transparent")
        content.pack(side="left", fill="both", expand=True, padx=10, pady=6)

        # Label
        ctk.CTkLabel(content, text=label, font=FONTS["small"],
                      text_color=COLORS["text_muted"]).pack(anchor="w")

        # Value row
        val_frame = ctk.CTkFrame(content, fg_color="transparent")
        val_frame.pack(anchor="w", pady=(1, 0))

        self.value_label = ctk.CTkLabel(val_frame, text="--",
                                         font=FONTS["mono"],
                                         text_color=COLORS["text_primary"])
        self.value_label.pack(side="left")

        if unit:
            ctk.CTkLabel(val_frame, text=unit, font=FONTS["small"],
                          text_color=COLORS["text_muted"]).pack(side="left", padx=(4, 0))

    def set_value(self, value, color=None):
        """Update displayed value."""
        self.value_label.configure(text=str(value))
        if color:
            self.value_label.configure(text_color=color)


# Keep backward compatibility alias
StatusCard = DataCard


# ── Scroll Fix (macOS trackpad) ───────────────────────────────
def _bind_scroll_recursive(scrollable_frame):
    """Fix macOS trackpad scrolling on CTkScrollableFrame.

    CustomTkinter only handles scroll events on the scrollable frame itself,
    not on child widgets. This binds mousewheel events on ALL children so
    scrolling works wherever the cursor is.
    """
    import platform

    try:
        canvas = scrollable_frame._parent_canvas
    except AttributeError:
        return

    def _on_mousewheel(event):
        try:
            canvas.yview_scroll(int(-1 * (event.delta / 120)), "units")
        except Exception:
            pass

    def _on_mousewheel_mac(event):
        try:
            canvas.yview_scroll(int(-1 * event.delta), "units")
        except Exception:
            pass

    handler = _on_mousewheel_mac if platform.system() == "Darwin" else _on_mousewheel

    def bind_children(widget):
        try:
            widget.bind("<MouseWheel>", handler, add="+")
            if platform.system() != "Darwin":
                widget.bind("<Button-4>", lambda e: canvas.yview_scroll(-3, "units"), add="+")
                widget.bind("<Button-5>", lambda e: canvas.yview_scroll(3, "units"), add="+")
        except Exception:
            pass
        for child in widget.winfo_children():
            bind_children(child)

    bind_children(scrollable_frame)
