//! A batch is a tool to encode render commands.

pub mod builder;
pub mod circle;
pub mod command;
pub mod mesh;
pub mod polyline;

pub use builder::{FillBuilder, Segment, StrokeBuilder, StrokePoint};
pub use circle::CircleBatch;
pub use command::RenderCommand;
pub use mesh::{MeshBatch, MeshVertex};
pub use polyline::Polyline;

use crate::{render::Gpu, scene::EntityId};
use std::mem;
use wgpu::{BindGroupLayout, RenderPass};

// ----- Batch -----

const MAX_CIRCLES: usize = 100_000;
const MAX_VERTICES: usize = 100_000;
const MAX_INDICES: usize = 50_000;
const BUILDER_TOLERANCE: f32 = 0.001;

/// A batch is a reusable geometry renderer.
pub struct Batch {
    circles: CircleBatch,
    meshes: MeshBatch,

    /// Encodes commands and their z_indices.
    commands: Vec<(EntityId, f32, RenderCommand)>,

    /// Encodes the batches.
    submissions: Vec<Submission>,
}

impl Batch {
    pub fn new(gpu: &Gpu, camera_layout: &BindGroupLayout) -> Self {
        Self {
            circles: CircleBatch::new(gpu, MAX_CIRCLES, camera_layout),
            meshes: MeshBatch::new(gpu, MAX_VERTICES, MAX_INDICES, camera_layout),
            commands: Vec::new(),
            submissions: Vec::new(),
        }
    }

    /// Encode a command for an entty.with an optional z_index. No z_index means that an object
    /// strictly follows painter's algorithm.
    pub fn add_command(&mut self, command: RenderCommand, id: EntityId, z_index: f32) {
        self.commands.push((id, z_index, command));
    }

    /// Begins a new batch by clearing the current batch. Require GPU handle to restrict
    /// this to top-level renderer, not individual batch submissions.
    pub fn begin(&mut self, _gpu: &Gpu) {
        self.commands.clear();
    }

    pub fn submit(&mut self, gpu: &Gpu, pass: &mut RenderPass) {
        // Clear the batches.
        self.circles.clear();
        self.meshes.clear();

        // First sort by entity id (high to low), then sort by z-index.
        let mut commands = mem::take(&mut self.commands);
        commands.sort_by(|m, n| m.0.cmp(&n.0));
        commands.sort_by(|m, n| m.1.total_cmp(&n.1));

        // Map non-primitive commands into primitives.
        let commands = commands
            .into_iter()
            .flat_map(|(_, _, command)| match command {
                // Stroke commands get mapped into mesh commands.
                RenderCommand::Stroke { vertices, close } => {
                    let mut iter = vertices.into_iter();
                    iter.next()
                        .map(|start| {
                            // Convert the stroke into a filled mesh.
                            let mut builder = StrokeBuilder::new(start, BUILDER_TOLERANCE);
                            iter.for_each(|v| builder.line_to(v));
                            let res = builder.finish(close);

                            // Return the mesh primitive command.
                            match res {
                                Ok(command) => Some(command),
                                Err(e) => {
                                    log::warn!("Failed to tesselate stroke: {e}");
                                    None
                                }
                            }
                        })
                        .flatten()
                }

                // Some goes for polygon commands.
                RenderCommand::Polygon { vertices } => {
                    let mut iter = vertices.into_iter();
                    iter.next()
                        .map(|start| {
                            // Encode the points into a builder.
                            let mut builder = FillBuilder::new(BUILDER_TOLERANCE);
                            builder.begin_subpath(start);
                            iter.for_each(|v| builder.line_to(v));
                            let res = builder.finish();

                            // Return the mesh primitive command.
                            match res {
                                Ok(command) => Some(command),
                                Err(e) => {
                                    log::warn!("Failed to tesselate polygon: {e}");
                                    None
                                }
                            }
                        })
                        .flatten()
                }
                command => Some(command),
            });

        // Clear and track our submissions.
        self.submissions.clear();

        // Encode the primitive commands. Currently we aren't batching at all.
        for command in commands {
            match command {
                RenderCommand::Circle { transform, color } => {
                    let index = self.circles.push(transform, color);
                    if let Some(index) = index {
                        self.submissions.push(Submission::Circle(index));
                    }
                }

                RenderCommand::Mesh { vertices, indices } => {
                    let index = self.meshes.push(&vertices, &indices);
                    if let Some(index) = index {
                        self.submissions.push(Submission::Mesh(index));
                    }
                }

                _ => (),
            }
        }

        // Upload data to GPU and execute submissions.
        self.circles.prepare(gpu);
        self.meshes.prepare(gpu);
        for submission in self.submissions.iter() {
            match *submission {
                Submission::Circle(idx) => self.circles.submit(gpu, pass, idx),
                Submission::Mesh(idx) => self.meshes.submit(gpu, pass, idx),
            }
        }
    }

    pub fn tolerance(&self) -> f32 {
        BUILDER_TOLERANCE
    }
}

enum Submission {
    Circle(usize),
    Mesh(usize),
}
