use eframe::egui;

const MILLIWATTS_PER_WATT: f64 = 1000.0;
const DECIBEL_POWER_FACTOR: f64 = 10.0;
const DECIBEL_AMPLITUDE_FACTOR: f64 = 20.0;
const WINDOWS_VOLUME_GAMMA: f64 = 2.2;

pub struct OhmsApp {
    dac: DAC,
    headphone: Headphone,
    sensitivity_unit: SensitivityUnit,
    target_volume: f64,
}

impl Default for OhmsApp {
    fn default() -> Self {
        Self {
            dac: DAC { max_voltage: 1.0 },
            headphone: Headphone {
                impedance: 23.5,
                sensitivity_decibels_per_milliwatt: 108.0,
            },
            sensitivity_unit: SensitivityUnit::DecibelsPerMilliwatt,
            target_volume: 77.0,
        }
    }
}

impl OhmsApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }

    fn calculate_results(&self) -> CalculationResult {
        let max_power =
            (self.dac.max_voltage.powi(2) / self.headphone.impedance) * MILLIWATTS_PER_WATT;
        let max_decibels_of_sound_pressure_level =
            self.headphone.sensitivity_decibels_per_milliwatt
                + DECIBEL_POWER_FACTOR * max_power.log10();
        let attenuation = self.target_volume - max_decibels_of_sound_pressure_level;
        let linear_scalar = 10.0_f64.powf(attenuation / DECIBEL_AMPLITUDE_FACTOR);
        let wpctl_volume = linear_scalar.cbrt().clamp(0.0, 1.0);
        let windows_volume = (linear_scalar.powf(1.0 / WINDOWS_VOLUME_GAMMA) * 100.0)
            .clamp(0.0, 100.0)
            .floor();

        CalculationResult {
            windows_volume,
            wpctl_volume,
        }
    }
}

impl eframe::App for OhmsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Specifications");
            specifications_widget(self, ui);
            ui.separator();
            results_widget(self, ui);
        });
    }
}

#[derive(PartialEq)]
enum SensitivityUnit {
    DecibelsPerMilliwatt,
    DecibelsPerVolt,
}

struct Headphone {
    impedance: f64,
    sensitivity_decibels_per_milliwatt: f64,
}

impl Headphone {
    fn get_sensitivity_decibels_per_volt(&self) -> f64 {
        self.sensitivity_decibels_per_milliwatt
            + 10.0 * (MILLIWATTS_PER_WATT / self.impedance).log10()
    }

    fn set_sensitivity_decibels_per_volt(&mut self, decibels_per_volt: f64) {
        self.sensitivity_decibels_per_milliwatt =
            decibels_per_volt - 10.0 * (MILLIWATTS_PER_WATT / self.impedance).log10()
    }
}

struct DAC {
    max_voltage: f64,
}

struct CalculationResult {
    windows_volume: f64,
    wpctl_volume: f64,
}

pub fn specifications_widget(app: &mut OhmsApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("Headphone Impedance:");
        ui.add(
            egui::DragValue::new(&mut app.headphone.impedance)
                .range(0.0..=f64::MAX)
                .suffix(" Ω"),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Headphone Sensitivity:");
        match app.sensitivity_unit {
            SensitivityUnit::DecibelsPerMilliwatt => {
                ui.add(
                    egui::DragValue::new(&mut app.headphone.sensitivity_decibels_per_milliwatt)
                        .range(0.0..=f64::MAX)
                        .suffix(" dB/mW"),
                );
            }
            SensitivityUnit::DecibelsPerVolt => {
                let mut decibels_per_volt = app.headphone.get_sensitivity_decibels_per_volt();
                let response = ui.add(egui::DragValue::new(&mut decibels_per_volt).suffix(" dB/V"));

                if response.changed() {
                    app.headphone
                        .set_sensitivity_decibels_per_volt(decibels_per_volt);
                }
            }
        };
        ui.radio_value(
            &mut app.sensitivity_unit,
            SensitivityUnit::DecibelsPerMilliwatt,
            "dB/mW",
        );
        ui.radio_value(
            &mut app.sensitivity_unit,
            SensitivityUnit::DecibelsPerVolt,
            "dB/V",
        );
    });
    ui.horizontal(|ui| {
        ui.label("DAC Max Voltage:");
        ui.add(
            egui::DragValue::new(&mut app.dac.max_voltage)
                .range(0.0..=f64::MAX)
                .suffix(" Vrms"),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Target Volume:");
        ui.add(
            egui::DragValue::new(&mut app.target_volume)
                .range(0.0..=f64::MAX)
                .suffix(" dB SPL"),
        );
    });
}

pub fn results_widget(app: &OhmsApp, ui: &mut egui::Ui) {
    ui.heading("Results");
    let results = app.calculate_results();
    ui.horizontal(|ui| {
        let label = ui.label("wpctl Command:");
        ui.monospace(format!(
            "wpctl set-volume @DEFAULT_AUDIO_SINK@ {:.2}",
            results.wpctl_volume
        ))
        .labelled_by(label.id);
    });
    ui.horizontal(|ui| {
        let label = ui.label("Windows Volume:");
        ui.label(format!("{}", results.windows_volume))
            .labelled_by(label.id);
    });
}
