use crate::{
    keys::{VMKeys, VMKeysPrepared},
    parameters::CryptographicParameters,
    Instruction, InstructionsParser, Interpreter, RV32I,
};
use plotters::prelude::*;
use poulpy_backend::FFT64Ref;
use poulpy_core::{
    layouts::{
        GGLWEToGGSWKeyPreparedFactory, GGSWPreparedFactory, GLWEAutomorphismKeyPreparedFactory,
        GLWEInfos, GLWESecret, GLWESecretPrepared, GLWESecretPreparedFactory, LWESecret,
    },
    GGLWEToGGSWKeyEncryptSk, GLWEAutomorphismKeyEncryptSk, GLWEDecrypt, GLWEEncryptSk,
    GLWEExternalProduct, GLWEPackerOps, GLWEPacking, GLWETrace, ScratchTakeCore,
};
use poulpy_hal::{
    api::{ModuleN, ModuleNew, ScratchOwnedAlloc, ScratchOwnedBorrow},
    layouts::{Backend, Module, Scratch, ScratchOwned},
    source::Source,
};
use poulpy_schemes::tfhe::{
    bdd_arithmetic::{
        BDDKeyEncryptSk, BDDKeyPreparedFactory, FheUintPrepare, FheUintPreparedEncryptSk,
        FheUintPreparedFactory, GGSWBlindRotation,
    },
    blind_rotation::{BlindRotationAlgo, BlindRotationKey, BlindRotationKeyFactory, CGGI},
};

#[test]
fn test_interpreter_cycles_fft64_ref() {
    test_interpreter_cycles::<CGGI, FFT64Ref>()
}

fn test_interpreter_cycles<BRA: BlindRotationAlgo, BE: Backend>()
where
    Module<BE>: ModuleNew<BE>
        + GLWESecretPreparedFactory<BE>
        + FheUintPreparedFactory<u32, BE>
        + ModuleN
        + GLWEEncryptSk<BE>
        + FheUintPreparedEncryptSk<u32, BE>
        + GLWEAutomorphismKeyEncryptSk<BE>
        + GGLWEToGGSWKeyEncryptSk<BE>
        + GLWETrace<BE>
        + BDDKeyEncryptSk<BRA, BE>
        + GGSWPreparedFactory<BE>
        + GLWEExternalProduct<BE>
        + GLWEPackerOps<BE>
        + GLWEPacking<BE>
        + FheUintPrepare<BRA, BE>
        + GGSWBlindRotation<u32, BE>
        + GGSWPreparedFactory<BE>
        + GLWEDecrypt<BE>
        + GLWEAutomorphismKeyPreparedFactory<BE>
        + GGLWEToGGSWKeyPreparedFactory<BE>
        + BDDKeyPreparedFactory<BRA, BE>,
    ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
    Scratch<BE>: ScratchTakeCore<BE>,
    BlindRotationKey<Vec<u8>, BRA>: BlindRotationKeyFactory<BRA>,
{
    let rom: Vec<Instruction> = vec![
        // RD[31] <- 1<<18
        RV32I::LUI.new().set_imm(1 << 6).set_rd(31),
        // RD[1] <- 0xABCD<<12
        RV32I::LUI.new().set_imm(0xABCD).set_rd(1),
        // RD[2] <- 0xEF10<<12
        RV32I::LUI.new().set_imm(0xEF10).set_rd(2),
        // RAM[RD[31] - 1<<18] <- RD[1] + 1<<12
        RV32I::ADDI.new().set_imm(0x1).set_rs1(1).set_rd(3),
        RV32I::SW.new().set_imm(0).set_rs1(31).set_rs2(3),
        RV32I::ADDI.new().set_imm(4).set_rs1(31).set_rd(31),
        // RAM[RD[31] - 1<<18] <- RD[1] < 0xEF10<<12
        RV32I::SLTI.new().set_imm(0xEF10).set_rs1(1).set_rd(3),
        RV32I::SW.new().set_imm(0).set_rs1(31).set_rs2(3),
        RV32I::ADDI.new().set_imm(4).set_rs1(31).set_rd(31),
    ];

    let ram: Vec<u32> = vec![0u32; 64];

    let params: CryptographicParameters<BE> = CryptographicParameters::<BE>::new();
    let module: &Module<BE> = params.module();

    let mut source_xs: Source = Source::new([0u8; 32]);
    let mut source_xa: Source = Source::new([0u8; 32]);
    let mut source_xe: Source = Source::new([0u8; 32]);

    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 24);

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc(params.n_glwe(), params.rank());
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);
    let mut sk_lwe: LWESecret<Vec<u8>> = LWESecret::alloc(params.n_lwe());
    sk_lwe.fill_binary_block(params.lwe_block_size(), &mut source_xs);

    let mut interpreter: Interpreter<BE> =
        Interpreter::new_with_debug(&params, rom.len(), ram.len());

    let mut instructions = InstructionsParser::new();
    for inst in &rom {
        instructions.add(*inst);
    }

    let mut sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, BE> =
        GLWESecretPrepared::alloc(module, sk_glwe.rank());
    sk_glwe_prepared.prepare(module, &sk_glwe);

    interpreter.instructions_encrypt_sk(
        module,
        &instructions,
        &sk_glwe_prepared,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    interpreter.ram_encrypt_sk(
        module,
        &ram,
        &sk_glwe_prepared,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 28);

    let key: VMKeys<Vec<u8>, BRA> =
        VMKeys::encrypt_sk(&params, &sk_lwe, &sk_glwe, &mut source_xa, &mut source_xe);

    let mut key_prepared: VMKeysPrepared<Vec<u8>, BRA, BE> = VMKeysPrepared::alloc(&params);
    key_prepared.prepare(module, &key, scratch.borrow());

    for _ in 0..rom.len() {
        interpreter.cycle_debug(
            16,
            module,
            &key_prepared,
            &sk_glwe_prepared,
            scratch.borrow(),
        );
    }
}

#[test]
fn test_interpreter_cycles_noise_progression_fft64_ref() {
    test_interpreter_cycles_noise_progression::<CGGI, FFT64Ref>()
}

fn test_interpreter_cycles_noise_progression<BRA: BlindRotationAlgo, BE: Backend>()
where
    Module<BE>: ModuleNew<BE>
        + GLWESecretPreparedFactory<BE>
        + FheUintPreparedFactory<u32, BE>
        + ModuleN
        + GLWEEncryptSk<BE>
        + FheUintPreparedEncryptSk<u32, BE>
        + GLWEAutomorphismKeyEncryptSk<BE>
        + GGLWEToGGSWKeyEncryptSk<BE>
        + GLWETrace<BE>
        + BDDKeyEncryptSk<BRA, BE>
        + GGSWPreparedFactory<BE>
        + GLWEExternalProduct<BE>
        + GLWEPackerOps<BE>
        + GLWEPacking<BE>
        + FheUintPrepare<BRA, BE>
        + GGSWBlindRotation<u32, BE>
        + GGSWPreparedFactory<BE>
        + GLWEDecrypt<BE>
        + GLWEAutomorphismKeyPreparedFactory<BE>
        + GGLWEToGGSWKeyPreparedFactory<BE>
        + BDDKeyPreparedFactory<BRA, BE>,
    ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
    Scratch<BE>: ScratchTakeCore<BE>,
    BlindRotationKey<Vec<u8>, BRA>: BlindRotationKeyFactory<BRA>,
{

    std::env::set_var("VERBOSE_TIMINGS", "0");

    let num_instructions = 100;
    let instruction = Instruction::new(0b00000000_00000000_00000000_1110011);
    let rom: Vec<Instruction> = vec![instruction; num_instructions];

    let ram: Vec<u32> = vec![0u32; 64];

    let params: CryptographicParameters<BE> = CryptographicParameters::<BE>::new();
    let module: &Module<BE> = params.module();

    let mut source_xs: Source = Source::new([0u8; 32]);
    let mut source_xa: Source = Source::new([0u8; 32]);
    let mut source_xe: Source = Source::new([0u8; 32]);

    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 24);

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc(params.n_glwe(), params.rank());
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);
    let mut sk_lwe: LWESecret<Vec<u8>> = LWESecret::alloc(params.n_lwe());
    sk_lwe.fill_binary_block(params.lwe_block_size(), &mut source_xs);

    let mut interpreter: Interpreter<BE> =
        Interpreter::new_with_debug(&params, rom.len(), ram.len());

    let mut instructions = InstructionsParser::new();
    for inst in &rom {
        instructions.add(*inst);
    }

    let mut sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, BE> =
        GLWESecretPrepared::alloc(module, sk_glwe.rank());
    sk_glwe_prepared.prepare(module, &sk_glwe);

    interpreter.instructions_encrypt_sk(
        module,
        &instructions,
        &sk_glwe_prepared,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    interpreter.ram_encrypt_sk(
        module,
        &ram,
        &sk_glwe_prepared,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 28);

    let key: VMKeys<Vec<u8>, BRA> =
        VMKeys::encrypt_sk(&params, &sk_lwe, &sk_glwe, &mut source_xa, &mut source_xe);

    let mut key_prepared: VMKeysPrepared<Vec<u8>, BRA, BE> = VMKeysPrepared::alloc(&params);
    key_prepared.prepare(module, &key, scratch.borrow());

    for _ in 0..rom.len() {
        interpreter.cycle_debug(
            16,
            module,
            &key_prepared,
            &sk_glwe_prepared,
            scratch.borrow(),
        );
        let series = vec![
            (
                "PC value noise".to_string(),
                interpreter.measurements.get_pc_val_fhe_uint_noise_list(),
            ),
            (
                "IMM value noise".to_string(),
                interpreter.measurements.get_imm_val_fhe_uint_noise_list(),
            ),
            (
                "RAM address read noise".to_string(),
                interpreter.measurements.get_ram_addr_read_noise_list(),
            ),
            (
                "RAM value read noise".to_string(),
                interpreter.measurements.get_ram_val_read_noise_list(),
            ),
            (
                "RD value noise".to_string(),
                interpreter.measurements.get_rd_val_fhe_uint_noise_list(),
            ),
        ];

        if series.iter().any(|(_, data)| !data.is_empty()) {
            let plot_dir = std::path::PathBuf::from("artifacts");
            if let Err(err) = std::fs::create_dir_all(&plot_dir) {
                println!(
                    "Failed to create plot directory {}: {err}",
                    plot_dir.display()
                );
            }
            let plot_path = plot_dir.join("noise_progression.svg");
            match plot_noise_progression(&params, &plot_path, &series) {
                Ok(()) => println!("RAM noise plot written to {}", plot_path.display()),
                Err(err) => println!("Failed to render RAM noise plot: {err}"),
            }
        }
    }
}

fn plot_noise_progression<P: AsRef<std::path::Path>, BE: Backend>(
    params: &CryptographicParameters<BE>,
    output_path: P,
    series: &[(String, Vec<f64>)],
) -> Result<(), Box<dyn std::error::Error>> {
    if series.is_empty() {
        return Ok(());
    }

    let max_len = series.iter().map(|(_, data)| data.len()).max().unwrap_or(0);
    if max_len == 0 {
        return Ok(());
    }
    let x_max = (max_len - 1) as f64;

    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;

    for (_, data) in series {
        for value in data {
            if value.is_finite() {
                y_min = y_min.min(*value);
                y_max = y_max.max(*value);
            }
        }
    }

    if !y_min.is_finite() || !y_max.is_finite() {
        y_min = -1.0;
        y_max = 1.0;
    } else if (y_max - y_min).abs() < f64::EPSILON {
        y_min -= 1.0;
        y_max += 1.0;
    }

    let drawing_area = SVGBackend::new(output_path.as_ref(), (1280, 720)).into_drawing_area();
    drawing_area.fill(&WHITE)?;

    let caption = format!("Noise Progression (N_GLWE: {}, N_LWE: {}, RANK: {})", params.n_glwe(), params.n_lwe(), params.rank());
    let mut chart = ChartBuilder::on(&drawing_area)
        .caption(caption, ("sans-serif", 30))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..x_max.max(1.0), y_min..y_max)?;

    chart
        .configure_mesh()
        .x_desc("Number of Cycles")
        .y_desc("|Log2(Noise)|")
        .draw()?;

    for (idx, (label, data)) in series.iter().enumerate() {
        let color = Palette99::pick(idx);
        let line_style = ShapeStyle::from(&color).stroke_width(3);
        let legend_style = line_style.clone();
        chart
            .draw_series(LineSeries::new(
                data.iter()
                    .enumerate()
                    .map(|(sample_idx, value)| (sample_idx as f64, *value)),
                line_style.clone(),
            ))?
            .label(label.clone())
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], legend_style.clone())
            });
    }

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    drawing_area.present()?;
    Ok(())
}
