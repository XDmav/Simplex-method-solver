#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter};
use eframe::egui::{CentralPanel, Color32, ComboBox, DragValue, FontFamily, FontId, IconData, ScrollArea, TextStyle, Ui, ViewportBuilder, Visuals};
use eframe::{egui, NativeOptions};
use egui_extras::{Column, TableBuilder};
use egui_plot::{Line, Plot, Points, Polygon};

fn main() -> eframe::Result {
	env_logger::init();
	
	let options = NativeOptions {
		viewport: ViewportBuilder::default()
			.with_icon(IconData::default())
			.with_inner_size([1600.0, 800.0])
			.with_min_inner_size([1600.0, 800.0]),
		..Default::default()
	};
	
	eframe::run_native(
		"Lab4",
		options,
		Box::new(|cc| {
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
			
			let mut result = Box::<MyApp>::default();
			result.update_simple_tab();
			Ok(result)
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
	cof: [f64; NUM_I + 1],
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

const NUM_I: usize = 4;
const NUM_J: usize = 3;

struct FinalEquation {
	cof: [f64; NUM_I],
	limit: Limit
}

struct MyApp {
	final_equation: FinalEquation,
	equations: [Equation; NUM_J],
	result: Option<[f64; NUM_I + NUM_J]>,
	simple_tab_history: Vec<[[f64; NUM_I + NUM_J + 1]; NUM_J]>,
	basis_history: Vec<[usize; NUM_J]>,
	delta_history: Vec<[f64; NUM_I + NUM_J + 1]>,
	dots: Vec<[f64; 2]>,
}

impl Default for MyApp {
	fn default() -> Self {
		Self {
			final_equation: FinalEquation { cof: [7.0, 8.0, 6.0, 5.0], limit: Limit::Max },
			equations: [
				Equation { cof: [1.0, 3.0, 5.0, 3.0, 40.0], cmp: Cmp::Lte },
				Equation { cof: [2.0, 6.0, 1.0, 0.0, 50.0], cmp: Cmp::Lte },
				Equation { cof: [2.0, 3.0, 2.0, 5.0, 30.0], cmp: Cmp::Lte },
			],
			result: None,
			simple_tab_history: Vec::new(),
			basis_history: Vec::new(),
			delta_history: Vec::new(),
			dots: Vec::new(),
		}
	}
}

fn add_drag_value(ui: &mut Ui, value: &mut f64, label: &str, changed: &mut bool) {
	let label = ui.label(label);
	if ui.add_sized([60.0, 15.0], DragValue::new(value).speed(0.1)).labelled_by(label.id).changed() {
		*changed = true
	};
}

impl MyApp {
	fn change_basis(&mut self, simple_tab: &mut [[f64; NUM_I + NUM_J + 1]; NUM_J], basis: &mut [usize; NUM_J], i: usize, j: usize) {
		basis[i] = j;
		let del = simple_tab[i][j];
		for item in simple_tab[i].iter_mut() {
			*item /= del;
		}
		
		for p in 0..NUM_J {
			if p != i {
				let multi = simple_tab[p][j];
				for q in 0..NUM_I + NUM_J + 1 {
					simple_tab[p][q] -= multi * simple_tab[i][q];
				}
			}
		}
		
		self.simple_tab_history.push(*simple_tab);
		self.basis_history.push(*basis);
	}
	
	fn is_optimal(&self, delta: &[f64; NUM_I + NUM_J + 1]) -> bool {
		match self.final_equation.limit {
			Limit::Max => {
				if delta.iter().all(|&x| x >= 0.0) {
					return true
				}
			}
			Limit::Min => {
				if delta.iter().all(|&x| x <= 0.0) {
					return true
				}
			}
		}
		
		false
	}
	
	fn if_valid(&self, x: f64, y: f64) -> bool {
		if x < -0.00001 || y < -0.00001 || x > 10000.00001 || y > 10000.00001 {
			return false
		}
		
		for equation in self.equations {
			let value = x * equation.cof[0] + y * equation.cof[1];
			
			match equation.cmp {
				Cmp::Gte => {
					if value < equation.cof[2] {
						return false
					}
				}
				Cmp::Lte => {
					if value > equation.cof[2] {
						return false
					}
				}
			}
		}
		
		true
	}
	
	fn find_intersection(&self, a1: f64, b1: f64, c1: f64, a2: f64, b2: f64, c2: f64) -> Option<[f64; 2]> {
		let determinant = a1 * b2 - a2 * b1;
		if determinant == 0.0 {
			return None
		}
		
		let x = (b2 * c1 - b1 * c2) / determinant;
		let y = (a1 * c2 - a2 * c1) / determinant;
		
		if !self.if_valid(x, y) {
			return None
		}
		
		Some([x, y])
	}
	
	fn update_simple_tab(&mut self) {
		self.dots = Vec::new();
		
		if NUM_I == 2 {
			for equation_1 in self.equations {
				for equation_2 in self.equations {
					let cord = self.find_intersection(equation_1.cof[0], equation_1.cof[1], equation_1.cof[2],
					                                  equation_2.cof[0], equation_2.cof[1], equation_2.cof[2]);
					if let Some(cord) = cord {
						self.dots.push(cord);
					}
				}
				
				for equation_2 in [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 10000.0], [1.0, 0.0, 10000.0]] {
					let cord = self.find_intersection(equation_1.cof[0], equation_1.cof[1], equation_1.cof[2],
					                                  equation_2[0], equation_2[1], equation_2[2]);
					if let Some(cord) = cord {
						self.dots.push(cord);
					}
				}
			}
			
			for cord in [[0.0, 0.0], [10000.0, 0.0], [10000.0, 10000.0], [0.0, 10000.0]] {
				if self.if_valid(cord[0], cord[1]) {
					self.dots.push(cord);
				}
			}
			
			let mut centroid_x = 0.0;
			for dot in self.dots.iter() {
				centroid_x += dot[0]
			}
			centroid_x /= self.dots.len() as f64;
			
			let mut centroid_y = 0.0;
			for dot in self.dots.iter() {
				centroid_y += dot[1]
			}
			centroid_y /= self.dots.len() as f64;
			
			self.dots.sort_by(|x1, x2| {
				if (x1[1] - centroid_y).atan2(x1[0] - centroid_x) < (x2[1] - centroid_y).atan2(x2[0] - centroid_x) {
					return Ordering::Less
				}
				Ordering::Greater
			});
		}
		
		let mut simple_tab = [[0.0; NUM_I + NUM_J + 1]; NUM_J];
		
		for (i, tab) in simple_tab.iter_mut().enumerate() {
			if self.equations[i].cmp == Cmp::Gte {
				for (j, item_j) in tab.iter_mut().enumerate().take(NUM_I) {
					*item_j = self.equations[i].cof[j] * -1.0;
				}
				tab[NUM_I + NUM_J] = self.equations[i].cof[NUM_I] * -1.0;
			} else {
				tab[..NUM_I].copy_from_slice(&self.equations[i].cof[..NUM_I]);
				tab[NUM_I + NUM_J] = self.equations[i].cof[NUM_I];
			}
		}
		
		for (i, item_i) in simple_tab.iter_mut().enumerate() {
			item_i[NUM_I + i] = 1.0;
		}
		
		self.simple_tab_history = vec![simple_tab];
		
		let mut basis = [0; NUM_J];
		
		for (j, item_j) in basis.iter_mut().enumerate() {
			*item_j = NUM_I + j
		}
		
		self.basis_history = vec![basis];
		
		let mut can = true;
		
		loop {
			let mut min = 0.0;
			let mut min_i = None;
			
			for (i, tab) in simple_tab.iter().enumerate() {
				if tab[NUM_I + NUM_J] < min {
					min = tab[NUM_I + NUM_J];
					min_i = Some(i);
				}
			}
			
			match min_i {
				None => break,
				Some(i) => {
					let mut min = 0.0;
					let mut min_j = None;
					for j in 0..NUM_I + NUM_J {
						if simple_tab[i][j] < min {
							min = simple_tab[i][j];
							min_j = Some(j);
						}
					}
					
					match min_j {
						None => {
							can = false;
							break;
						}
						Some(j) => self.change_basis(&mut simple_tab, &mut basis, i, j)
					}
				}
			}
		}
		
		self.result = None;
		
		if can {
			self.delta_history = Vec::new();
			
			for _ in 0..20 {
				let mut delta = [0.0; NUM_I + NUM_J + 1];
				
				for (i, item_i) in simple_tab.iter().enumerate() {
					for (j, item_j) in delta.iter_mut().enumerate() {
						if (0..NUM_I).contains(&basis[i]) {
							*item_j += item_i[j] * self.final_equation.cof[basis[i]];
						}
					}
				}
				
				for (i, item) in delta.iter_mut().enumerate().take(NUM_I) {
					*item -= self.final_equation.cof[i];
				}
				
				self.delta_history.push(delta);
				
				if self.is_optimal(&delta) {
					let mut x = [0.0; NUM_I + NUM_J];
					for (i, item) in simple_tab.iter().enumerate() {
						x[basis[i]] = item[NUM_I + NUM_J];
					}
					
					self.result = Some(x);
					break;
				}
				
				let mut max_min = 0.0;
				let mut max_min_j = 0;
				for (j, item) in delta.iter().enumerate().take(NUM_I + NUM_J) {
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
					if item[max_min_j] > 0.0 && item[NUM_I + NUM_J] / item[max_min_j] < min {
						min = item[NUM_I + NUM_J] / item[max_min_j];
						min_i = Some(i);
					}
				}
				
				match min_i {
					None => break,
					Some(i) => self.change_basis(&mut simple_tab, &mut basis, i, max_min_j)
				}
			}
		}
	}
}

impl eframe::App for MyApp {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		let mut changed = false;
		
		CentralPanel::default().show(ctx, |ui| {
			ui.horizontal(|ui| {
				add_drag_value(ui, &mut self.final_equation.cof[0], "z = ", &mut changed);
				add_drag_value(ui, &mut self.final_equation.cof[1], "x1 + ", &mut changed);
				add_drag_value(ui, &mut self.final_equation.cof[2], "x2 + ", &mut changed);
				add_drag_value(ui, &mut self.final_equation.cof[3], "x3 + ", &mut changed);
				ui.label("x4 -> ");
				ComboBox::from_id_salt(3)
					.selected_text(self.final_equation.limit.to_string())
					.width(20.0)
					.show_ui(ui, |ui| {
						if ui.selectable_value(&mut self.final_equation.limit, Limit::Max, "Max").changed() {
							changed = true
						};
						if ui.selectable_value(&mut self.final_equation.limit, Limit::Min, "Min").changed() {
							changed = true
						};
					});
			});
			for i in 0..NUM_J {
				ui.horizontal(|ui| {
					add_drag_value(ui, &mut self.equations[i].cof[0], "", &mut changed);
					add_drag_value(ui, &mut self.equations[i].cof[1], "x1 + ", &mut changed);
					add_drag_value(ui, &mut self.equations[i].cof[2], "x2 + ", &mut changed);
					add_drag_value(ui, &mut self.equations[i].cof[3], "x3 + ", &mut changed);
					ui.label("x4 ");
					ComboBox::from_id_salt(i)
						.selected_text(self.equations[i].cmp.to_string())
						.width(15.0)
						.show_ui(ui, |ui| {
						if ui.selectable_value(&mut self.equations[i].cmp, Cmp::Gte, ">=").changed() {
							changed = true
						};
						if ui.selectable_value(&mut self.equations[i].cmp, Cmp::Lte, "<=").changed() {
							changed = true
						};
						});
					add_drag_value(ui, &mut self.equations[i].cof[4], "", &mut changed);
				});
			}
			
			if changed {
				self.update_simple_tab()
			}
			
			ScrollArea::vertical()
				.max_height(160.0)
				.show(ui, |ui| {
					for (i, simple_tab) in self.simple_tab_history.iter().enumerate() {
						TableBuilder::new(ui)
							.id_salt((i * 2) + 1001)
							.vscroll(false)
							.striped(true)
							.columns(Column::auto().at_least(50.0), NUM_I + NUM_J + 2)
							.header(20.0, |mut header| {
								header.col(|ui| {
									ui.heading("");
								});
								for i in 0..NUM_I + NUM_J {
									header.col(|ui| {
										ui.heading("x".to_owned() + (i + 1).to_string().as_ref());
									});
								}
								header.col(|ui| {
									ui.heading("b");
								});
							})
							.body(|mut body| {
								for (j, items_row) in simple_tab.iter().enumerate() {
									body.row(20.0, |mut row| {
										row.col(|ui| {
											ui.label("x".to_owned() + (self.basis_history[i][j] + 1).to_string().as_ref());
										});
										for item in items_row {
											row.col(|ui| {
												ui.label(format!("{:.2}", item));
											});
										}
									})
								}
							});
						if i >= self.simple_tab_history.len() - self.delta_history.len() {
							ui.add_space(5.0);
							
							TableBuilder::new(ui)
								.id_salt(i * 2 + 1000)
								.vscroll(false)
								.striped(true)
								.columns(Column::auto().at_least(50.0), NUM_I + NUM_J + 2)
								.header(20.0, |mut header| {
									header.col(|ui| {
										ui.heading("");
									});
									for i in 0..NUM_I + NUM_J {
										header.col(|ui| {
											ui.heading("d".to_owned() + (i + 1).to_string().as_ref());
										});
									}
									header.col(|ui| {
										ui.heading("db");
									});
								})
								.body(|mut body| {
									body.row(20.0, |mut row| {
										row.col(|_ui| {});
										for item in self.delta_history[i - (self.simple_tab_history.len() - self.delta_history.len())] {
											row.col(|ui| {
												ui.label(format!("{:.2}", item));
											});
										}
									})
								});
						}
						ui.add_space(10.0);
					}
				});
			
			if let Some(result) = self.result {
				let mut sum = 0.0;
				ui.horizontal(|ui| {
					for (i, item_i) in result.iter().take(NUM_I).enumerate() {
						ui.label(format!("{:.2}", *item_i));
						ui.add_space(10.0);
						sum += *item_i * self.final_equation.cof[i];
					}
				});
				ui.add_space(10.0);
				ui.label(format!("{:.2}", sum));
				ui.add_space(10.0);
			}
			
			if NUM_I == 2 {
				Plot::new("plot")
					.auto_bounds(false)
					.x_axis_label("x1")
					.y_axis_label("x2")
					.show(ui, |plot| {
						for i in 0..NUM_J {
							let x1 = 1000.0;
							let x2 = (self.equations[i].cof[2] - self.equations[i].cof[0] * x1) / self.equations[i].cof[1];
							let x1_2 = -1000.0;
							let x2_2 = (self.equations[i].cof[2] - self.equations[i].cof[0] * x1_2) / self.equations[i].cof[1];
							plot.add(Line::new(Vec::from([[x1, x2], [x1_2, x2_2]]))
								.color(Color32::BLUE)
								.width(2.0)
							);
						}
						
						if let Some(result) = self.result {
							plot.add(Points::new([result[0], result[1]]).color(Color32::RED).radius(5.0));
							
							let x2_1 = (result[0] - 1000.0) * -self.final_equation.cof[0] / self.final_equation.cof[1];
							let x2_2 = (result[0] + 1000.0) * -self.final_equation.cof[0] / self.final_equation.cof[1];
							plot.add(Line::new(Vec::from([
								[2.0 * result[0] - 1000.0, x2_1 + result[1]],
								[2.0 * result[0] + 1000.0, x2_2 + result[1]]
							]))
								.color(Color32::GREEN)
								.width(2.0));
						}
						
						plot.add(Polygon::new(self.dots.clone()).fill_color(Color32::from_rgba_premultiplied(80, 80, 0, 100)))
					});
			};
		});
	}
}