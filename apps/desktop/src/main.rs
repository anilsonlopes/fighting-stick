use eframe::egui::{self, Color32, RichText};
use fighting_stick_core::{
    load_profile, note_name, save_profile, MidiState, PhysicalInput, Profile, APP_DIR,
};
use fighting_stick_input::{InputEvent, InputManager};
use fighting_stick_midi::MidiOut;
use std::{collections::BTreeSet, path::PathBuf, time::Duration};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([850.0, 650.0])
            .with_min_inner_size([680.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Fighting Stick MIDI",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

struct App {
    input: Option<InputManager>,
    midi: MidiOut,
    state: MidiState,
    profile: Option<Profile>,
    profile_dir: PathBuf,
    ports: Vec<String>,
    connected: bool,
    active_inputs: BTreeSet<PhysicalInput>,
    learn_row: Option<usize>,
    diagnostic: Vec<String>,
    status: String,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let base = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join(APP_DIR);
        let (input, status) = match InputManager::new() {
            Ok(i) => (Some(i), "Conecte o Fighting Stick".into()),
            Err(e) => (None, e.to_string()),
        };
        let ports = MidiOut::ports().unwrap_or_default();
        Self {
            input,
            midi: MidiOut::new(),
            state: MidiState::default(),
            profile: None,
            profile_dir: base,
            ports,
            connected: false,
            active_inputs: BTreeSet::new(),
            learn_row: None,
            diagnostic: vec![],
            status,
        }
    }

    fn send(&mut self, commands: Vec<fighting_stick_core::MidiCommand>) {
        for c in commands {
            if let Err(e) = self.midi.send(c) {
                self.status = e.to_string();
            }
        }
    }

    fn silence(&mut self) {
        let commands = self.state.panic();
        self.send(commands);
        self.active_inputs.clear();
    }

    fn handle(&mut self, event: InputEvent) {
        match event {
            InputEvent::Connected { guid, name, .. } => {
                self.silence();
                self.connected = true;
                let p = load_profile(&self.profile_dir, &guid)
                    .unwrap_or_else(|_| Profile::default_for(guid, name.clone()));
                let midi_error = p
                    .midi_port
                    .clone()
                    .and_then(|port| self.midi.connect(&port).err());
                self.status = midi_error
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| format!("Conectado: {name}"));
                self.profile = Some(p);
            }
            InputEvent::Disconnected => {
                self.silence();
                self.connected = false;
                self.status = "Controle desconectado — notas encerradas".into();
            }
            InputEvent::Diagnostic(s) => {
                self.diagnostic.push(s);
            }
            InputEvent::Changed { input, pressed } => {
                if pressed {
                    self.active_inputs.insert(input.clone());
                } else {
                    self.active_inputs.remove(&input);
                }
                self.diagnostic.push(format!(
                    "{}: {}",
                    input,
                    if pressed { "pressionado" } else { "solto" }
                ));
                if self.diagnostic.len() > 12 {
                    self.diagnostic.remove(0);
                }
                if pressed {
                    if let Some(row) = self.learn_row.take() {
                        if let Some(p) = &mut self.profile {
                            let note = p.assignments[row].note;
                            match p.learn(row, input, note) {
                                Ok(_) => self.status = "Entrada aprendida".into(),
                                Err(e) => self.status = e.to_string(),
                            }
                        }
                        return;
                    }
                }
                if let Some(p) = &self.profile {
                    let commands = self.state.input(p, input, pressed);
                    self.send(commands);
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let events = self
            .input
            .as_mut()
            .map(InputManager::scan)
            .unwrap_or_default();
        for event in events {
            self.handle(event);
        }
        ctx.request_repaint_after(Duration::from_millis(8));

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Fighting Stick MIDI");
                let color = if self.connected {
                    Color32::LIGHT_GREEN
                } else {
                    Color32::YELLOW
                };
                ui.label(RichText::new(format!("● {}", self.status)).color(color));
                ui.menu_button("Ajuda", |ui| {
                    ui.label(format!("Versão {}", env!("CARGO_PKG_VERSION")));
                    ui.separator();
                    ui.hyperlink_to(
                        "Como usar",
                        "https://github.com/anilsonlopes/fighting-stick#readme",
                    );
                    ui.hyperlink_to(
                        "Configurar no Ableton Live",
                        "https://github.com/anilsonlopes/fighting-stick/blob/main/docs/ABLETON.md",
                    );
                    ui.hyperlink_to(
                        "Solução de problemas",
                        "https://github.com/anilsonlopes/fighting-stick/blob/main/docs/TROUBLESHOOTING.md",
                    );
                    ui.hyperlink_to(
                        "Downloads e atualizações",
                        "https://github.com/anilsonlopes/fighting-stick/releases",
                    );
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Saída MIDI:");
                let selected = self
                    .profile
                    .as_ref()
                    .and_then(|p| p.midi_port.clone())
                    .unwrap_or_else(|| "Selecione a porta loopMIDI".into());
                egui::ComboBox::from_id_salt("midi_port")
                    .selected_text(&selected)
                    .show_ui(ui, |ui| {
                        for port in self.ports.clone() {
                            if ui.selectable_label(port == selected, &port).clicked() {
                                self.silence();
                                match self.midi.connect(&port) {
                                    Ok(()) => {
                                        if let Some(p) = &mut self.profile {
                                            p.midi_port = Some(port.clone());
                                        }
                                        self.status = format!("Saída MIDI: {port}");
                                    }
                                    Err(e) => self.status = e.to_string(),
                                }
                            }
                        }
                    });
                if ui.button("Atualizar portas").clicked() {
                    self.ports = MidiOut::ports().unwrap_or_default();
                }
            });

            if let Some(mut p) = self.profile.take() {
                let mut restore_default = false;
                ui.horizontal(|ui| {
                    ui.label("Velocidade:");
                    ui.add(egui::Slider::new(&mut p.velocity, 1..=127));
                });
                ui.separator();
                ui.heading("Mapeamento");
                egui::Grid::new("mapping")
                    .striped(true)
                    .min_col_width(130.0)
                    .show(ui, |ui| {
                        ui.strong("Entrada física");
                        ui.strong("Nota");
                        ui.strong("Nome");
                        ui.strong("Learn");
                        ui.end_row();
                        for row in 0..p.assignments.len() {
                            let input = p.assignments[row].input.clone();
                            let active = self.active_inputs.contains(&input);
                            ui.label(RichText::new(input.to_string()).color(if active {
                                Color32::LIGHT_GREEN
                            } else {
                                ui.visuals().text_color()
                            }));
                            let old = p.assignments[row].note;
                            let mut candidate = old;
                            ui.add(egui::DragValue::new(&mut candidate).range(0..=127));
                            if candidate != old {
                                if p.assignments
                                    .iter()
                                    .enumerate()
                                    .any(|(i, other)| i != row && other.note == candidate)
                                {
                                    self.status = format!("A nota MIDI {candidate} já está em uso");
                                } else {
                                    p.assignments[row].note = candidate;
                                    p.assignments[row].label = note_name(candidate);
                                }
                            }
                            ui.label(&p.assignments[row].label);
                            let learning = self.learn_row == Some(row);
                            if ui
                                .selectable_label(
                                    learning,
                                    if learning { "Pressione…" } else { "Aprender" },
                                )
                                .clicked()
                            {
                                self.learn_row = if learning { None } else { Some(row) };
                            }
                            ui.end_row();
                        }
                    });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Salvar perfil").clicked() {
                        match save_profile(&self.profile_dir, &p) {
                            Ok(_) => self.status = "Perfil salvo".into(),
                            Err(e) => self.status = e.to_string(),
                        }
                    }
                    if ui.button("Restaurar padrão").clicked() {
                        self.silence();
                        restore_default = true;
                        self.status = "Mapeamento padrão restaurado".into();
                    }
                    if ui.button("Parar todas as notas").clicked() {
                        self.silence();
                        self.status = "Todas as notas encerradas".into();
                    }
                });
                self.profile = Some(if restore_default {
                    Profile::default_for(p.controller_guid, p.controller_name)
                } else {
                    p
                });
            } else {
                ui.add_space(24.0);
                ui.label("Conecte um controle compatível para criar ou carregar seu perfil.");
            }

            ui.separator();
            ui.collapsing("Diagnóstico de entradas", |ui| {
                if self.diagnostic.is_empty() {
                    ui.label("Pressione botões para ver os eventos aqui.");
                }
                for line in self.diagnostic.iter().rev() {
                    ui.monospace(line);
                }
            });
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.silence();
    }
}
