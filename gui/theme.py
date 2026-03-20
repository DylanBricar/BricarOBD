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
        self.current_value = min_val
        self._draw_gauge()

    def set_value(self, value):
        """Update gauge value."""
        self.current_value = max(self.min_val, min(self.max_val, value))
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
        pct = (self.current_value - self.min_val) / max(self.max_val - self.min_val, 1)
        fill_sweep = sweep * pct

        # Track arc
        self.create_arc(cx - r, cx - r, cx + r, cx + r,
                        start=start, extent=-sweep,
                        outline=COLORS["gauge_track"], width=arc_w, style="arc")

        # Fill arc
        if fill_sweep > 0:
            self.create_arc(cx - r, cx - r, cx + r, cx + r,
                            start=start, extent=-fill_sweep,
                            outline=self._get_color(), width=arc_w, style="arc")

        # Value text
        val_text = f"{int(self.current_value)}"
        self.create_text(cx, cx - 4, text=val_text,
                         font=FONTS["mono_large"], fill=COLORS["text_primary"])

        # Unit below value
        self.create_text(cx, cx + 20, text=self.unit,
                         font=FONTS["small"], fill=COLORS["text_muted"])

        # Label at top
        self.create_text(cx, 14, text=self.label,
                         font=FONTS["small"], fill=COLORS["text_secondary"])


# ── Data Card ──────────────────────────────────────────────────
class DataCard(ctk.CTkFrame):
    """Minimal data card with colored left accent bar."""

    def __init__(self, parent, label="", unit="", accent_color=None, **kwargs):
        super().__init__(parent, fg_color=COLORS["bg_card"],
                         corner_radius=10, **kwargs)
        self.accent_color = accent_color or COLORS["accent"]

        # Left accent bar
        accent_bar = ctk.CTkFrame(self, fg_color=self.accent_color,
                                   width=3, corner_radius=2)
        accent_bar.pack(side="left", fill="y", padx=(0, 0), pady=8)

        # Content
        content = ctk.CTkFrame(self, fg_color="transparent")
        content.pack(side="left", fill="both", expand=True, padx=12, pady=8)

        # Label
        ctk.CTkLabel(content, text=label, font=FONTS["small"],
                      text_color=COLORS["text_muted"]).pack(anchor="w")

        # Value row
        val_frame = ctk.CTkFrame(content, fg_color="transparent")
        val_frame.pack(anchor="w", pady=(2, 0))

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
