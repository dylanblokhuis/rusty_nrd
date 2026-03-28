fn shader_bytes(desc: &rusty_nrd::ComputeShaderDesc) -> &[u8] {
    if desc.bytecode.is_null() || desc.size == 0 {
        return &[];
    }
    unsafe { std::slice::from_raw_parts(desc.bytecode as *const u8, desc.size as usize) }
}

fn main() {
    let _lib = rusty_nrd::LibraryInfo::query().expect(
        "linked libNRD major.minor must match this crate's headers; update Include/libNRD or regenerate ffi",
    );

    let mut instance = rusty_nrd::Instance::try_new_denoisers(&[rusty_nrd::DenoiserSlot {
        identifier: rusty_nrd::Identifier(0),
        denoiser: rusty_nrd::Denoiser::ReblurDiffuse,
    }])
    .expect("Create NRD instance");

    let inst_desc = instance.description().expect("instance description");
    for pipeline in inst_desc.pipelines() {
        let spirv = shader_bytes(&pipeline.computeShaderSPIRV);
        let metal = shader_bytes(&pipeline.computeShaderMetal);
        println!("SPIRV size: {} bytes", spirv.len());
        println!("Metal metallib size: {} bytes", metal.len());
        println!("{:#?}", pipeline);
    }
    println!("{:#?}", inst_desc.raw());

    let mut common_settings = rusty_nrd::default_common_settings();
    common_settings.resourceSize = [1920, 1080];
    common_settings.rectSize = [1920, 1080];

    let reblur_settings = rusty_nrd::default_reblur_settings();

    instance
        .set_common_settings(&common_settings)
        .expect("SetCommonSettings");
    instance
        .set_reblur_settings(rusty_nrd::Identifier(0), &reblur_settings)
        .expect("SetDenoiserSettings (REBLUR)");

    let dispatches = instance
        .compute_dispatches(&[rusty_nrd::Identifier(0)])
        .expect("GetComputeDispatches");
    println!("{:#?}", dispatches);

    if let Some(name) = rusty_nrd::denoiser_name(rusty_nrd::Denoiser::ReblurDiffuse) {
        println!("denoiser: {}", name.to_string_lossy());
    }

    let pipeline_count = instance.description().expect("instance description").pipelines().len();
    for pipeline_index in 0..pipeline_count {
        let pipeline_index = pipeline_index as u16;

        let bindings = instance
            .pipeline_descriptor_binding_descs(pipeline_index, false)
            .expect("GetPipelineDescriptorBindingDescs (native)");
        println!("pipeline {} bindings (native): {:#?}", pipeline_index, bindings);

        let spirv_bindings = instance
            .pipeline_descriptor_binding_descs(pipeline_index, true)
            .expect("GetPipelineDescriptorBindingDescs (spirv)");
        println!(
            "pipeline {} bindings (spirv): {:#?}",
            pipeline_index, spirv_bindings
        );
    }
}
