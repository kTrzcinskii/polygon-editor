pub struct Popups {
    // Const Width Constraint popup fields
    const_width_constraint_popup_id: egui::Id,
    const_width_constraint_user_input: f32,
    const_width_constraint_submitted: bool,
}

impl Popups {
    pub fn open_const_width_constraint_popup_below_widget(
        &mut self,
        ui: &mut egui::Ui,
        intial_width: f32,
    ) {
        ui.memory_mut(|mem| mem.toggle_popup(self.const_width_constraint_popup_id));
        self.const_width_constraint_user_input = intial_width;
    }

    pub fn render_const_width_constraint_popup_below_widget(
        &mut self,
        ui: &mut egui::Ui,
        widget: &egui::Response,
    ) {
        egui::popup_below_widget(
            ui,
            self.const_width_constraint_popup_id,
            widget,
            egui::PopupCloseBehavior::CloseOnClickOutside,
            |ui| {
                ui.horizontal(|ui| {
                    ui.label("Enter width");
                    ui.add(egui::DragValue::new(
                        &mut self.const_width_constraint_user_input,
                    ))
                });
                if ui.button("Apply").clicked() {
                    ui.memory_mut(|mem| mem.toggle_popup(self.const_width_constraint_popup_id));
                    self.const_width_constraint_submitted = true;
                }
            },
        );
    }

    pub fn const_width_constraint_submitted(&self) -> bool {
        self.const_width_constraint_submitted
    }

    pub fn const_width_constraint_user_input(&self) -> f32 {
        self.const_width_constraint_user_input
    }

    pub fn reset_const_width_constraint_submitted(&mut self) {
        self.const_width_constraint_submitted = false;
    }
}

impl Default for Popups {
    fn default() -> Self {
        Self {
            const_width_constraint_popup_id: "const_width_constraint_popup_id".into(),
            const_width_constraint_user_input: 0.0,
            const_width_constraint_submitted: false,
        }
    }
}
