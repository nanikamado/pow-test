mod sha256;

use pollster::block_on;
use sha256::{sha256_push, Sha256Ctx};
use wgpu::util::DeviceExt;
use wgpu::PowerPreference;

async fn sha256(prefix: &str, suffix: &str) -> Result<(String, u32), wgpu::Error> {
    let suffix = format!(":{suffix}");
    // Load the shader code
    let shader_code = include_str!("./sha256.wgsl");

    // Request the GPU adapter and device
    let instance = wgpu::Instance::default();
    let mut wgpu_options = wgpu::RequestAdapterOptions::default();
    wgpu_options.power_preference = PowerPreference::HighPerformance;
    let adapter = instance.request_adapter(&wgpu_options).await.unwrap();
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .unwrap();

    // Create the bind group layout
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
        label: None,
    });

    // Compile the shader and create the pipeline
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(shader_code.into()),
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        layout: Some(
            &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
                label: None,
            }),
        ),
        module: &shader_module,
        entry_point: Some("main"),
        label: None,
        compilation_options: Default::default(),
        cache: None,
    });

    // Buffer size calculations and creation
    eprintln!("device.limits() = {:#?}", device.limits());
    let group_x = device.limits().max_compute_workgroups_per_dimension;
    let group_y = 1;
    let result_buffer_size =
        std::mem::size_of::<u32>() as u64 * 256 * group_x as u64 * group_y as u64;
    let result_matrix_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: result_buffer_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let mut max_diff = 0;
    let mut max_result = String::new();

    for i in 0..10 {
        let suffix_buff = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(
                &suffix
                    .as_bytes()
                    .iter()
                    .map(|a| *a as u32)
                    .collect::<Vec<_>>(),
            ),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let mut initial_ctx = Sha256Ctx::default();
        for n in format!("{prefix}{i}-").as_bytes().iter() {
            sha256_push(&mut initial_ctx, *n as u32);
        }

        let initial_ctx_buff = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&initial_ctx.to_array()),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let size = [suffix.len() as u32];
        let suffix_size_buff = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&size),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: initial_ctx_buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: suffix_buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: suffix_size_buff.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: result_matrix_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });

        // Command buffer encoding
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&compute_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(group_x, group_y, 1);
        }

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: result_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
            label: None,
        });

        encoder.copy_buffer_to_buffer(
            &result_matrix_buffer,
            0,
            &staging_buffer,
            0,
            result_buffer_size,
        );
        queue.submit(Some(encoder.finish()));

        // Await results
        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = flume::bounded(1);
        buffer_slice.map_async(wgpu::MapMode::Read, move |a| sender.send(a).unwrap());
        device.poll(wgpu::Maintain::Wait).panic_on_timeout();
        if let Ok(Ok(())) = receiver.recv_async().await {
            let data = buffer_slice.get_mapped_range();
            let result_data: Vec<u32> = bytemuck::cast_slice(&data).to_vec();
            drop(data);
            staging_buffer.unmap();

            let index_of_max_value = result_data
                .iter()
                .enumerate()
                .max_by_key(|&(_, &x)| x)
                .map(|(index, _)| index)
                .unwrap();

            if result_data[index_of_max_value] > max_diff {
                max_diff = result_data[index_of_max_value];
                max_result = format!(
                    "{}-{}",
                    i,
                    format!("{:0>10}", index_of_max_value)
                        .chars()
                        .rev()
                        .collect::<String>()
                );
                println!("Current: {}, Diff: {}", max_result, max_diff);
            }
        } else {
            panic!("failed to run compute on gpu!")
        }
    }

    Ok((max_result, max_diff as u32))
}

// [0,"79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",1736674829,1,["nonce","","43"],"pow"]

fn main() {
    let input_string = r#"[0,"79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",1736674829,1,[["nonce",""#;
    let (result, diff) = block_on(sha256(input_string, r#"","43"]],"pow"]"#)).unwrap();
    println!("Result: {}, Diff: {}", result, diff);
}
