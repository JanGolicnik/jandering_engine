use crate::compute_pass::{ComputePassData, ComputePassTrait};

use super::WGPURenderer;

pub struct WGPUComputePass<'renderer> {
    renderer: &'renderer mut WGPURenderer,
    data: ComputePassData,
}

impl<'renderer> WGPUComputePass<'renderer> {
    pub fn new(renderer: &'renderer mut WGPURenderer) -> Self {
        Self {
            renderer,

            data: ComputePassData::default(),
        }
    }
}

impl<'renderer> ComputePassTrait for WGPUComputePass<'renderer> {
    fn get_data(&mut self) -> &mut ComputePassData {
        &mut self.data
    }

    fn dispatch(self, x: u32, y: u32, z: u32) {
        let mut encoder =
            self.renderer
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Compute Encoder"),
                });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });

            for (index, handle) in self.data.bind_groups.iter() {
                compute_pass.set_bind_group(
                    *index,
                    &self.renderer.bind_groups_render_data[handle.0].bind_group,
                    &[],
                );
            }
            let shader = &self.renderer.compute_shaders[self.data.shader.0];
            compute_pass.set_pipeline(&shader.pipeline);
            compute_pass.dispatch_workgroups(x, y, z);
        }

        self.renderer.queue.submit(Some(encoder.finish()));
    }
}
