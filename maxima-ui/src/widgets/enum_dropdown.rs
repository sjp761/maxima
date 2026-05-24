use egui::{Color32, CornerRadius, Stroke};

use crate::enum_locale_map::EnumToString;
use strum::IntoEnumIterator;

const F9B233: Color32 = Color32::from_rgb(249, 178, 51);
const DARK_GREY: Color32 = Color32::from_rgb(64, 64, 64);

pub fn enum_dropdown<T>(
    ui: &mut egui::Ui,
    id: String,
    val: &mut T,
    width: f32,
    label: &str,
    enum_holder: &impl EnumToString<T>,
) -> egui::InnerResponse<Option<()>>
where
    T: IntoEnumIterator + PartialEq,
{
    puffin::profile_function!();
    ui.visuals_mut().extreme_bg_color = Color32::TRANSPARENT;

    ui.visuals_mut().widgets.inactive.expansion = 0.0;
    ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
    ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    ui.visuals_mut().widgets.inactive.fg_stroke = Stroke::new(2.0_f32, Color32::WHITE);
    ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(2.0_f32, DARK_GREY);
    ui.visuals_mut().widgets.inactive.corner_radius = CornerRadius::same(2);

    ui.visuals_mut().widgets.active.bg_fill = Color32::TRANSPARENT;
    ui.visuals_mut().widgets.active.weak_bg_fill = Color32::TRANSPARENT;
    ui.visuals_mut().widgets.active.fg_stroke = Stroke::new(2.0_f32, Color32::WHITE);
    ui.visuals_mut().widgets.active.bg_stroke = Stroke::new(2.0_f32, DARK_GREY);
    ui.visuals_mut().widgets.active.corner_radius = CornerRadius::same(2);

    ui.visuals_mut().widgets.hovered.bg_fill = Color32::TRANSPARENT;
    ui.visuals_mut().widgets.hovered.weak_bg_fill = Color32::TRANSPARENT;
    ui.visuals_mut().widgets.hovered.fg_stroke = Stroke::new(2.0_f32, F9B233);
    ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::new(2.0_f32, F9B233);
    ui.visuals_mut().widgets.hovered.corner_radius = CornerRadius::same(2);

    ui.visuals_mut().widgets.open.bg_fill = DARK_GREY;
    ui.visuals_mut().widgets.open.weak_bg_fill = DARK_GREY;
    ui.visuals_mut().widgets.open.fg_stroke = Stroke::new(2.0_f32, Color32::WHITE);
    ui.visuals_mut().widgets.open.bg_stroke = Stroke::new(2.0_f32, DARK_GREY);
    ui.visuals_mut().widgets.open.corner_radius = CornerRadius::same(2);

    egui::ComboBox::new(id + "combo", label)
        .width(width)
        .selected_text(enum_holder.get_string(val))
        .show_ui(ui, |contents| {
            let corner_radius = CornerRadius::same(2);
            contents.visuals_mut().widgets.inactive.corner_radius = corner_radius;
            contents.visuals_mut().widgets.active.corner_radius = corner_radius;
            contents.visuals_mut().widgets.hovered.corner_radius = corner_radius;
            contents.visuals_mut().selection.bg_fill = F9B233;
            contents.visuals_mut().selection.stroke = Stroke::new(2.0_f32, Color32::BLACK);
            contents.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(2.0_f32, F9B233);
            for iter in T::iter() {
                let display = enum_holder.get_string_nonmut(&iter);
                contents.selectable_value(val, iter, display);
            }
        })
}
