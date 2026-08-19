//! A batch is a tool to encode render commands.

pub mod builder;
pub mod command;
pub mod polyline;

pub use builder::{FillBuilder, StrokeBuilder};
pub use command::{MeshVertex, RenderCommand, StrokeVertex};
pub use polyline::Polyline;

mod circle;
mod mesh;
mod pipeline;

use crate::scene::EntityId;
use circle::CircleBatch;
use mesh::MeshBatch;
use std::mem;
use wgpu::{BindGroupLayout, Device, Queue, RenderPass, TextureFormat};

// ----- Submission -----

enum Submission {
    Circle(usize),
    Mesh(usize),
}

// ----- Batch -----

const MAX_CIRCLES: usize = 100_000;
const MAX_VERTICES: usize = 100_000;
const MAX_INDICES: usize = 50_000;

/// A batch is a list of high-level render commands.
pub struct Batch {
    circles: CircleBatch,
    meshes: MeshBatch,

    /// Encodes commands and their z_indices.
    commands: Vec<(EntityId, RenderCommand, f32)>,

    /// Tracks the submissions that we need to make. We'll try to batch these together in the
    /// future.
    submissions: Vec<Submission>,
}

impl Batch {
    pub fn new(device: &Device, format: TextureFormat, camera_layout: &BindGroupLayout) -> Self {
        Self {
            circles: CircleBatch::new(device, format, MAX_CIRCLES, camera_layout),
            meshes: MeshBatch::new(device, format, MAX_VERTICES, MAX_INDICES, camera_layout),
            commands: Vec::new(),
            submissions: Vec::new(),
        }
    }

    /// Clears this batch and creates an encoder to record render commands.
    pub fn encoder(&mut self) -> Encoder<'_> {
        // First clear the commands we have
        self.commands.clear();

        Encoder {
            commands: &mut self.commands,
        }
    }

    /// Copies the data from encoded commands to GPU.
    pub fn prepare(&mut self, queue: &Queue) {
        // Clear the batches.
        self.circles.clear();
        self.meshes.clear();

        // First sort by entity-id, then sort by z-index.
        let mut commands = mem::take(&mut self.commands);
        commands.sort_by_key(|id| id.0);
        commands.sort_by(|m, n| m.2.total_cmp(&n.2));

        // Map non-primitive commands into primitives.
        let commands = commands
            .into_iter()
            .flat_map(|(_, command, _)| match command {
                // Stroke commands get mapped into mesh commands.
                RenderCommand::Stroke { vertices, close } => {
                    let mut iter = vertices.into_iter();
                    iter.next()
                        .map(|start| {
                            // Convert the stroke into a filled mesh.
                            let mut builder = StrokeBuilder::new(start);
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
                            let mut builder = FillBuilder::default();
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
        self.circles.prepare(queue);
        self.meshes.prepare(queue);
    }

    /// Submits the calls to the GPU.
    pub fn submit(&mut self, pass: &mut RenderPass) {
        for submission in self.submissions.iter() {
            match *submission {
                Submission::Circle(idx) => self.circles.submit(pass, idx),
                Submission::Mesh(idx) => self.meshes.submit(pass, idx),
            }
        }
    }
}

// ----- Encoder -----

/// Builds a list of `RenderCommands` for a `Batch`.
pub struct Encoder<'a> {
    commands: &'a mut Vec<(EntityId, RenderCommand, f32)>,
}

impl Encoder<'_> {
    /// Encode a command for an entity with specified z-index.
    pub fn add_command(&mut self, id: EntityId, command: RenderCommand, z_index: f32) {
        self.commands.push((id, command, z_index));
    }
}
