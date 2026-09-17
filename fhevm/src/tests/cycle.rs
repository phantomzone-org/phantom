use poulpy_bin_fhe::blind_rotation::{BlindRotationAlgo, CGGI};
use poulpy_cpu_ref::FFT64Ref;
use poulpy_hal::layouts::{Module, ScratchOwned};

use crate::{
    backend::{prelude::*, PhantomBackend, PhantomBootstrapModule},
    parameters::CryptographicParameters,
    tests::TestContext,
    Instruction, InstructionsParser, Interpreter, RV32I,
};

#[test]
fn test_interpreter_cycles_fft64_ref() {
    test_interpreter_cycles::<CGGI, FFT64Ref>()
}

fn test_interpreter_cycles<BRA: BlindRotationAlgo, BE: PhantomBackend>()
where
    Module<BE>: ModuleNew<BE> + PhantomBootstrapModule<BRA, BE>,
    ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
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

    let mut ctx: TestContext<BRA, BE> = TestContext::new();
    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 24);

    let mut interpreter: Interpreter<BE> =
        Interpreter::new_with_debug(&ctx.params, rom.len(), ram.len());

    let mut instructions = InstructionsParser::new();
    for inst in &rom {
        instructions.add(*inst);
    }

    interpreter.instructions_encrypt_sk(
        ctx.params.module(),
        &instructions,
        &ctx.sk_glwe_prepared,
        &mut ctx.source_xa,
        &mut ctx.source_xe,
        &mut scratch.borrow(),
    );

    interpreter.ram_encrypt_sk(
        ctx.params.module(),
        &ram,
        &ctx.sk_glwe_prepared,
        &mut ctx.source_xa,
        &mut ctx.source_xe,
        &mut scratch.borrow(),
    );

    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 28);

    interpreter.set_verbose_timings(true);

    for _ in 0..rom.len() {
        interpreter.cycle_debug(
            ctx.params.module(),
            &ctx.keys_prepared,
            &ctx.sk_glwe_prepared,
            &mut scratch.borrow(),
        );
    }
}

#[test]
fn test_interpreter_cycles_noise_progression_fft64_ref() {
    test_interpreter_cycles_noise_progression::<CGGI, FFT64Ref>()
}

fn test_interpreter_cycles_noise_progression<BRA: BlindRotationAlgo, BE: PhantomBackend>()
where
    Module<BE>: ModuleNew<BE> + PhantomBootstrapModule<BRA, BE>,
    ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
{
    let num_instructions = 100;
    let instruction = Instruction::new(0b00000000_00000000_00000000_1110011);
    let rom: Vec<Instruction> = vec![instruction; num_instructions];

    let ram: Vec<u32> = vec![0u32; 64];

    let mut ctx: TestContext<BRA, BE> = TestContext::new();
    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 24);

    let mut interpreter: Interpreter<BE> =
        Interpreter::new_with_debug(&ctx.params, rom.len(), ram.len());

    let mut instructions = InstructionsParser::new();
    for inst in &rom {
        instructions.add(*inst);
    }

    interpreter.instructions_encrypt_sk(
        ctx.params.module(),
        &instructions,
        &ctx.sk_glwe_prepared,
        &mut ctx.source_xa,
        &mut ctx.source_xe,
        &mut scratch.borrow(),
    );

    interpreter.ram_encrypt_sk(
        ctx.params.module(),
        &ram,
        &ctx.sk_glwe_prepared,
        &mut ctx.source_xa,
        &mut ctx.source_xe,
        &mut scratch.borrow(),
    );

    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 28);

    interpreter.set_verbose_timings(false);
    interpreter.set_threads(16);

    for _ in 0..rom.len() {
        interpreter.cycle_debug(
            ctx.params.module(),
            &ctx.keys_prepared,
            &ctx.sk_glwe_prepared,
            &mut scratch.borrow(),
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
            let plot_dir = std::path::PathBuf::from("../doc");
            if let Err(err) = std::fs::create_dir_all(&plot_dir) {
                println!(
                    "Failed to create plot directory {}: {err}",
                    plot_dir.display()
                );
            }
            let plot_path = plot_dir.join(format!(
                "noise_progression_n-glwe={}_n-lwe={}_rank={}_base2k={}.png",
                ctx.params.n_glwe(),
                ctx.params.n_lwe(),
                ctx.params.rank(),
                ctx.params.base2k_fhe_uint()
            ));
            match plot_noise_progression(&ctx.params, &plot_path, &series) {
                Ok(()) => println!("Noise progression plot written to {}", plot_path.display()),
                Err(err) => println!("Failed to render noise progression plot: {err}"),
            }
        }
    }
}

fn plot_noise_progression<P: AsRef<std::path::Path>, BE: PhantomBackend>(
    params: &CryptographicParameters<BE>,
    output_path: P,
    series: &[(String, Vec<f64>)],
) -> Result<(), Box<dyn std::error::Error>> {
    use plotters::prelude::*; // Ensure PNGBackend is in scope

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

    // Use PNGBackend instead of SVGBackend
    let drawing_area = BitMapBackend::new(output_path.as_ref(), (1280, 720)).into_drawing_area();
    drawing_area.fill(&WHITE)?;

    let caption = format!(
        "Noise Progression (N_GLWE: {}, N_LWE: {}, RANK: {})",
        params.n_glwe(),
        params.n_lwe(),
        params.rank()
    );
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
