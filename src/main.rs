#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fmt;
use std::fmt::{Display, Formatter};
use eframe::egui::{CentralPanel, Color32, ComboBox, DragValue, FontFamily, FontId, TextStyle, Ui, ViewportBuilder, Visuals};
use eframe::{egui, NativeOptions};
use egui_plot::{Line, Plot, Points, Polygon};

fn main() -> eframe::Result {
	env_logger::init();
	
	let options = NativeOptions {
		viewport: ViewportBuilder::default()
			.with_inner_size([1600.0, 800.0])
			.with_min_inner_size([1600.0, 800.0]),
		..Default::default()
	};
	
	eframe::run_native(
		"Lab4",
		options,
		Box::new(|cc| {
			egui_extras::install_image_loaders(&cc.egui_ctx);
			if cc.egui_ctx.style().visuals.dark_mode {
				cc.egui_ctx.set_visuals(Visuals {
					override_text_color: Some(Color32::from_rgb(255, 255, 255)),
					..cc.egui_ctx.style().visuals.clone()
				});
			}
			let mut style = (*cc.egui_ctx.style()).clone();
			style.text_styles.insert(
				TextStyle::Body,
				FontId::new(16.0, FontFamily::Proportional),
			);
			cc.egui_ctx.set_style(style);
			Ok(Box::<MyApp>::default())
		}),
	)
}

#[derive(Clone, Copy, PartialEq)]
enum Cmp {
	Gte,
	Lte,
}

impl Display for Cmp {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Cmp::Gte => write!(f, ">="),
			Cmp::Lte => write!(f, "<="),
		}
	}
}

#[derive(Clone, Copy)]
struct Equation {
	a: f64,
	b: f64,
	c: f64,
	cmp: Cmp,
}

#[derive(PartialEq)]
enum Limit {
	Max,
	Min
}

impl Display for Limit {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Limit::Max => write!(f, "Max"),
			Limit::Min => write!(f, "Min"),
		}
	}
}

struct FinalEquation {
	a: f64,
	b: f64,
	limit: Limit
}

struct MyApp {
	final_equation: FinalEquation,
	equations: [Equation; 3],
}

impl Default for MyApp {
	fn default() -> Self {
		Self {
			final_equation: FinalEquation { a: 1.0, b: 1.0, limit: Limit::Min },
			equations: [Equation { a: 1.0, b: 1.0, c: 1.0, cmp: Cmp::Gte }; 3]
		}
	}
}

fn add_drag_value(ui: &mut Ui, value: &mut f64, label: &str,) {
	let label = ui.label(label);
	ui.add_sized([60.0, 15.0], DragValue::new(value).speed(0.1)).labelled_by(label.id);
}

fn change_basis(simple_tab: &mut [[f64; 6]; 3], basis: &mut [usize; 3], i: usize, j: usize) {
	basis[i] = j;
	let del = simple_tab[i][j];
	for item in simple_tab[i].iter_mut() {
		*item /= del;
	}
	
	for p in 0..3 {
		if p != i {
			let multi = simple_tab[p][j];
			for q in 0..6 {
				simple_tab[p][q] -= multi * simple_tab[i][q];
			}
		}
	}
}

impl eframe::App for MyApp {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		CentralPanel::default().show(ctx, |ui| {
			ui.horizontal(|ui| {
				add_drag_value(ui, &mut self.final_equation.a, "z = ");
				add_drag_value(ui, &mut self.final_equation.b, "x1 + ");
				ui.label("x2 -> ");
				ComboBox::from_id_salt(3)
					.selected_text(self.final_equation.limit.to_string())
					.width(20.0)
					.show_ui(ui, |ui| {
						ui.selectable_value(&mut self.final_equation.limit, Limit::Max, "Max");
						ui.selectable_value(&mut self.final_equation.limit, Limit::Min, "Min");
					});
			});
			for i in 0..3 {
				ui.horizontal(|ui| {
					add_drag_value(ui, &mut self.equations[i].a, "");
					add_drag_value(ui, &mut self.equations[i].b, "x1 + ");
					ui.label("x2 ");
					ComboBox::from_id_salt(i)
						.selected_text(self.equations[i].cmp.to_string())
						.width(15.0)
						.show_ui(ui, |ui| {
							ui.selectable_value(&mut self.equations[i].cmp, Cmp::Gte, ">=");
							ui.selectable_value(&mut self.equations[i].cmp, Cmp::Lte, "<=");
						});
					add_drag_value(ui, &mut self.equations[i].c, "");
				});
			}
			
			let mut simple_tab = [[0.0; 6]; 3];
			
			for (i, tab) in simple_tab.iter_mut().enumerate() {
				if self.equations[i].cmp == Cmp::Gte {
					tab[0] = self.equations[i].a * -1.0;
					tab[1] = self.equations[i].b * -1.0;
					tab[5] = self.equations[i].c * -1.0;
				} else {
					tab[0] = self.equations[i].a;
					tab[1] = self.equations[i].b;
					tab[5] = self.equations[i].c;
				}
			}
			
			simple_tab[0][2] = 1.0;
			simple_tab[1][3] = 1.0;
			simple_tab[2][4] = 1.0;
			
			let mut basis = [2, 3, 4];
			
			let mut can = true;
			
			loop {
				let mut min = 0.0;
				let mut min_i = None;
				
				for (i, tab) in simple_tab.iter().enumerate() {
					if tab[5] < min {
						min = tab[5];
						min_i = Some(i);
					}
				}
				
				if let Some(i) = min_i {
					let mut min = 0.0;
					let mut min_j = None;
					for j in 0..5 {
						if simple_tab[i][j] < min {
							min = simple_tab[i][j];
							min_j = Some(j);
						}
					}
					
					if let Some(j) = min_j {
						change_basis(&mut simple_tab, &mut basis, i, j);
					} else {
						can = false;
						break;
					}
				} else {
					break;
				}
			}
			
			let mut result = None;
			
			if can {
				can = false;
				for _ in 0..20 {
					let mut delta = [0.0; 6];
					
					for (i, item_i) in simple_tab.iter().enumerate() {
						for (j, item_j) in delta.iter_mut().enumerate() {
							if basis[i] == 0 {
								*item_j += item_i[j] * self.final_equation.a;
							} else if basis[i] == 1 {
								*item_j += item_i[j] * self.final_equation.b;
							}
						}
					}
					delta[0] -= self.final_equation.a;
					delta[1] -= self.final_equation.b;
					
					match self.final_equation.limit {
						Limit::Max => {
							if delta.iter().all(|&x| x >= 0.0) {
								can = true;
								break;
							}
						}
						Limit::Min => {
							if delta.iter().all(|&x| x <= 0.0) {
								can = true;
								break;
							}
						}
					}
					
					let mut max_min = 0.0;
					let mut max_min_j = 0;
					for (j, item) in delta.iter().enumerate().take(5) {
						match self.final_equation.limit {
							Limit::Max => {
								if *item < max_min {
									max_min = *item;
									max_min_j = j;
								}
							}
							Limit::Min => {
								if *item > max_min {
									max_min = *item;
									max_min_j = j;
								}
							}
						}
					}
					
					let mut min = f64::INFINITY;
					let mut min_i = None;
					for (i, item) in simple_tab.iter().enumerate() {
						if item[max_min_j] > 0.0 && item[5] / item[max_min_j] < min {
							min = item[5] / item[max_min_j];
							min_i = Some(i);
						}
					}
					
					if let Some(i) = min_i {
						change_basis(&mut simple_tab, &mut basis, i, max_min_j);
					} else {
						break;
					}
				}
				
				if can {
					let mut x = [0.0; 5];
					for (i, item) in simple_tab.iter().enumerate() {
						x[basis[i]] = item[5];
					}
					
					result = Some([x[0], x[1]]);
				}
			}
			
			Plot::new("plot")
				.auto_bounds(false)
				.x_axis_label("x1")
				.y_axis_label("x2")
				.show(ui, |plot| {
					for i in 0..3 {
						let x1 = 100.0;
						let x2 = (self.equations[i].c - self.equations[i].a * x1) / self.equations[i].b;
						let x1_2 = -100.0;
						let x2_2 = (self.equations[i].c - self.equations[i].a * x1_2) / self.equations[i].b;
						plot.add(Line::new(Vec::from([[x1, x2], [x1_2, x2_2]]))
							.color(Color32::BLUE)
							.width(2.0)
						);
						let vec1 = [x1 - x1_2, x2 - x2_2];
						let mut vec1 = [-vec1[1], vec1[0]];
						if self.equations[i].cmp == Cmp::Lte {
							vec1[0] *= -1.0;
							vec1[1] *= -1.0;
						}
						plot.add(Polygon::new(Vec::from([[x1, x2], [x1_2, x2_2], [x1_2 + vec1[0], x2_2 + vec1[1]], [x1 + vec1[0], x2 + vec1[1]]]))
							.fill_color(Color32::from_rgba_premultiplied(20, 0, 0, 10)))
					}
					
					if let Some(result) = result {
						plot.add(Points::new([result[0], result[1]]).color(Color32::RED).radius(5.0));
						
						let x2_1 = (result[0] - 100.0) * -self.final_equation.a / self.final_equation.b;
						let x2_2 = (result[0] + 100.0) * -self.final_equation.a / self.final_equation.b;
						plot.add(Line::new(Vec::from([
							[2.0 * result[0] - 100.0, x2_1 + result[1]],
							[2.0 * result[0] + 100.0, x2_2 + result[1]]
						]))
							.color(Color32::GREEN)
							.width(2.0));
					}
				})
		});
	}
}