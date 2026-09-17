use poulpy_bin_fhe::{
    bdd_arithmetic::FheUint,
    blind_rotation::{BlindRotationAlgo, CGGI},
};
use poulpy_cpu_ref::FFT64Ref;
use poulpy_hal::layouts::{Module, ScratchOwned};
use rand_core::Rng;

use crate::{
    backend::{prelude::*, PhantomBackend, PhantomBootstrapModule},
    rd_update::Evaluate,
    tests::TestContext,
    Instruction, InstructionsParser, Interpreter, RD_UPDATE_RV32I_OP_LIST,
};

#[test]
fn test_interpreter_init_one_instruction_fft64_ref() {
    test_interpreter_init_one_instruction::<CGGI, FFT64Ref>()
}

pub fn test_interpreter_init_one_instruction<BRA: BlindRotationAlgo, BE: PhantomBackend>()
where
    Module<BE>: ModuleNew<BE> + PhantomBootstrapModule<BRA, BE>,
    ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
{
    let mut ctx: TestContext<BRA, BE> = TestContext::new();
    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 22);

    let rom_size = 1 << 10;
    let ram_size = 1 << 10;
    let mut interpreter: Interpreter<BE> = Interpreter::new(&ctx.params, rom_size, ram_size);

    let instruction_u32 = 258455;
    let mut instructions: InstructionsParser = InstructionsParser::new();
    instructions.add(Instruction::new(instruction_u32));

    interpreter.instructions_encrypt_sk(
        ctx.params.module(),
        &instructions,
        &ctx.sk_glwe_prepared,
        &mut ctx.source_xa,
        &mut ctx.source_xe,
        &mut scratch.borrow(),
    );

    let idx = 0;

    let instruction: Instruction = instructions.get_raw(idx);
    let correct_imm: u32 = instruction.get_imm();
    let (rs2, rs1, rd) = instruction.get_registers();
    let correct_rs1: u32 = rs1 as u32;
    let correct_rs2: u32 = rs2 as u32;
    let correct_rd: u32 = rd as u32;

    let (rdu, mu, pcu) = instruction.get_opid();
    let correct_rdu: u32 = rdu as u32;
    let correct_mu: u32 = mu as u32;
    let correct_pcu: u32 = pcu as u32;

    interpreter.pc_fhe_uint = ctx.encrypt_fhe_uint(idx as u32, &mut scratch.borrow());

    let mut this_cycle_measurement = crate::Measurement::new();
    interpreter.read_and_prepare_instruction_components(
        1,
        ctx.params.module(),
        &ctx.keys_prepared,
        Some(&ctx.sk_glwe_prepared),
        &mut scratch.borrow(),
        &mut this_cycle_measurement,
    );

    let dec = |ct: &FheUint<Vec<u8>, u32, i64>, scratch: &mut ScratchOwned<BE>| -> u32 {
        ct.decrypt(
            ctx.params.module(),
            &ctx.sk_glwe_prepared,
            &mut scratch.borrow(),
        )
    };

    let dec_rs1: u32 = dec(&interpreter.rs1_addr_fhe_uint, &mut scratch);
    let dec_rs2: u32 = dec(&interpreter.rs2_addr_fhe_uint, &mut scratch);
    let dec_rd: u32 = dec(&interpreter.rd_addr_fhe_uint, &mut scratch);
    let dec_imm: u32 = dec(&interpreter.imm_val_fhe_uint, &mut scratch);
    let dec_rdu: u32 = dec(&interpreter.rdu_val_fhe_uint, &mut scratch);
    let dec_mu: u32 = dec(&interpreter.mu_val_fhe_uint, &mut scratch);
    let dec_pcu: u32 = dec(&interpreter.pcu_val_fhe_uint, &mut scratch);

    println!(
        "{} {} {} {} {} {} {}",
        correct_imm, correct_rs1, correct_rs2, correct_rd, correct_rdu, correct_mu, correct_pcu
    );
    println!(
        "{} {} {} {} {} {} {}",
        dec_imm, dec_rs1, dec_rs2, dec_rd, dec_rdu, dec_mu, dec_pcu
    );

    assert_eq!(correct_imm, dec_imm);
    assert_eq!(correct_rs1, dec_rs1);
    assert_eq!(correct_rs2, dec_rs2);
    assert_eq!(correct_rd, dec_rd);
    assert_eq!(correct_rdu, dec_rdu);
    assert_eq!(correct_mu, dec_mu);
    assert_eq!(correct_pcu, dec_pcu);
}

#[test]
fn test_interpreter_init_one_op_fft64_ref() {
    test_interpreter_init_one_op::<CGGI, FFT64Ref>()
}

pub fn test_interpreter_init_one_op<BRA: BlindRotationAlgo, BE: PhantomBackend>()
where
    Module<BE>: ModuleNew<BE> + PhantomBootstrapModule<BRA, BE>,
    ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
{
    let threads = 4;

    let mut ctx: TestContext<BRA, BE> = TestContext::new();
    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc((1 << 22) * threads);

    let rom_size = 1 << 10;
    let ram_size = 1 << 10;
    let mut interpreter: Interpreter<BE> = Interpreter::new(&ctx.params, rom_size, ram_size);

    // Deterministic operands: the first draws of a fresh source.
    let mut source: poulpy_hal::source::Source = poulpy_hal::source::Source::new([0u8; 32]);
    let imm_value: u32 = source.next_u32();
    let rs1_value: u32 = source.next_u32();
    let rs2_value: u32 = source.next_u32();
    let pc_value: u32 = source.next_u32();
    let ram_value: u32 = source.next_u32();

    let mut this_cycle_measurement = crate::Measurement::new();
    interpreter.read_and_prepare_instruction_components(
        1,
        ctx.params.module(),
        &ctx.keys_prepared,
        Some(&ctx.sk_glwe_prepared),
        &mut scratch.borrow(),
        &mut this_cycle_measurement,
    );

    interpreter.rs1_val_fhe_uint_prepared =
        ctx.encrypt_fhe_uint_prepared(rs1_value, &mut scratch.borrow());
    interpreter.rs2_val_fhe_uint_prepared =
        ctx.encrypt_fhe_uint_prepared(rs2_value, &mut scratch.borrow());
    interpreter.imm_val_fhe_uint_prepared =
        ctx.encrypt_fhe_uint_prepared(imm_value, &mut scratch.borrow());
    interpreter.pc_fhe_uint_prepared =
        ctx.encrypt_fhe_uint_prepared(pc_value, &mut scratch.borrow());
    interpreter.ram_val_fhe_uint = ctx.encrypt_fhe_uint(ram_value, &mut scratch.borrow());

    let mut res: FheUint<Vec<u8>, u32, i64> =
        FheUint::alloc_from_infos(ctx.params.module(), &ctx.params.fhe_uint_infos());

    for op in RD_UPDATE_RV32I_OP_LIST {
        op.eval_enc(
            threads,
            ctx.params.module(),
            &mut res,
            &interpreter.rs1_val_fhe_uint_prepared,
            &interpreter.rs2_val_fhe_uint_prepared,
            &interpreter.imm_val_fhe_uint_prepared,
            &interpreter.pc_fhe_uint_prepared,
            &interpreter.ram_val_fhe_uint,
            &ctx.keys_prepared,
            &mut scratch.borrow(),
        );

        let have: u32 = res.decrypt(
            ctx.params.module(),
            &ctx.sk_glwe_prepared,
            &mut scratch.borrow(),
        );
        let want: u32 = op.eval_plain(imm_value, rs1_value, rs2_value, pc_value, ram_value);

        assert_eq!(
            have, want,
            "{:#?}\n
            rs1: {rs1_value}\n
            rs2: {rs2_value}\n
            imm: {imm_value}\n
            pc:  {pc_value}\n
            ram: {ram_value}\n
            -> have: {} != want: {}",
            op, have, want
        );
    }
}

#[test]
fn test_interpreter_init_many_instructions_fft64_ref() {
    test_interpreter_init_many_instructions::<CGGI, FFT64Ref>()
}

fn test_interpreter_init_many_instructions<BRA: BlindRotationAlgo, BE: PhantomBackend>()
where
    Module<BE>: ModuleNew<BE> + PhantomBootstrapModule<BRA, BE>,
    ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
{
    let mut ctx: TestContext<BRA, BE> = TestContext::new();
    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 24);

    let rom_size = 1 << 10;
    let ram_size = 1 << 10;
    let mut interpreter: Interpreter<BE> = Interpreter::new(&ctx.params, rom_size, ram_size);

    let instructions_u32 = vec![
        258455, 33653139, 512279, 4286644499, 66579, 10507363, 3221229683, 8388847, 3221229683,
        791, 8585319, 259383,
    ];

    let mut instructions = InstructionsParser::new();
    for inst in instructions_u32.clone() {
        instructions.add(Instruction::new(inst));
    }

    interpreter.instructions_encrypt_sk(
        ctx.params.module(),
        &instructions,
        &ctx.sk_glwe_prepared,
        &mut ctx.source_xa,
        &mut ctx.source_xe,
        &mut scratch.borrow(),
    );

    for idx in 0..instructions_u32.len() {
        let instruction = instructions.get_raw(idx);
        let correct_imm = instruction.get_imm();
        let (rs2, rs1, rd) = instruction.get_registers();
        let correct_rs1: u32 = rs1 as u32;
        let correct_rs2: u32 = rs2 as u32;
        let correct_rd: u32 = rd as u32;
        let (rdu, mu, pcu) = instruction.get_opid();
        let correct_rdu: u32 = rdu as u32;
        let correct_mu: u32 = mu as u32;
        let correct_pcu: u32 = pcu as u32;

        interpreter.pc_fhe_uint_prepared =
            ctx.encrypt_fhe_uint_prepared((idx << 2) as u32, &mut scratch.borrow());
        let mut this_cycle_measurement = crate::Measurement::new();
        interpreter.read_and_prepare_instruction_components(
            1,
            ctx.params.module(),
            &ctx.keys_prepared,
            Some(&ctx.sk_glwe_prepared),
            &mut scratch.borrow(),
            &mut this_cycle_measurement,
        );

        let dec = |ct: &FheUint<Vec<u8>, u32, i64>, scratch: &mut ScratchOwned<BE>| -> u32 {
            ct.decrypt(
                ctx.params.module(),
                &ctx.sk_glwe_prepared,
                &mut scratch.borrow(),
            )
        };

        let dec_rs1: u32 = dec(&interpreter.rs1_addr_fhe_uint, &mut scratch);
        let dec_rs2: u32 = dec(&interpreter.rs2_addr_fhe_uint, &mut scratch);
        let dec_rd: u32 = dec(&interpreter.rd_addr_fhe_uint, &mut scratch);
        let dec_imm: u32 = dec(&interpreter.imm_val_fhe_uint, &mut scratch);
        let dec_rdu: u32 = dec(&interpreter.rdu_val_fhe_uint, &mut scratch);
        let dec_mu: u32 = dec(&interpreter.mu_val_fhe_uint, &mut scratch);
        let dec_pcu: u32 = dec(&interpreter.pcu_val_fhe_uint, &mut scratch);

        assert_eq!(correct_imm, dec_imm);
        assert_eq!(correct_rs1, dec_rs1);
        assert_eq!(correct_rs2, dec_rs2);
        assert_eq!(correct_rd, dec_rd);
        assert_eq!(correct_rdu, dec_rdu);
        assert_eq!(correct_mu, dec_mu);
        assert_eq!(correct_pcu, dec_pcu);
    }
}

#[test]
fn test_interpreter_cycle_single_instruction_noop_fft64_ref() {
    test_interpreter_cycle_single_instruction_noop::<CGGI, FFT64Ref>();
}

fn test_interpreter_cycle_single_instruction_noop<BRA: BlindRotationAlgo, BE: PhantomBackend>()
where
    Module<BE>: ModuleNew<BE> + PhantomBootstrapModule<BRA, BE>,
    ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
{
    let mut ctx: TestContext<BRA, BE> = TestContext::new();
    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 24);

    let rom_size = 1 << 10;
    let ram_size = 1 << 10;
    let mut interpreter: Interpreter<BE> = Interpreter::new(&ctx.params, rom_size, ram_size);

    let instructions_u32 = vec![
        // 258455
        0b00000000_00000000_00000000_1110011,
        0b00000000_00000000_00000000_1110011,
    ];

    let mut instructions = InstructionsParser::new();
    for inst in instructions_u32.clone() {
        instructions.add(Instruction::new(inst));
    }

    interpreter.instructions_encrypt_sk(
        ctx.params.module(),
        &instructions,
        &ctx.sk_glwe_prepared,
        &mut ctx.source_xa,
        &mut ctx.source_xe,
        &mut scratch.borrow(),
    );

    interpreter.pc_fhe_uint = ctx.encrypt_fhe_uint(0, &mut scratch.borrow());

    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 28);

    interpreter.set_verbose_timings(true);
    interpreter.set_threads(16);

    println!("Cycle");
    interpreter.cycle(
        ctx.params.module(),
        &ctx.keys_prepared,
        &mut scratch.borrow(),
    );
    println!("Cycle done");

    let pc = interpreter.pc_fhe_uint.decrypt(
        ctx.params.module(),
        &ctx.sk_glwe_prepared,
        &mut scratch.borrow(),
    );
    println!("PC: {}", pc);
    assert_eq!(pc, 4);
}
